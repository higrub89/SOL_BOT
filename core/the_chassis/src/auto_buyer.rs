//! # Auto-Buyer Inteligente (HFT)
//! 
//! Orquestador final que conecta Sensores, Decision Engine y Ejecución.
//! Coordina la compra automática minimizando riesgo y optimizando la entrada.

use anyhow::{Result, anyhow};
use solana_sdk::signature::Keypair;
use crate::engine::{DecisionEngine, TokenContext, ExecutionParams};
use crate::executor_v2::{TradeExecutor, ExecutorConfig};
use crate::raydium::RaydiumExecutor; // Future integration
use std::sync::Arc;

/// Configuración de Compra Automática
#[derive(Debug, Clone)]
pub struct AutoBuyConfig {
    pub token_mint: String,
    pub symbol: Option<String>,
    pub amount_sol: f64,
    pub slippage_bps: u16,      // Slippage base (Engine ajustará si es necesario)
    pub add_to_monitoring: bool,
    pub stop_loss_percent: f64,
    pub trailing_enabled: bool,
}

/// Resultado de una compra exitosa
#[derive(Debug)]
pub struct BuyResult {
    pub signature: String,
    pub token_mint: String,
    pub amount_sol: f64,
    pub tokens_received: f64,
    pub effective_price: f64,
    pub route: String, // "Raydium Direct" | "Jupiter Aggregator"
}

pub struct AutoBuyer {
    engine: DecisionEngine,
    executor: Arc<TradeExecutor>,
    // raydium_executor: Arc<RaydiumExecutor>, // TODO: Phase 2
}

impl AutoBuyer {
    /// Crea una nueva instancia del AutoBuyer
    pub fn new(rpc_url: String) -> Result<Self> {
        let config = ExecutorConfig {
            rpc_url: rpc_url.clone(),
            slippage_bps: 200, // 2% default base
            priority_fee: 100_000, // 0.0001 SOL base
            dry_run: false,
        };

        Ok(Self {
            engine: DecisionEngine::new(),
            executor: Arc::new(TradeExecutor::new(config)),
        })
    }

    /// Ejecuta el proceso completo de compra inteligente
    pub async fn buy(
        &self, 
        config: &AutoBuyConfig, 
        wallet: &Keypair
    ) -> Result<BuyResult> {
        println!("🤖 AUTO-BUYER: Analizando {}...", config.token_mint);

        // 1. Construir Contexto (Sensores)
        // Nota: En fase 1, estos datos vendrán de APIs externas simuladas o basicas
        // En fase 2, vendrán de gRPC real-time
        let ctx = self.build_context(&config.token_mint).await?;

        // 2. Evaluar Decision Engine (Filtros + Actuadores)
        let exec_params = match self.engine.evaluate(&ctx) {
            Ok(params) => params,
            Err(reason) => {
                println!("❌ AUTO-BUY RECHAZADO: {}", reason);
                return Err(anyhow!("Rechazado por Engine: {}", reason));
            }
        };

        println!("✅ AUTO-BUY APROBADO | Stage: {:?} | Tip: {} lamports | Slippage: {} bps", 
            exec_params.maturity_stage, 
            exec_params.priority_fee_lamports, 
            exec_params.slippage_bps
        );

        // 3. Ejecución (Routing Inteligente)
        // Intentar Raydium primero (Fase 2), Fallback a Jupiter (Fase 1)
        
        // TODO: Implementar lógica de selección de ruta real
        // Por ahora, usamos Jupiter con los parámetros optimizados del Engine
        
        let swap_result = self.executor.execute_buy_with_custom_params(
            &config.token_mint,
            Some(wallet),
            config.amount_sol,
            exec_params.priority_fee_lamports,
            exec_params.slippage_bps
        ).await?;

        Ok(BuyResult {
            signature: swap_result.signature,
            token_mint: config.token_mint.clone(),
            amount_sol: swap_result.input_amount,
            tokens_received: swap_result.output_amount, // Ajustar según quote real
            effective_price: swap_result.price_impact_pct, // Placeholder
            route: "Jupiter (Optimized)".to_string(),
        })
    }

    /// Construye el contexto del token consultando APIs externas (Simulado por ahora)
    async fn build_context(&self, mint: &str) -> Result<TokenContext> {
        // TODO: Conectar a Helius/DexScreener para datos reales
        // Por ahora retornamos un contexto "Ideal" para probar el flujo
        Ok(TokenContext {
            mint: mint.to_string(),
            symbol: "UNKNOWN".to_string(),
            age_minutes: 30, // Simulando Momentum Core
            liquidity_usd: 50_000.0,
            volume_5m: 10_000.0,
            price_usd: 0.001,
            momentum_slope: 0.5, // Fuerte subida simulada
            unique_wallets_ratio: 0.4, // Saludable
            top_10_holders_pct: 15.0,
            dev_wallet_pct: 0.0,
            mint_authority: None,
            freeze_authority: None,
            lp_burned_pct: 100.0,
        })
    }
}
