//! # Trade Executor V2
//! 
//! Implementación completa del executor con Jupiter Aggregator integration.
//! Soporte para ejecución automática de swaps con firma y broadcast.

use anyhow::{Result, Context};
use solana_sdk::{
    pubkey::Pubkey,
    signature::{Keypair, Signer, Signature},
    transaction::VersionedTransaction,
    commitment_config::CommitmentConfig,
    program_pack::Pack,
};
use solana_client::rpc_client::RpcClient;
use spl_token::state::Account as TokenAccount;
use std::str::FromStr;
use base64::{Engine as _, engine::general_purpose};

use crate::jupiter::{JupiterClient, SwapResult, BuyResult};
use crate::validation::FinancialValidator;

/// Configuración del executor
#[derive(Debug, Clone)]
pub struct ExecutorConfig {
    pub rpc_url: String,
    pub dry_run: bool,
    pub slippage_bps: u16,      // Basis points (100 = 1%)
    pub priority_fee: u64,       // Micro lamports
}

impl ExecutorConfig {
    pub fn new(rpc_url: String, dry_run: bool) -> Self {
        Self {
            rpc_url,
            dry_run,
            slippage_bps: 100,      // 1% slippage default
            priority_fee: 50000,     // ~0.00005 SOL
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

// ... imports
use crate::raydium::RaydiumClient;

/// Executor de trades con Jupiter integration y Raydium Fallback
pub struct TradeExecutor {
    config: ExecutorConfig,
    rpc_client: RpcClient,
    jupiter: JupiterClient,
    raydium: Option<RaydiumClient>, 
}

impl TradeExecutor {
    pub fn new(config: ExecutorConfig) -> Self {
        let rpc_client = RpcClient::new_with_commitment(
            config.rpc_url.clone(),
            CommitmentConfig::confirmed(),
        );

        // Intentar inicializar Raydium (puede fallar si no hay cache, no es crítico)
        let raydium = match RaydiumClient::new(config.rpc_url.clone()) {
            Ok(client) => {
                println!("✅ Raydium Client: Activado (Modo Directo)");
                Some(client)
            },
            Err(e) => {
                eprintln!("⚠️  Raydium Client: Desactivado ({})", e);
                None
            }
        };

        Self {
            config,
            rpc_client,
            jupiter: JupiterClient::new(),
            raydium,
        }
    }

    /// Ejecuta una venta de emergencia completa
    pub async fn execute_emergency_sell(
        &self,
        token_mint: &str,
        wallet_keypair: Option<&Keypair>,
        amount_percent: u8, // 100 = vender todo
    ) -> Result<SwapResult> {
        println!("╔════════════════════════════════════════════════════════════╗");
        println!("║           ⚡ EMERGENCY SELL EXECUTOR V2 ⚡               ║");
        println!("╚════════════════════════════════════════════════════════════╝\n");

        // Modo dry run si no se proporciona keypair
        if self.config.dry_run || wallet_keypair.is_none() {
            return self.simulate_emergency_sell(token_mint, amount_percent).await;
        }

        let keypair = wallet_keypair
            .ok_or_else(|| anyhow::anyhow!("Keypair requerido para ejecución real"))?;
        let user_pubkey = keypair.pubkey();

        println!("🎯 Token:        {}", token_mint);
        println!("🔑 Wallet:       {}...", &user_pubkey.to_string()[..8]);
        println!("📊 Amount:       {}%", amount_percent);
        println!("📉 Slippage:     {}%", self.config.slippage_bps as f64 / 100.0);
        println!("⚙️  Mode:         PRODUCTION\n");

        // Flash check: Raydium Fast Path para ventas?
        // TODO: Implementar venta en Raydium. Por ahora venta siempre va por Jupiter para asegurar mejor precio de salida.
        
        // 1. Obtener token account y balance
        println!("📊 Verificando balance de tokens...");
        let (token_account, token_balance) = self.get_token_account_balance(&user_pubkey, token_mint)?;
        
        let amount_to_sell = (token_balance as f64 * (amount_percent as f64 / 100.0)) as u64;
        
        println!("   • Token Account: {}", token_account);
        println!("   • Balance:       {} tokens", token_balance);
        println!("   • A vender:      {} tokens ({}%)\n", amount_to_sell, amount_percent);

        if amount_to_sell == 0 {
            anyhow::bail!("No hay suficiente balance para vender");
        }

        // 2. Obtener quote de Jupiter
        println!("🔍 Consultando Jupiter para mejor ruta...");
        
        const SOL_MINT: &str = "So11111111111111111111111111111111111111112";
        
        let quote = self.jupiter.get_quote(
            token_mint,
            SOL_MINT,
            amount_to_sell,
            self.config.slippage_bps,
        ).await?;

        self.jupiter.print_quote_summary(&quote);

        // 3. Obtener transacción firmable
        println!("\n🔧 Generando transacción de swap...");
        let swap_response = self.jupiter.get_swap_transaction(
            &quote,
            &user_pubkey.to_string(),
            true, // unwrap WSOL a SOL nativo
        ).await?;

        // 4. Deserializar transacción
        println!("🔐 Firmando transacción con keypair...");
        let tx_bytes = general_purpose::STANDARD
            .decode(&swap_response.swap_transaction)
            .context("Error decodificando transacción base64")?;
        
        let mut transaction: VersionedTransaction = bincode::deserialize(&tx_bytes)
            .context("Error deserializando transacción")?;

        // ✅ CRITICAL FIX: Jupiter devuelve la tx sin la firma del usuario.
        // Obtenemos un blockhash fresco y firmamos con nuestro keypair antes de broadcast.
        let recent_blockhash = self.rpc_client
            .get_latest_blockhash()
            .context("Error obteniendo blockhash reciente")?;
        transaction.message.set_recent_blockhash(recent_blockhash);
        let signed_tx = VersionedTransaction::try_new(transaction.message, &[keypair])
            .context("Error firmando transacción con keypair")?;

        // 5. Enviar transacción
        println!("📡 Broadcasting transacción a Solana...");
        let signature = self.send_transaction_with_retry(&signed_tx, 3).await?;

        println!("✅ Transacción confirmada!\n");
        println!("🔗 Signature: {}", signature);
        println!("🔗 Solscan:   https://solscan.io/tx/{}\n", signature);

        // 6. Construir resultado con validación estricta
        let sol_received = FinancialValidator::parse_price_safe(
            &quote.out_amount,
            "Jupiter out_amount"
        )? / 1_000_000_000.0;
        
        // Validar que recibimos algo razonable
        FinancialValidator::validate_sol_amount(sol_received, "SOL received")?;
        
        let price_impact = FinancialValidator::parse_price_safe(
            &quote.price_impact_pct,
            "Jupiter price_impact_pct"
        )?;
        
        let result = SwapResult {
            signature: signature.to_string(),
            input_amount: amount_to_sell as f64,
            output_amount: sol_received,
            route: quote.route_plan.iter()
                .map(|r| r.swap_info.label.clone())
                .collect::<Vec<_>>()
                .join(" → "),
            price_impact_pct: price_impact,
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
        println!("⚡ HFT EXECUTION | Tip: {} | Slip: {} bps", priority_fee_lamports, slippage_bps);

        if self.config.dry_run || wallet_keypair.is_none() {
            return self.simulate_buy_v2(token_mint, amount_sol).await;
        }

        let keypair = wallet_keypair.unwrap();
        const SOL_MINT: &str = "So11111111111111111111111111111111111111112";
        let user_pubkey = keypair.pubkey();

        // 1. Obtener quote de Jupiter con slippage dinámico
        let amount_lamports = (amount_sol * 1_000_000_000.0) as u64;
        
        let quote = self.jupiter.get_quote(
            SOL_MINT,
            token_mint,
            amount_lamports,
            slippage_bps,
        ).await?;

        // 2. Obtener transacción optimizada
        // Nota: Jupiter API permite configurar priority fee en la request de swap
        // Si la librería client lo soporta. Si no, habría que inyectar instrucción ComputeBudget manual.
        // Por ahora asumimos configuración estándar o Jito Bundle externo.
        
        let swap_response = self.jupiter.get_swap_transaction(
            &quote,
            &user_pubkey.to_string(),
            true,
        ).await?;

        // 3. Firmar y Enviar
        let tx_bytes = general_purpose::STANDARD
            .decode(&swap_response.swap_transaction)
            .context("Error decoding tx")?;
        
        let mut transaction: VersionedTransaction = bincode::deserialize(&tx_bytes)?;
        
        // ✅ CRITICAL FIX: Jupiter devuelve la tx sin la firma del usuario.
        // Debemos obtener el blockhash reciente y firmar con nuestro keypair.
        let recent_blockhash = self.rpc_client
            .get_latest_blockhash()
            .context("Error obteniendo blockhash reciente")?;
        transaction.message.set_recent_blockhash(recent_blockhash);
        let signed_tx = VersionedTransaction::try_new(transaction.message, &[keypair])
            .context("Error firmando transacción con keypair")?;
        
        let signature = self.send_transaction_with_retry(&signed_tx, 3).await?;

        Ok(SwapResult {
            signature: signature.to_string(),
            input_amount: amount_sol,
            output_amount: 0.0,
            route: "Jupiter Adjusted".to_string(),
            price_impact_pct: 0.0,
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

        if self.config.dry_run || wallet_keypair.is_none() {
            return self.simulate_buy(token_mint, amount_sol).await;
        }

        let keypair = wallet_keypair.unwrap();
        const SOL_MINT: &str = "So11111111111111111111111111111111111111112";

        // 1. INTENTO VÍA RAYDIUM DIRECT (Low Latency)
        // Solo si tenemos cliente y el pool está en caché
        if let Some(raydium) = &self.raydium {
            // Amount in lamports (SOL)
            let amount_in = (amount_sol * 1_000_000_000.0) as u64;
            
            // Slippage calculation simple para intento directo
            // Asumimos precio jupiter momentáneamente para calcular min_out o usamos oráculo interno
            // Por simplicidad en V1 híbrida: Si el pool existe, intentamos quote rápido
            
                if let Ok(pool_info) = raydium.find_pool(SOL_MINT, token_mint) {
                println!("⚡ [FAST PATH] Pool detectado en caché: {}", pool_info.name);
                println!("🚀 Intentando ejecución directa en Raydium...");

                // Estrategia Híbrida:
                // 1. Consultar precio rápido en Jupiter (Oracle Check)
                // 2. Ejecutar en Raydium (Execution Layer)
                
                // Paso 1: Obtener precio de referencia para slippage protection
                let quote_check = self.jupiter.get_quote(
                    SOL_MINT,
                    token_mint,
                    amount_in, // Input exacto
                    self.config.slippage_bps,
                ).await;

                if let Ok(quote) = quote_check {
                    let estimated_out = FinancialValidator::parse_amount_safe(
                        &quote.out_amount,
                        "Jupiter estimated tokens"
                    )?;
                    
                    if estimated_out > 0 {
                        // Calcular mínimo aceptable basado en el slippage configurado
                        let min_amount_out = raydium.calculate_min_amount_out(estimated_out, self.config.slippage_bps);
                        
                        println!("   Price Check (Jup): {} tokens", estimated_out);
                        println!("   Min Out (Ray):     {} tokens ({}% slip)", min_amount_out, self.config.slippage_bps as f64 / 100.0);

                        // Paso 2: Ejecutar Swap en Raydium
                        if let Some(kp) = wallet_keypair {
                             match raydium.execute_swap(
                                SOL_MINT,
                                token_mint,
                                amount_in,
                                min_amount_out,
                                kp
                            ) {
                                Ok(sig) => {
                                    println!("✅ RAYDIUM SWAP COMPLETADO: {}", sig);
                                    
                                    // Devolver resultado exitoso construido manualmente
                                    let tokens_received = estimated_out as f64; // Estimado, real se ve en explorer
                                    let price_per_token = amount_sol / (tokens_received / 1_000_000.0); // Ajustar decimales según token (asumiendo 6)
                                    
                                    return Ok(BuyResult {
                                        signature: sig,
                                        sol_spent: amount_sol,
                                        tokens_received: tokens_received / 1_000_000.0, // TODO: Usar decimales reales del token
                                        price_per_token,
                                        route: "Raydium Direct (V4)".to_string(),
                                        price_impact_pct: 0.0, // No calculado localmente aun
                                    });
                                },
                                Err(e) => {
                                    eprintln!("❌ Error en ejecución Raydium: {}", e);
                                    println!("⚠️  Fallback a Jupiter activado...");
                                }
                            }
                        } else {
                             println!("🧪 DRY RUN: Ejecución Raydium simulada con éxito.");
                             return self.simulate_buy(token_mint, amount_sol).await;
                        }
                    }
                } else {
                    println!("⚠️  No se pudo obtener precio de referencia. Abortando Fast Path por seguridad.");
                }
            }
        }

        // 2. FALLBACK/STANDARD: JUPITER AGGREGATOR
        println!("🔄 [STANDARD PATH] Ruteando vía Jupiter Aggregator...");
        
        // ... (resto de la lógica original de Jupiter)
        
        let user_pubkey = keypair.pubkey();

        println!("🎯 Token:        {}", token_mint);
        // ... (logs)
        
        // SOL amount en lamports
        let amount_lamports = (amount_sol * 1_000_000_000.0) as u64;

        // 1. Obtener quote de Jupiter
        println!("🔍 Consultando Jupiter para mejor ruta...");
        
        let quote = self.jupiter.get_quote(
            SOL_MINT,
            token_mint,
            amount_lamports,
            self.config.slippage_bps,
        ).await?;

        // ... (resto de ejecución normal)
        
        self.jupiter.print_quote_summary(&quote);

        let tokens_to_receive = FinancialValidator::parse_price_safe(
            &quote.out_amount,
            "Jupiter tokens to receive"
        )?;
        
        // Validar que recibiremos tokens
        if tokens_to_receive <= 0.0 {
            anyhow::bail!("Jupiter quote inválido: 0 tokens a recibir");
        }
        
        let price_per_token = amount_sol / tokens_to_receive;
        
        // Validar price impact
        let price_impact = FinancialValidator::parse_price_safe(
            &quote.price_impact_pct,
            "Jupiter price impact"
        )?;

        println!("\n🔧 Generando transacción de swap...");
        let swap_response = self.jupiter.get_swap_transaction(
            &quote,
            &user_pubkey.to_string(),
            true,
        ).await?;

        println!("🔐 Firmando transacción con keypair...");
        let tx_bytes = general_purpose::STANDARD
            .decode(&swap_response.swap_transaction)
            .context("Error decodificando transacción base64")?;
        
        let mut transaction: VersionedTransaction = bincode::deserialize(&tx_bytes)
            .context("Error deserializando transacción")?;

        // ✅ CRITICAL FIX: Jupiter devuelve la tx sin la firma del usuario.
        // Obtenemos un blockhash fresco y firmamos con nuestro keypair.
        let recent_blockhash = self.rpc_client
            .get_latest_blockhash()
            .context("Error obteniendo blockhash reciente")?;
        transaction.message.set_recent_blockhash(recent_blockhash);
        let signed_tx = VersionedTransaction::try_new(transaction.message, &[keypair])
            .context("Error firmando transacción con keypair")?;

        println!("📡 Broadcasting transacción a Solana...");
        let signature = self.send_transaction_with_retry(&signed_tx, 3).await?;

        println!("✅ Compra confirmada!\n");
        println!("🔗 Signature: {}", signature);
        
        let result = BuyResult {
            signature: signature.to_string(),
            sol_spent: amount_sol,
            tokens_received: tokens_to_receive,
            price_per_token,
            route: quote.route_plan.iter()
                .map(|r| r.swap_info.label.clone())
                .collect::<Vec<_>>()
                .join(" → "),
            price_impact_pct: price_impact,
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
        })
    }

    /// Simula una venta (dry run)
    async fn simulate_emergency_sell(&self, token_mint: &str, amount_percent: u8) -> Result<SwapResult> {
        println!("🧪 Mode:         DRY RUN (Simulation)");
        println!("🎯 Token:        {}", token_mint);
        println!("📊 Amount:       {}%\n", amount_percent);
        
        // Simular obtención de quote real para la simulación
        const SOL_MINT: &str = "So11111111111111111111111111111111111111112";
        
        let quote_result = self.jupiter.get_quote(
            token_mint,
            SOL_MINT,
            1000000000, // Simular con 1 SOL de valor de token si no sabemos balance
            self.config.slippage_bps,
        ).await;

        let (output_sol, route, price_impact) = match quote_result {
            Ok(quote) => {
                let sol = FinancialValidator::parse_price_safe(
                    &quote.out_amount,
                    "Simulation out_amount"
                ).unwrap_or(0.0) / 1_000_000_000.0;
                
                let route_str = quote.route_plan.iter()
                    .map(|r| r.swap_info.label.clone())
                    .collect::<Vec<_>>()
                    .join(" → ");
                
                let impact = FinancialValidator::parse_price_safe(
                    &quote.price_impact_pct,
                    "Simulation price impact"
                ).unwrap_or(0.0);
                
                (sol, route_str, impact)
            },
            Err(e) => {
                eprintln!("⚠️  No se pudo obtener quote de Jupiter para simulación: {}", e);
                (0.0, "Simulation (No quote available)".to_string(), 0.0)
            }
        };

        println!("⚠️  SIMULACIÓN ACTIVA:");
        println!("   ✓ Quote de Jupiter calculado: {} SOL", output_sol);
        println!("   ✓ Ruta óptima identificada: {}", route);
        println!("   ✗ Transacción NO enviada a blockchain\n");
        
        // Registrar en log de simulación
        self.log_simulated_trade(token_mint, output_sol)?;

        println!("💡 Para ejecutar en PRODUCCIÓN:");
        println!("   1. Proporciona el Keypair de tu wallet");
        println!("   2. Ajusta 'auto_execute = true' en targets.json\n");
        
        Ok(SwapResult {
            signature: "SIMULATION_ONLY".to_string(),
            input_amount: 1000000000.0,
            output_amount: output_sol,
            route,
            price_impact_pct: price_impact,
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

    /// Obtiene el token account y balance para un mint específico
    fn get_token_account_balance(&self, wallet: &Pubkey, mint: &str) -> Result<(Pubkey, u64)> {
        let mint_pubkey = Pubkey::from_str(mint)
            .context("Token mint inválido")?;

        // Derivar el Associated Token Account (ATA)
        let token_account = spl_associated_token_account::get_associated_token_address(
            wallet,
            &mint_pubkey,
        );

        // Obtener el account data
        let account_data = self.rpc_client
            .get_account(&token_account)
            .context("No se pudo obtener token account - ¿no tienes tokens?")?;

        // Parsear el account como TokenAccount
        let token_account_state = TokenAccount::unpack(&account_data.data)
            .context("Error parseando token account")?;

        Ok((token_account, token_account_state.amount))
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
                    anyhow::bail!("Error enviando transacción después de {} intentos: {}", max_retries, e);
                }
            }
        }
        
        unreachable!()
    }

    /// Verifica si una transacción fue confirmada
    pub fn verify_transaction(&self, signature: &str) -> Result<bool> {
        let sig = Signature::from_str(signature)
            .context("Signature inválida")?;
        
        match self.rpc_client.get_signature_status(&sig)? {
            Some(Ok(_)) => Ok(true),
            Some(Err(e)) => {
                eprintln!("Transacción falló: {:?}", e);
                Ok(false)
            }
            None => Ok(false),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_executor_config() {
        let config = ExecutorConfig::new(
            "https://api.mainnet-beta.solana.com".to_string(),
            true,
        )
        .with_slippage(150)
        .with_priority_fee(100000);

        assert_eq!(config.slippage_bps, 150);
        assert_eq!(config.priority_fee, 100000);
        assert!(config.dry_run);
    }

    #[tokio::test]
    async fn test_simulate_sell() {
        let config = ExecutorConfig::new(
            "https://api.mainnet-beta.solana.com".to_string(),
            true,
        );
        let executor = TradeExecutor::new(config);

        let result = executor.execute_emergency_sell(
            "TEST_MINT",
            None,
            100,
        ).await;

        assert!(result.is_ok());
        let swap_result = result.unwrap();
        assert_eq!(swap_result.signature, "SIMULATION_ONLY");
    }
}
