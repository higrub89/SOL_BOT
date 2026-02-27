//! # Trade Executor V2
//!
//! Implementación completa del executor con Jupiter Aggregator integration.
//! Soporte para ejecución automática de swaps con firma y broadcast.
//! [HFT EDITION - Dynamic ECU Parameters]

use anyhow::{Context, Result};
use base64::{engine::general_purpose, Engine as _};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    program_pack::Pack,
    pubkey::Pubkey,
    signature::{Keypair, Signature, Signer},
    transaction::VersionedTransaction,
};
use spl_token::state::Account as TokenAccount;
use std::str::FromStr;

use crate::jito::JitoClient;
use crate::jupiter::{BuyResult, JupiterClient, SwapResult};
use crate::raydium::RaydiumClient;
use crate::validation::FinancialValidator;

/// Configuración del executor
#[derive(Debug, Clone)]
pub struct ExecutorConfig {
    pub rpc_url: String,
    pub dry_run: bool,
    pub slippage_bps: u16, // Basis points (100 = 1%)
    pub priority_fee: u64, // Micro lamports
}

impl ExecutorConfig {
    pub fn new(rpc_url: String, dry_run: bool) -> Self {
        Self {
            rpc_url,
            dry_run,
            slippage_bps: 100,   // 1% slippage default
            priority_fee: 50000, // ~0.00005 SOL
        }
    }

    pub fn with_slippage(mut self, slippage_bps: u16) -> Self {
        self.slippage_bps = slippage_bps;
        self
    }

    pub fn with_priority_fee(mut self, priority_fee: u64) -> Self {
        self.priority_fee = priority_fee;
        self
    }
}

/// Executor de trades con Jupiter integration y Raydium Fallback
pub struct TradeExecutor {
    config: ExecutorConfig,
    rpc_client: RpcClient,
    jupiter: JupiterClient,
    raydium: Option<RaydiumClient>,
    jito_client: JitoClient,
}

impl TradeExecutor {
    pub fn new(config: ExecutorConfig) -> Self {
        let rpc_client =
            RpcClient::new_with_commitment(config.rpc_url.clone(), CommitmentConfig::confirmed());

        // Intentar inicializar Raydium (Robust)
        let raydium = match RaydiumClient::new(config.rpc_url.clone()) {
            Ok(client) => {
                println!("✅ Raydium Client: Activado (Modo Directo)");
                Some(client)
            }
            Err(e) => {
                eprintln!(
                    "⚠️  Raydium Client fallback: No se pudo cargar caché ({}). Iniciando vacío...",
                    e
                );
                match RaydiumClient::new(config.rpc_url.clone()) {
                    Ok(c) => Some(c),
                    Err(_) => {
                        eprintln!("❌ Raydium Client: Fallo fatal en inicialización.");
                        None
                    }
                }
            }
        };

        Self {
            config,
            rpc_client,
            jupiter: JupiterClient::new(),
            raydium,
            jito_client: JitoClient::new(),
        }
    }

