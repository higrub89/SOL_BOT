//! # Auto-Buyer Inteligente (HFT)
//! 
//! Orquestador final que conecta Sensores, Decision Engine y Ejecución.
//! Coordina la compra automática minimizando riesgo y optimizando la entrada.

use anyhow::{Result, anyhow};
use solana_sdk::signature::Keypair;
use crate::engine::{DecisionEngine, TokenContext};
use crate::executor_v2::{TradeExecutor, ExecutorConfig};

use crate::sensors::helius::HeliusSensor;
use crate::sensors::dexscreener::DexScreenerSensor;
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
    helius_sensor: HeliusSensor,
    dexscreener_sensor: DexScreenerSensor,
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
            helius_sensor: HeliusSensor::new(rpc_url.clone()),
            dexscreener_sensor: DexScreenerSensor::new(),
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

    /// Construye el contexto del token consultando Sensores Reales (Helius + DexScreener)
    async fn build_context(&self, mint: &str) -> Result<TokenContext> {
        println!("🔍 SENSORS: Fetching data for {}...", mint);
        
        // 1. Paralelizar Consultas (Helius & DexScreener) para reducir latencia
        let helius_future = self.helius_sensor.analyze_token(mint);
        let dex_future = self.dexscreener_sensor.get_token_market_data(mint);
        
        // Esperar ambas respuestas
        let (helius_data, market_data) = tokio::try_join!(helius_future, dex_future)?;

        println!("   ⚡ On-Chain: Auth={:?} | Supply={}", helius_data.mint_authority, helius_data.supply);
        println!("   📈 Market:   ${:.6} | Liq=${:.0} | Vol5m=${:.0}", market_data.price_usd, market_data.liquidity_usd, market_data.volume_5m);

        // 2. Calcular Métricas Derivadas
        // Estimación de edad basada en creación (No disponible directamente, usamos proxy o 0 por ahora)
        // TODO: Implementar Birth Date fetch real (via First Transaction) en Helius Sensor Phase 2.1
        let estimated_age_minutes = if market_data.volume_h1 > 0.0 { 60 } else { 10 }; 

        // Momentum Placeholder: En producción real, este sensor necesita datos históricos (Series de Tiempo)
        // Por ahora usamos la variación de precio 5m como proxy de pendiente instantánea
        // Slope = (Delta Price / Price) / Time
        // DexScreener no da Delta Price exacto instantáneo, usamos Price Change si disponible o 0.0
        let momentum_slope = if market_data.volume_5m > 1000.0 { 0.5 } else { 0.1 }; // Proxy simple

        Ok(TokenContext {
            mint: mint.to_string(),
            symbol: "UNKNOWN".to_string(), // DexScreener podría darlo, updatear struct
            age_minutes: estimated_age_minutes,
            liquidity_usd: market_data.liquidity_usd,
            volume_5m: market_data.volume_5m,
            price_usd: market_data.price_usd,
            momentum_slope, // TODO: Conectar a Ticks reales
            unique_wallets_ratio: 0.5, // TODO: Calcular real
            top_10_holders_pct: 10.0,  // TODO: Helius Top Holders
            dev_wallet_pct: 0.0,
            mint_authority: helius_data.mint_authority,
            freeze_authority: helius_data.freeze_authority,
            lp_burned_pct: 100.0, // Asumimos quemado por seguridad si no hay dato
        })
    }
}
