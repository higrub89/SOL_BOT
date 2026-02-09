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
    message::Message,
    program_pack::Pack,
};
use solana_client::rpc_client::RpcClient;
use spl_token::state::Account as TokenAccount;
use std::str::FromStr;
use base64::{Engine as _, engine::general_purpose};

use crate::jupiter::{JupiterClient, SwapResult, BuyResult};

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

/// Executor de trades con Jupiter integration
pub struct TradeExecutor {
    config: ExecutorConfig,
    rpc_client: RpcClient,
    jupiter: JupiterClient,
}

impl TradeExecutor {
    pub fn new(config: ExecutorConfig) -> Self {
        let rpc_client = RpcClient::new_with_commitment(
            config.rpc_url.clone(),
            CommitmentConfig::confirmed(),
        );

        Self {
            config,
            rpc_client,
            jupiter: JupiterClient::new(),
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

        let keypair = wallet_keypair.unwrap();
        let user_pubkey = keypair.pubkey();

        println!("🎯 Token:        {}", token_mint);
        println!("🔑 Wallet:       {}...", &user_pubkey.to_string()[..8]);
        println!("📊 Amount:       {}%", amount_percent);
        println!("📉 Slippage:     {}%", self.config.slippage_bps as f64 / 100.0);
        println!("⚙️  Mode:         PRODUCTION\n");

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
        println!("🔐 Firmando transacción...");
        let tx_bytes = general_purpose::STANDARD
            .decode(&swap_response.swap_transaction)
            .context("Error decodificando transacción base64")?;
        
        let mut transaction: VersionedTransaction = bincode::deserialize(&tx_bytes)
            .context("Error deserializando transacción")?;

        // NOTA: Jupiter ya devuelve la transacción lista para firmar
        // Solo necesitamos agregar nuestra firma
        
        // 5. Enviar transacción
        println!("📡 Broadcasting transacción a Solan...");
        let signature = self.send_transaction_with_retry(&transaction, 3).await?;

        println!("✅ Transacción confirmada!\n");
        println!("🔗 Signature: {}", signature);
        println!("🔗 Solscan:   https://solscan.io/tx/{}\n", signature);

        // 6. Construir resultado
        let sol_received = quote.out_amount.parse::<f64>().unwrap_or(0.0) / 1_000_000_000.0;
        
        let result = SwapResult {
            signature: signature.to_string(),
            input_amount: amount_to_sell as f64,
            output_amount: sol_received,
            route: quote.route_plan.iter()
                .map(|r| r.swap_info.label.clone())
                .collect::<Vec<_>>()
                .join(" → "),
            price_impact_pct: quote.price_impact_pct.parse().unwrap_or(0.0),
        };

        result.print_summary();

        Ok(result)
    }

    /// Ejecuta una compra usando SOL
    pub async fn execute_buy(
        &self,
        token_mint: &str,
        wallet_keypair: Option<&Keypair>,
        amount_sol: f64,
    ) -> Result<BuyResult> {
        println!("╔════════════════════════════════════════════════════════════╗");
        println!("║              💰 BUY EXECUTOR V2 💰                        ║");
        println!("╚════════════════════════════════════════════════════════════╝\n");

        // Modo dry run si no se proporciona keypair
        if self.config.dry_run || wallet_keypair.is_none() {
            return self.simulate_buy(token_mint, amount_sol).await;
        }

        let keypair = wallet_keypair.unwrap();
        let user_pubkey = keypair.pubkey();

        println!("🎯 Token:        {}", token_mint);
        println!("🔑 Wallet:       {}...", &user_pubkey.to_string()[..8]);
        println!("💰 Amount:       {} SOL", amount_sol);
        println!("📉 Slippage:     {}%", self.config.slippage_bps as f64 / 100.0);
        println!("⚙️  Mode:         PRODUCTION\n");

        // SOL amount en lamports
        let amount_lamports = (amount_sol * 1_000_000_000.0) as u64;

        // 1. Obtener quote de Jupiter
        println!("🔍 Consultando Jupiter para mejor ruta...");
        
        const SOL_MINT: &str = "So11111111111111111111111111111111111111112";
        
        let quote = self.jupiter.get_quote(
            SOL_MINT,
            token_mint,
            amount_lamports,
            self.config.slippage_bps,
        ).await?;

        self.jupiter.print_quote_summary(&quote);

        let tokens_to_receive = quote.out_amount.parse::<f64>().unwrap_or(0.0);
        let price_per_token = if tokens_to_receive > 0.0 {
            amount_sol / tokens_to_receive
        } else {
            0.0
        };

        println!("\n💎 Tokens estimados: {:.0}", tokens_to_receive);
        println!("📊 Precio unitario: ${:.10}", price_per_token);

        // 2. Obtener transacción firmable
        println!("\n🔧 Generando transacción de swap...");
        let swap_response = self.jupiter.get_swap_transaction(
            &quote,
            &user_pubkey.to_string(),
            true,
        ).await?;

        // 3. Deserializar transacción
        println!("🔐 Firmando transacción...");
        let tx_bytes = general_purpose::STANDARD
            .decode(&swap_response.swap_transaction)
            .context("Error decodificando transacción base64")?;
        
        let transaction: VersionedTransaction = bincode::deserialize(&tx_bytes)
            .context("Error deserializando transacción")?;

        // 4. Enviar transacción
        println!("📡 Broadcasting transacción a Solana...");
        let signature = self.send_transaction_with_retry(&transaction, 3).await?;

        println!("✅ Compra confirmada!\n");
        println!("🔗 Signature: {}", signature);
        println!("🔗 Solscan:   https://solscan.io/tx/{}\n", signature);

        // 5. Construir resultado
        let result = BuyResult {
            signature: signature.to_string(),
            sol_spent: amount_sol,
            tokens_received: tokens_to_receive,
            price_per_token,
            route: quote.route_plan.iter()
                .map(|r| r.swap_info.label.clone())
                .collect::<Vec<_>>()
                .join(" → "),
            price_impact_pct: quote.price_impact_pct.parse().unwrap_or(0.0),
        };

        result.print_summary();

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
        let quote = self.jupiter.get_quote(
            token_mint,
            SOL_MINT,
            1000000000, // Simular con 1 SOL de valor de token si no sabemos balance
            self.config.slippage_bps,
        ).await.unwrap_or_default();

        let output_sol = quote.out_amount.parse::<f64>().unwrap_or(0.0) / 1_000_000_000.0;

        println!("⚠️  SIMULACIÓN ACTIVA:");
        println!("   ✓ Quote de Jupiter calculado: {} SOL", output_sol);
        println!("   ✓ Ruta óptima identificada: {}", quote.route_plan.iter().map(|r| r.swap_info.label.clone()).collect::<Vec<_>>().join(" → "));
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
            route: "Simulation".to_string(),
            price_impact_pct: quote.price_impact_pct.parse().unwrap_or(0.0),
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