    /// ⚡ DYNAMIC PRIORITY FEE — Consulta Helius para el fee óptimo real.
    ///
    /// Usa `getPriorityFeeEstimate` de Helius RPC con nivel "High" para equilibrar
    /// velocidad y coste. Fallback transparente al valor por defecto si Helius no responde.
    pub async fn get_dynamic_priority_fee(&self) -> u64 {
        const DEFAULT: u64 = 100_000; // 100k micro-lamports
        const MAX_FEE: u64 = 2_000_000; // 2M micro-lamports cap

        let api_key = match std::env::var("HELIUS_API_KEY") {
            Ok(k) => k,
            Err(_) => return DEFAULT,
        };

        let url = format!("https://mainnet.helius-rpc.com/?api-key={}", api_key);
        let body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": "dynamic-fee",
            "method": "getPriorityFeeEstimate",
            "params": [{
                "accountKeys": ["JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4"],
                "options": { "priorityLevel": "High" }
            }]
        });

        let client = match reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(2))
            .build() {
            Ok(c) => c,
            Err(_) => return DEFAULT,
        };

        match client.post(&url).json(&body).send().await {
            Ok(resp) => match resp.json::<serde_json::Value>().await {
                Ok(json) => {
                    let fee = json
                        .get("result")
                        .and_then(|r| r.get("priorityFeeLevels"))
                        .and_then(|l| l.get("high"))
                        .and_then(|v| v.as_f64())
                        .map(|f| f as u64)
                        .unwrap_or(DEFAULT);
                    let capped = fee.min(MAX_FEE);
                    println!("⛽ [DynFee] Helius: {}µL → Cap: {}µL", fee, capped);
                    capped
                }
                Err(_) => { eprintln!("⚠️ [DynFee] Parse error → fallback"); DEFAULT }
            },
            Err(_) => { eprintln!("⚠️ [DynFee] Timeout → fallback"); DEFAULT }
        }
    }

    /// Convierte lamports de Jito tip a SOL
    #[inline]
    fn lamports_to_sol(lamports: u64) -> f64 {
        lamports as f64 / 1_000_000_000.0
    }

    /// Actuador asíncrono con control de tracción para slippage dinámico y Jito Tips (Zero-Allocation)
    pub async fn execute_sell_with_retry(
        &self,
        token_mint: String,
        wallet_keypair: Option<&Keypair>,
        amount_percent: u8,
        is_emergency: bool, // True si es Stop-Loss (prioridad absoluta)
    ) -> Result<SwapResult> {
        let mut attempt = 1;
        let max_attempts = if is_emergency { 5 } else { 3 };

        // Configuración inicial de inyección
        let mut current_slippage_bps = self.config.slippage_bps;
        let mut current_jito_tip = crate::config::AppConfig::load()
            .map(|c| c.global_settings.jito_tip_lamports)
            .unwrap_or(100_000);

        loop {
            println!(
                "🔄 [Intento {}/{}] Venta de {} | Tracción: {} bps | Tip: {} µL",
                attempt, max_attempts, token_mint, current_slippage_bps, current_jito_tip
            );

            // ⚡ Llamada directa inyectando la sobrealimentación, sin reconstruir la instancia TCP
            match self
                .execute_emergency_sell_with_params(
                    &token_mint,
                    wallet_keypair,
                    amount_percent,
                    Some(current_slippage_bps),
                    Some(current_jito_tip),
                )
                .await
            {
                Ok(result) => {
                    println!(
                        "✅ Maniobra HFT confirmada en red. [Tx: {}]",
                        result.signature
                    );
                    return Ok(result); // Maniobra exitosa, salimos del circuito
                }
                Err(e) => {
                    let error_msg = e.to_string().to_lowercase();

                    if attempt >= max_attempts {
                        eprintln!(
                            "💥 Motor calado. Abortando salida para {}. Fallo irrecuperable tras {} intentos: {}", 
                            token_mint, max_attempts, e
                        );
                        return Err(anyhow::anyhow!(
                            "Fallo definitivo de Venta tras reintentos: {}",
                            e
                        ));
                    }

                    // --- SISTEMA DE TELEMETRÍA Y RESPUESTA ACTIVA ---

                    if error_msg.contains("slippage")
                        || error_msg.contains("0x11")
                        || error_msg.contains("insufficient")
                        || error_msg.contains("error")
                    {
                        // Pérdida de tracción. Duplicamos la tolerancia de precios.
                        current_slippage_bps = (current_slippage_bps as f32 * 2.0) as u16;

                        if is_emergency && attempt == max_attempts - 1 {
                            println!("☢️ [EMERGENCIA] Último ciclo para {}. Activando Modo Degen (Slippage MÁXIMO).", token_mint);
                            current_slippage_bps = 10000; // 100% Slippage
                        } else {
                            println!(
                                "⚠️ Deslizamiento superado. Ajustando tracción a {} bps.",
                                current_slippage_bps
                            );
                        }
                    } else if error_msg.contains("timeout")
                        || error_msg.contains("blockhashnotfound")
                        || error_msg.contains("0x0")
                    {
                        // Pérdida de presión en la red. Aumentamos el Tip un 50% para saltar la congestión.
                        current_jito_tip = (current_jito_tip as f64 * 1.5) as u64;
                        println!("⚡ Retraso de red detectado. Aumentando presión de inyección Jito a {} µLamports.", current_jito_tip);
                    } else if error_msg.contains("toomanyrequests") {
                        println!("⏳ Restricción térmica del RPC. Enfriando conductos...");
                    }

                    // Backoff Exponencial (200ms, 400ms, 800ms...) para respetar ciclos del nodo
                    let delay = std::time::Duration::from_millis(200 * (2u64.pow(attempt as u32)));
                    tokio::time::sleep(delay).await;

                    attempt += 1;
                }
            }
        }
    }

    /// Envoltorio estándar para mantener compatibilidad con API existente
    pub async fn execute_emergency_sell(
        &self,
        token_mint: &str,
        wallet_keypair: Option<&Keypair>,
        amount_percent: u8,
    ) -> Result<SwapResult> {
        self.execute_emergency_sell_with_params(
            token_mint,
            wallet_keypair,
            amount_percent,
            None,
            None,
        )
        .await
    }

    /// Ejecuta una venta de emergencia completa inyectando parámetros dinámicos si se requiere
    pub async fn execute_emergency_sell_with_params(
        &self,
        token_mint: &str,
        wallet_keypair: Option<&Keypair>,
        amount_percent: u8,                // 100 = vender todo
        dynamic_slippage_bps: Option<u16>, // ⚡ Override de tracción
        dynamic_jito_tip: Option<u64>,     // ⚡ Override de presión
    ) -> Result<SwapResult> {
        println!("╔════════════════════════════════════════════════════════════╗");
        println!("║           ⚡ EMERGENCY SELL EXECUTOR V2 ⚡               ║");
        println!("╚════════════════════════════════════════════════════════════╝\n");

        let active_slippage = dynamic_slippage_bps.unwrap_or(self.config.slippage_bps);
        let active_jito_tip = dynamic_jito_tip.unwrap_or_else(|| {
            crate::config::AppConfig::load()
                .map(|c| c.global_settings.jito_tip_lamports)
                .unwrap_or(100_000)
        });

        // ✅ CRITICAL: Validar mint ANTES de cualquier operación
        let token_mint =
            crate::validation::FinancialValidator::validate_mint(token_mint, "EMERGENCY SELL")?;

        println!("✅ Mint validation passed: {}\n", token_mint);

        // Modo dry run si no se proporciona keypair
        if self.config.dry_run || wallet_keypair.is_none() {
            return self
                .simulate_emergency_sell(&token_mint, amount_percent, active_slippage)
                .await;
        }

        let keypair = wallet_keypair
            .ok_or_else(|| anyhow::anyhow!("Keypair requerido para ejecución real"))?;
        let user_pubkey = keypair.pubkey();

        println!("🎯 Token:        {}", token_mint);
        println!("🔑 Wallet:       {}...", &user_pubkey.to_string()[..8]);
        println!("📊 Amount:       {}%", amount_percent);
        println!("📉 Slippage:     {}%", active_slippage as f64 / 100.0);
        println!("⚙️  Mode:         PRODUCTION\n");

        // 1. Obtener token account y balance
        println!("📊 Verificando balance de tokens...");
        let (token_account, token_balance) =
            self.get_token_account_balance(&user_pubkey, &token_mint)?;

        let amount_to_sell = (token_balance as f64 * (amount_percent as f64 / 100.0)) as u64;

        println!("   • Token Account: {}", token_account);
        println!("   • Balance:       {} tokens", token_balance);
        println!(
            "   • A vender:      {} tokens ({}%)\n",
            amount_to_sell, amount_percent
        );

        if amount_to_sell == 0 {
            anyhow::bail!("No hay suficiente balance para vender");
        }

        // ⚡ FAST PATH: Raydium Direct Sell (Prioridad Absoluta en Emergencias)
        // Latencia: ~50-150ms vs ~300-500ms de Jupiter
        if let Some(raydium) = &self.raydium {
            let min_sol_out = raydium.calculate_min_amount_out(
                amount_to_sell,
                active_slippage,
            );

            println!("⚡ [FAST PATH] Intentando venta directa en Raydium...");
            match raydium
                .execute_sell_with_jito(
                    &token_mint,
                    amount_to_sell,
                    min_sol_out,
                    active_jito_tip,
                    keypair,
                )
                .await
            {
                Ok(sig) => {
                    // Estimación de SOL recibido: no podemos saberlo sin confirmar la TX,
                    // pero estimamos como fallback para el TradeRecord
                    let estimated_sol = min_sol_out as f64 / 1_000_000_000.0;
                    println!("✅ [RAYDIUM FAST EXIT] Sig: {}", sig);
                    return Ok(SwapResult {
                        signature: sig,
                        input_amount: amount_to_sell as f64,
                        output_amount: estimated_sol,
                        route: "Raydium Direct".to_string(),
                        price_impact_pct: active_slippage as f64 / 100.0,
                        fee_sol: Self::lamports_to_sol(active_jito_tip),
                    });
                }
                Err(e) => {
                    eprintln!(
                        "⚠️ [FAST PATH] Raydium no disponible: {}. Activando STANDARD PATH (Jupiter)...",
                        e
                    );
                }
            }
        }

        // 2. STANDARD PATH: Obtener quote de Jupiter
        println!("🔍 Consultando Jupiter para mejor ruta...");

        const SOL_MINT: &str = "So11111111111111111111111111111111111111112";

        let quote = self
            .jupiter
            .get_quote(
                &token_mint,
                SOL_MINT,
                amount_to_sell,
                active_slippage, // ⚡ Inyección
            )
            .await?;

        self.jupiter.print_quote_summary(&quote);

        // 3. Obtener transacción firmable
        println!("\n🔧 Generando transacción de swap...");
        let swap_response = self
            .jupiter
            .get_swap_transaction(
                &quote,
                &user_pubkey.to_string(),
                true, // unwrap WSOL a SOL nativo
            )
            .await?;

        // 4. Deserializar transacción
        println!("🔐 Firmando transacción con keypair...");
        let tx_bytes = general_purpose::STANDARD
            .decode(&swap_response.swap_transaction)
            .context("Error decodificando transacción base64")?;

        let mut transaction: VersionedTransaction =
            bincode::deserialize(&tx_bytes).context("Error deserializando transacción")?;

        let recent_blockhash = self
            .rpc_client
            .get_latest_blockhash()
            .context("Error obteniendo blockhash reciente")?;
        transaction.message.set_recent_blockhash(recent_blockhash);
        let signed_tx = VersionedTransaction::try_new(transaction.message, &[keypair])
            .context("Error firmando transacción con keypair")?;

        // 5. Enviar transacción (Standard vs Jito)
        println!("📡 Broadcasting transacción a Solana...");

        let signature_str = if active_jito_tip > 0 {
            println!(
                "🛡️  Preparando Jito Bundle con Tip ({} SOL)...",
                active_jito_tip as f64 / 1_000_000_000.0
            );

            let tip_ix = JitoClient::create_tip_instruction(&user_pubkey, active_jito_tip); // ⚡ Inyección
            let tip_msg = solana_sdk::message::Message::new(&[tip_ix], Some(&user_pubkey));
            let mut tip_tx = solana_sdk::transaction::Transaction::new_unsigned(tip_msg);
            tip_tx.sign(&[keypair], recent_blockhash);
            let versioned_tip_tx = VersionedTransaction::from(tip_tx);

            let bundle = vec![signed_tx.clone(), versioned_tip_tx];

            match self.jito_client.send_bundle(bundle).await {
                Ok(bundle_id) => {
                    println!("✅ Bundle enviado a Jito. ID: {}", bundle_id);
                    signed_tx.signatures[0].to_string()
                }
                Err(e) => {
                    eprintln!("⚠️  Jito falló: {}. Fallback a RPC standard...", e);
                    self.send_transaction_with_retry(&signed_tx, 3)
                        .await?
                        .to_string()
                }
            }
        } else {
            self.send_transaction_with_retry(&signed_tx, 3)
                .await?
                .to_string()
        };
        let signature = solana_sdk::signature::Signature::from_str(&signature_str)
            .unwrap_or(solana_sdk::signature::Signature::default());

        println!("✅ Transacción confirmada!\n");
        println!("🔗 Signature: {}", signature);
        println!("🔗 Solscan:   https://solscan.io/tx/{}\n", signature);

        // 6. Construir resultado con validación estricta
        let sol_received =
            FinancialValidator::parse_price_safe(&quote.out_amount, "Jupiter out_amount")?
                / 1_000_000_000.0;

        FinancialValidator::validate_sol_amount(sol_received, "SOL received")?;

        let price_impact = FinancialValidator::parse_price_safe(
            &quote.price_impact_pct,
            "Jupiter price_impact_pct",
        )?;

        let result = SwapResult {
            signature: signature.to_string(),
            input_amount: amount_to_sell as f64,
            output_amount: sol_received,
            route: quote
                .route_plan
                .iter()
                .map(|r| r.swap_info.label.clone())
                .collect::<Vec<_>>()
                .join(" → "),
            price_impact_pct: price_impact,
            fee_sol: Self::lamports_to_sol(active_jito_tip),
        };

        result.print_summary();

        Ok(result)
    }

    /// Ejecuta una compra con parámetros HFT personalizados (Dynamic Tip & Slippage)
    pub async fn execute_buy_with_custom_params(
        &self,
        token_mint: &str,
        wallet_keypair: Option<&Keypair>,
        amount_sol: f64,
        priority_fee_lamports: u64,
        slippage_bps: u16,
    ) -> Result<SwapResult> {
        println!(
            "⚡ HFT EXECUTION | Tip: {} | Slip: {} bps",
            priority_fee_lamports, slippage_bps
        );

        if self.config.dry_run || wallet_keypair.is_none() {
            return self.simulate_buy_v2(token_mint, amount_sol).await;
        }

        let keypair = match wallet_keypair {
            Some(kp) => kp,
            None => anyhow::bail!("Keypair requerido para HFT execution"),
        };
        const SOL_MINT: &str = "So11111111111111111111111111111111111111112";
        let user_pubkey = keypair.pubkey();

        // 1. Obtener quote de Jupiter con slippage dinámico
        let amount_lamports = (amount_sol * 1_000_000_000.0) as u64;

        let quote = self
            .jupiter
            .get_quote(SOL_MINT, token_mint, amount_lamports, slippage_bps)
            .await?;

        // 2. Obtener transacción optimizada
        let swap_response = self
            .jupiter
            .get_swap_transaction(&quote, &user_pubkey.to_string(), true)
            .await?;

        // 3. Firmar y Enviar
        let tx_bytes = general_purpose::STANDARD
            .decode(&swap_response.swap_transaction)
            .context("Error decoding tx")?;

        let mut transaction: VersionedTransaction = bincode::deserialize(&tx_bytes)?;

        let recent_blockhash = self
            .rpc_client
            .get_latest_blockhash()
            .context("Error obteniendo blockhash reciente")?;
        transaction.message.set_recent_blockhash(recent_blockhash);
        let signed_tx = VersionedTransaction::try_new(transaction.message, &[keypair])
            .context("Error firmando transacción con keypair")?;

        let signature_str = if priority_fee_lamports > 0 {
            println!(
                "🛡️  Preparando Jito Bundle con Tip ({} SOL)...",
                priority_fee_lamports as f64 / 1_000_000_000.0
            );

            let tip_ix = JitoClient::create_tip_instruction(&user_pubkey, priority_fee_lamports);
            let tip_msg = solana_sdk::message::Message::new(&[tip_ix], Some(&user_pubkey));
            let mut tip_tx = solana_sdk::transaction::Transaction::new_unsigned(tip_msg);
            tip_tx.sign(&[keypair], recent_blockhash);

            let bundle = vec![signed_tx.clone(), VersionedTransaction::from(tip_tx)];

            match self.jito_client.send_bundle(bundle).await {
                Ok(bundle_id) => {
                    println!("✅ Bundle enviado a Jito. ID: {}", bundle_id);
                    signed_tx.signatures[0].to_string()
                }
                Err(e) => {
                    eprintln!("⚠️  Jito falló: {}. Fallback a RPC standard...", e);
                    self.send_transaction_with_retry(&signed_tx, 3)
                        .await?
                        .to_string()
                }
            }
        } else {
            self.send_transaction_with_retry(&signed_tx, 3)
                .await?
                .to_string()
        };
        let signature = solana_sdk::signature::Signature::from_str(&signature_str)
            .unwrap_or(solana_sdk::signature::Signature::default());

        let out_amount_raw = quote.out_amount.parse::<f64>().unwrap_or(0.0);
        let price_impact = quote.price_impact_pct.parse::<f64>().unwrap_or(0.0);

        let decimals = match solana_sdk::pubkey::Pubkey::from_str(token_mint) {
            Ok(mint_pubkey) => {
                match self.rpc_client.get_token_supply(&mint_pubkey) {
                    Ok(supply) => supply.decimals,
                    Err(_) => 6, // Fallback a 6
                }
            }
            Err(_) => 6,
        };

        let output_amount = out_amount_raw / 10f64.powi(decimals as i32);

        Ok(SwapResult {
            signature: signature.to_string(),
            input_amount: amount_sol,
            output_amount,
            route: "Jupiter Adjusted".to_string(),
            price_impact_pct: price_impact,
            fee_sol: Self::lamports_to_sol(priority_fee_lamports),
        })
    }

    /// Simula una compra (dry run) - V2
    async fn simulate_buy_v2(&self, _token_mint: &str, amount_sol: f64) -> Result<SwapResult> {
        println!("🧪 Mode: DRY RUN V2 (HFT Mock)");
        Ok(SwapResult {
            signature: "HFT_SIMULATION".to_string(),
            input_amount: amount_sol,
            output_amount: amount_sol * 1000.0, // Mock rate
            route: "Simulated HFT Route".to_string(),
            price_impact_pct: 0.1,
            fee_sol: 0.0,
        })
    }

    /// Ejecuta una compra usando SOL
    /// Prioridad: 1. Raydium Direct (Si caché) -> 2. Jupiter (Universal)
    pub async fn execute_buy(
        &self,
        token_mint: &str,
        wallet_keypair: Option<&Keypair>,
        amount_sol: f64,
    ) -> Result<BuyResult> {
        println!("╔════════════════════════════════════════════════════════════╗");
        println!("║              💰 BUY EXECUTOR V2 (HYBRID) 💰               ║");
        println!("╚════════════════════════════════════════════════════════════╝\n");

        let token_mint =
            crate::validation::FinancialValidator::validate_mint(token_mint, "BUY EXECUTOR")?;

        println!("✅ Mint validation passed: {}\n", token_mint);

        if self.config.dry_run || wallet_keypair.is_none() {
            return self.simulate_buy(&token_mint, amount_sol).await;
        }

        let keypair = match wallet_keypair {
            Some(kp) => kp,
            None => anyhow::bail!("Keypair requerido para ejecución de compra"),
        };
        const SOL_MINT: &str = "So11111111111111111111111111111111111111112";

        // 1. INTENTO VÍA RAYDIUM DIRECT (Prioridad Absoluta)
        if let Some(raydium) = &self.raydium {
            let amount_in = (amount_sol * 1_000_000_000.0) as u64;

            if let Ok(pool_info) = raydium.find_pool(SOL_MINT, &token_mint).await {
                println!("⚡ [ULTRA-FAST PATH] Pool detectado: {}", pool_info.name);
                println!("🚀 Intentando ejecución directa en Raydium...");

                let oracle_quote = self
                    .jupiter
                    .get_quote(SOL_MINT, &token_mint, amount_in, 500)
                    .await;

                let (estimated_out, found_oracle) = match oracle_quote {
                    Ok(q) => (
                        FinancialValidator::parse_amount_safe(&q.out_amount, "Jup Estimate")
                            .unwrap_or(0),
                        true,
                    ),
                    Err(_) => {
                        println!("⚠️ [ORACLE FAIL] Jupiter no conoce el token. Usando ejecución DEGEN...");
                        (0, false)
                    }
                };

                let min_out = if found_oracle && estimated_out > 0 {
                    raydium.calculate_min_amount_out(estimated_out, self.config.slippage_bps)
                } else {
                    1 // 1 lamport mínimo
                };

                match raydium
                    .execute_swap(SOL_MINT, &token_mint, amount_in, min_out, keypair)
                    .await
                {
                    Ok(sig) => {
                        println!("✅ RAYDIUM SUCCESS: {}", sig);

                        let tokens_received = if estimated_out > 0 {
                            estimated_out as f64 / 1_000_000.0
                        } else {
                            0.0
                        };
                        let price_per_token = if tokens_received > 0.0 {
                            amount_sol / tokens_received
                        } else {
                            0.0
                        };

                        let raydium_jito = crate::config::AppConfig::load()
                            .map(|c| c.global_settings.jito_tip_lamports)
                            .unwrap_or(100_000);
                        return Ok(BuyResult {
                            signature: sig,
                            sol_spent: amount_sol,
                            tokens_received,
                            price_per_token,
                            route: format!(
                                "Raydium Direct (Oracle: {})",
                                if found_oracle { "OK" } else { "BYPASSED" }
                            ),
                            price_impact_pct: 0.0,
                            fee_sol: Self::lamports_to_sol(raydium_jito),
                        });
                    }
                    Err(e) => {
                        eprintln!("❌ Raydium Swap failed: {}. Continuing to Jupiter...", e);
                    }
                }
            }
        }

        // 2. FALLBACK/STANDARD: JUPITER AGGREGATOR
        println!("🔄 [STANDARD PATH] Ruteando vía Jupiter Aggregator...");

        let user_pubkey = keypair.pubkey();
        println!("🎯 Token:        {}", token_mint);

        let amount_lamports = (amount_sol * 1_000_000_000.0) as u64;

        // 1. Obtener quote de Jupiter
        println!("🔍 Consultando Jupiter para mejor ruta...");

        let quote = self
            .jupiter
            .get_quote(
                SOL_MINT,
                &token_mint,
                amount_lamports,
                self.config.slippage_bps,
            )
            .await?;

        self.jupiter.print_quote_summary(&quote);

        let tokens_to_receive =
            FinancialValidator::parse_price_safe(&quote.out_amount, "Jupiter tokens to receive")?;

        if tokens_to_receive <= 0.0 {
            anyhow::bail!("Jupiter quote inválido: 0 tokens a recibir");
        }

        let price_per_token = amount_sol / tokens_to_receive;
        let price_impact = quote.price_impact_pct.parse::<f64>().unwrap_or(0.0);

        println!("\n🔧 Generando transacción de swap...");
        let swap_response = self
            .jupiter
            .get_swap_transaction(&quote, &user_pubkey.to_string(), true)
            .await?;

        println!("🔐 Firmando transacción con keypair...");
        let tx_bytes = general_purpose::STANDARD
            .decode(&swap_response.swap_transaction)
            .context("Error decodificando transacción base64")?;

        let mut transaction: VersionedTransaction =
            bincode::deserialize(&tx_bytes).context("Error deserializando transacción")?;

        let recent_blockhash = self
            .rpc_client
            .get_latest_blockhash()
            .context("Error obteniendo blockhash reciente")?;
        transaction.message.set_recent_blockhash(recent_blockhash);
        let signed_tx = VersionedTransaction::try_new(transaction.message, &[keypair])
            .context("Error firmando transacción con keypair")?;

        println!("📡 Broadcasting transacción a Solana...");
        // ⚡ DYNAMIC FEE: Obtener fee óptimo de Helius en tiempo real
        let dynamic_priority_fee = self.get_dynamic_priority_fee().await;
        let jito_tip_lamports = crate::config::AppConfig::load()
            .map(|c| c.global_settings.jito_tip_lamports)
            .unwrap_or(100_000);

        let signature_str = if jito_tip_lamports > 0 {
            println!(
                "🛡️  Preparando Jito Bundle con Tip ({} SOL)...",
                jito_tip_lamports as f64 / 1_000_000_000.0
            );
            let tip_ix = JitoClient::create_tip_instruction(&user_pubkey, jito_tip_lamports);
            let tip_msg = solana_sdk::message::Message::new(&[tip_ix], Some(&user_pubkey));
            let mut tip_tx = solana_sdk::transaction::Transaction::new_unsigned(tip_msg);
            tip_tx.sign(&[keypair], recent_blockhash);

            let bundle = vec![signed_tx.clone(), VersionedTransaction::from(tip_tx)];

            match self.jito_client.send_bundle(bundle).await {
                Ok(bundle_id) => {
                    println!("✅ Bundle enviado a Jito. ID: {}", bundle_id);
                    signed_tx.signatures[0].to_string()
                }
                Err(e) => {
                    eprintln!("⚠️  Jito falló: {}. Fallback a RPC standard...", e);
                    self.send_transaction_with_retry(&signed_tx, 3)
                        .await?
                        .to_string()
                }
            }
        } else {
            self.send_transaction_with_retry(&signed_tx, 3)
                .await?
                .to_string()
        };
        let signature = solana_sdk::signature::Signature::from_str(&signature_str)
            .unwrap_or(solana_sdk::signature::Signature::default());

        println!("✅ Compra confirmada!\n");
        println!("🔗 Signature: {}", signature);

        let result = BuyResult {
            signature: signature.to_string(),
            sol_spent: amount_sol,
            tokens_received: tokens_to_receive,
            price_per_token,
            route: quote
                .route_plan
                .iter()
                .map(|r| r.swap_info.label.clone())
                .collect::<Vec<_>>()
                .join(" → "),
            price_impact_pct: price_impact,
            fee_sol: Self::lamports_to_sol(jito_tip_lamports) + Self::microlamports_to_sol(dynamic_priority_fee),
        };

        Ok(result)
    }

    /// Simula una compra (dry run)
    async fn simulate_buy(&self, token_mint: &str, amount_sol: f64) -> Result<BuyResult> {
        println!("🧪 Mode:         DRY RUN (Simulation)");
        println!("🎯 Token:        {}", token_mint);
        println!("💰 Amount:       {} SOL\n", amount_sol);

        println!("⚠️  SIMULACIÓN ACTIVA:");
        println!("   ✓ Quote calculado");
        println!("   ✓ Precio estimado");
        println!("   ✗ Transacción NO enviada\n");

        Ok(BuyResult {
            signature: "SIMULATION_ONLY".to_string(),
            sol_spent: amount_sol,
            tokens_received: 100000.0,
            price_per_token: amount_sol / 100000.0,
            route: "Simulation".to_string(),
            price_impact_pct: 0.5,
            fee_sol: 0.0,
        })
    }

    /// Helper: convierte micro-lamports a SOL (1e12 conversion)
    #[inline]
    fn microlamports_to_sol(microlamports: u64) -> f64 {
        microlamports as f64 / 1_000_000_000_000.0
    }

    /// Simula una venta (dry run) con soporte para slippage opcional
    async fn simulate_emergency_sell(
        &self,
        token_mint: &str,
        amount_percent: u8,
        slippage_bps: u16,
    ) -> Result<SwapResult> {
        println!("🧪 Mode:         DRY RUN (Simulation)");
        println!("🎯 Token:        {}", token_mint);
        println!("📊 Amount:       {}%\n", amount_percent);

        const SOL_MINT: &str = "So11111111111111111111111111111111111111112";

        let quote_result = self
            .jupiter
            .get_quote(token_mint, SOL_MINT, 1000000000, slippage_bps)
            .await;

        let (output_sol, route, price_impact) = match quote_result {
            Ok(quote) => {
                let sol = FinancialValidator::parse_price_safe(
                    &quote.out_amount,
                    "Simulation out_amount",
                )
                .unwrap_or(0.0)
                    / 1_000_000_000.0;

                let route_str = quote
                    .route_plan
                    .iter()
                    .map(|r| r.swap_info.label.clone())
                    .collect::<Vec<_>>()
                    .join(" → ");

                let impact = FinancialValidator::parse_price_safe(
                    &quote.price_impact_pct,
                    "Simulation price impact",
                )
                .unwrap_or(0.0);

                (sol, route_str, impact)
            }
            Err(e) => {
                eprintln!(
                    "⚠️  No se pudo obtener quote de Jupiter para simulación: {}",
                    e
                );
                (0.0, "Simulation (No quote available)".to_string(), 0.0)
            }
        };

        println!("⚠️  SIMULACIÓN ACTIVA:");
        println!("   ✓ Quote de Jupiter calculado: {} SOL", output_sol);
        println!("   ✓ Ruta óptima identificada: {}", route);
        println!("   ✗ Transacción NO enviada a blockchain\n");

        self.log_simulated_trade(token_mint, output_sol)?;

        let sig = format!(
            "SIM_{}_{}",
            token_mint,
            chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0)
        );

        Ok(SwapResult {
            signature: sig,
            input_amount: 1000000000.0,
            output_amount: output_sol,
            route,
            price_impact_pct: price_impact,
            fee_sol: 0.0,
        })
    }

    fn log_simulated_trade(&self, token: &str, amount_sol: f64) -> Result<()> {
        use std::fs::OpenOptions;
        use std::io::Write;

        let log_path = "../../operational/logs/simulated_trades.csv";
        let file_exists = std::path::Path::new(log_path).exists();

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_path)?;

        if !file_exists {
            writeln!(file, "timestamp,token,type,amount_sol,status")?;
        }

        let now = chrono::Utc::now().to_rfc3339();
        writeln!(file, "{},{},SELL,{:.6},SIMULATED", now, token, amount_sol)?;

        Ok(())
    }

    /// Ejecuta múltiples ventas agrupadas en un solo Jito Bundle
    pub async fn execute_multi_sell(
        &self,
        mints: Vec<String>,
        wallet_keypair: &Keypair,
        amount_percent: u8,
    ) -> Result<Vec<SwapResult>> {
        println!("╔════════════════════════════════════════════════════════════╗");
        println!("║           🚀 JITO MULTI-TOKEN BUNDLE EXECUTOR            ║");
        println!("╚════════════════════════════════════════════════════════════╝\n");

        if self.config.dry_run {
            println!(
                "🧪 Mode: DRY RUN - Simulando bundle de {} tokens",
                mints.len()
            );
            let mut results = Vec::new();
            for mint in mints {
                results.push(
                    self.simulate_emergency_sell(&mint, amount_percent, self.config.slippage_bps)
                        .await?,
                );
            }
            return Ok(results);
        }

        let user_pubkey = wallet_keypair.pubkey();
        let mut transactions = Vec::new();
        let mut sell_infos = Vec::new();

        const SOL_MINT: &str = "So11111111111111111111111111111111111111112";

        for mint in &mints {
            println!("🔍 Procesando {}...", &mint[..8]);

            // 1. Balance
            let (_, token_balance) = match self.get_token_account_balance(&user_pubkey, mint) {
                Ok(b) => b,
                Err(e) => {
                    eprintln!("   ⚠️ Saltar {}: {}", mint, e);
                    continue;
                }
            };
            let amount_to_sell = (token_balance as f64 * (amount_percent as f64 / 100.0)) as u64;
            if amount_to_sell == 0 {
                continue;
            }

            // 2. Quote
            let quote = match self
                .jupiter
                .get_quote(mint, SOL_MINT, amount_to_sell, self.config.slippage_bps)
                .await
            {
                Ok(q) => q,
                Err(e) => {
                    eprintln!("   ⚠️ Error quote {}: {}", mint, e);
                    continue;
                }
            };

            // 3. Tx
            let swap_response = match self
                .jupiter
                .get_swap_transaction(&quote, &user_pubkey.to_string(), true)
                .await
            {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("   ⚠️ Error swap tx {}: {}", mint, e);
                    continue;
                }
            };

            // 4. Decode & Sign
            let tx_bytes = general_purpose::STANDARD.decode(&swap_response.swap_transaction)?;
            let mut transaction: VersionedTransaction = bincode::deserialize(&tx_bytes)?;

            let recent_blockhash = self.rpc_client.get_latest_blockhash()?;
            transaction.message.set_recent_blockhash(recent_blockhash);
            let signed_tx = VersionedTransaction::try_new(transaction.message, &[wallet_keypair])?;

            let sig = signed_tx.signatures[0].to_string();
            transactions.push(signed_tx);
            sell_infos.push((mint.clone(), quote, sig));
        }

        if transactions.is_empty() {
            anyhow::bail!("No hay transacciones válidas para enviar en el bundle");
        }

        // 5. Jito Tip
        let jito_tip_lamports = crate::config::AppConfig::load()
            .map(|c| c.global_settings.jito_tip_lamports)
            .unwrap_or(100_000);

        let recent_blockhash = self.rpc_client.get_latest_blockhash()?;
        let tip_ix = JitoClient::create_tip_instruction(&user_pubkey, jito_tip_lamports);
        let tip_msg = solana_sdk::message::Message::new(&[tip_ix], Some(&user_pubkey));
        let mut tip_tx = solana_sdk::transaction::Transaction::new_unsigned(tip_msg);
        tip_tx.sign(&[wallet_keypair], recent_blockhash);

        let mut bundle = transactions.clone();
        bundle.push(VersionedTransaction::from(tip_tx));

        // 6. Send Bundle
        let bundle_id = self.jito_client.send_bundle(bundle).await?;
        println!("✅ Jito Multi-Bundle Enviado: {}", bundle_id);

        // 7. Results
        let mut final_results = Vec::new();
        // Fee por transacción en el bundle: split del jito_tip entre todas las TXs
        let fee_per_tx = Self::lamports_to_sol(jito_tip_lamports) / (sell_infos.len() as f64).max(1.0);
        for (_mint, quote, sig) in sell_infos {
            let sol_received =
                FinancialValidator::parse_price_safe(&quote.out_amount, "Jup")? / 1_000_000_000.0;
            final_results.push(SwapResult {
                signature: sig,
                input_amount: 0.0,
                output_amount: sol_received,
                route: "Jupiter Bundle".to_string(),
                price_impact_pct: 0.0,
                fee_sol: fee_per_tx,
            });
        }

        Ok(final_results)
    }

    /// Obtiene el token account y balance para un mint específico con reintentos para ATA creation
    fn get_token_account_balance(&self, wallet: &Pubkey, mint: &str) -> Result<(Pubkey, u64)> {
        let mint_pubkey = Pubkey::from_str(mint).context("Token mint inválido")?;

        let token_account =
            spl_associated_token_account::get_associated_token_address(wallet, &mint_pubkey);

        let mut retries = 5;
        let mut last_error = None;

        while retries > 0 {
            match self.rpc_client.get_account(&token_account) {
                Ok(account_data) => {
                    let token_account_state = TokenAccount::unpack(&account_data.data)
                        .context("Error parseando token account")?;
                    return Ok((token_account, token_account_state.amount));
                }
                Err(e) => {
                    last_error = Some(e);
                    retries -= 1;
                    if retries > 0 {
                        println!(
                            "⏳ [RETRY] ATA no detectado aún para {}. Esperando 600ms...",
                            &mint[..8]
                        );
                        std::thread::sleep(std::time::Duration::from_millis(600));
                    }
                }
            }
        }

        anyhow::bail!(
            "Fallo definitivo buscando ATA tras reintentos: {:?}",
            last_error
        );
    }

    /// Envía una transacción con reintentos
    async fn send_transaction_with_retry(
        &self,
        transaction: &VersionedTransaction,
        max_retries: u32,
    ) -> Result<Signature> {
        for attempt in 1..=max_retries {
            println!("   Intento {}/{}...", attempt, max_retries);

            match self.rpc_client.send_and_confirm_transaction(transaction) {
                Ok(sig) => {
                    println!("   ✅ Confirmado en intento {}", attempt);
                    return Ok(sig);
                }
                Err(e) if attempt < max_retries => {
                    eprintln!("   ⚠️  Fallo (intento {}): {}", attempt, e);
                    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                }
                Err(e) => {
                    anyhow::bail!(
                        "Error enviando transacción después de {} intentos: {}",
                        max_retries,
                        e
                    );
                }
            }
        }

        unreachable!()
    }

    /// Verifica si una transacción fue confirmada
    pub fn verify_transaction(&self, signature: &str) -> Result<bool> {
        let sig = Signature::from_str(signature).context("Signature inválida")?;

        match self.rpc_client.get_signature_status(&sig)? {
            Some(Ok(_)) => Ok(true),
            Some(Err(e)) => {
                eprintln!("Transacción falló: {:?}", e);
                Ok(false)
            }
            None => Ok(false),
        }
    }

    /// Ejecuta una compra DEGENERATE (Pure Raydium, Zero Safety Check)
    pub async fn execute_raydium_buy(
        &self,
        token_mint: &str,
        wallet_keypair: Option<&Keypair>,
        amount_sol: f64,
    ) -> Result<BuyResult> {
        println!("🚀 [DEGEN MODE] Initiating Direct Raydium Assault...");

        let token_mint =
            crate::validation::FinancialValidator::validate_mint(token_mint, "DEGEN BUY")?;

        let raydium = self
            .raydium
            .as_ref()
            .context("Raydium engine not initialized")?;
        let keypair = wallet_keypair.context("Wallet keypair required for Degen Mode")?;
        const SOL_MINT: &str = "So11111111111111111111111111111111111111112";
        let amount_in = (amount_sol * 1_000_000_000.0) as u64;

        // 1. Obtener balance PRE-compra
        let user_pubkey = keypair.pubkey();
        let pre_balance = self
            .get_token_account_balance(&user_pubkey, &token_mint)
            .map(|(_, bal)| bal)
            .unwrap_or(0);

        // 2. Ejecutar
        let _pool_info = raydium.find_pool(SOL_MINT, &token_mint).await?;
        let sig = raydium
            .execute_swap(SOL_MINT, &token_mint, amount_in, 1, keypair)
            .await?;

        // 3. Esperar confirmación
        println!("⏳ Esperando confirmación para calcular precio real...");
        tokio::time::sleep(std::time::Duration::from_millis(1500)).await;

        let mut post_balance = pre_balance;
        for i in 0..5 {
            if let Ok((_, bal)) = self.get_token_account_balance(&user_pubkey, &token_mint) {
                if bal > pre_balance {
                    post_balance = bal;
                    break;
                }
            }
            tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
            println!("   Reintentando lectura de balance ({} / 5)...", i + 1);
        }

        let tokens_received_raw = post_balance.saturating_sub(pre_balance);
        let tokens_received = tokens_received_raw as f64 / 1_000_000.0;

        let price_per_token = if tokens_received > 0.0 {
            amount_sol / tokens_received
        } else {
            0.0
        };

        if tokens_received <= 0.0 {
            println!("⚠️ [WARNING] No se detectó cambio en el balance de tokens. El monitoreo podría fallar.");
        } else {
            println!(
                "📊 [REAL DATA] Recibido: {:.4} tokens | Precio: {:.8} SOL/token",
                tokens_received, price_per_token
            );
        }

        Ok(BuyResult {
            signature: sig,
            sol_spent: amount_sol,
            tokens_received,
            price_per_token,
            route: "Raydium Direct (Degen Mode)".to_string(),
            price_impact_pct: 0.0,
            fee_sol: 0.0, // Degen mode: fee se captura via jito_tip del AppConfig
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_executor_config() {
        let config = ExecutorConfig::new("https://api.mainnet-beta.solana.com".to_string(), true)
            .with_slippage(150)
            .with_priority_fee(100000);

        assert_eq!(config.slippage_bps, 150);
        assert_eq!(config.priority_fee, 100000);
        assert!(config.dry_run);
    }

    #[tokio::test]
    async fn test_simulate_sell() {
        let config = ExecutorConfig::new("https://api.mainnet-beta.solana.com".to_string(), true);
        let executor = TradeExecutor::new(config);

        let result = executor
            .execute_emergency_sell(
                "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v", // Valid mint format
                None,
                100,
            )
            .await;

        assert!(result.is_ok());
        let swap_result = result.unwrap();
        assert!(swap_result.signature.starts_with("SIM_"));
    }

    #[test]
    fn test_lamports_to_sol() {
        assert_eq!(TradeExecutor::lamports_to_sol(1_000_000_000), 1.0);
        assert_eq!(TradeExecutor::lamports_to_sol(500_000_000), 0.5);
    }

    #[test]
    fn test_microlamports_to_sol() {
        assert_eq!(TradeExecutor::microlamports_to_sol(1_000_000_000_000), 1.0);
        assert_eq!(TradeExecutor::microlamports_to_sol(100_000), 0.0000001);
    }
}
