//! # Decision Engine (Core HFT Logic)
//!
//! Este módulo orquesta la toma de decisiones basada en Sensores, Filtros y Actuadores.
//! Sigue el patrón "Safety-Critical Pipeline".

pub mod actuators;
pub mod filters;
pub mod momentum;
pub mod types;

// Re-exportar tipos para uso externo (AutoBuyer)
pub use self::actuators::{AdaptiveSlippageCalculator, DynamicTipCalculator};
pub use self::types::{FilterResult, RejectionReason, TokenContext, TradeFilter}; // ✅ Exportación Explicita

use self::filters::{
    AuthorityFilter, CircuitBreaker, MomentumFilter, TokenCooldown, WashTradingFilter,
};

use intelligence_rs::strategy_engine::{MarketData, Strategy, TradeAction, SellReason};
use chrono::Utc;

/// Decisión final del Engine unificando Estrategia, Filtros y Actuadores
#[derive(Debug)]
pub enum EngineDecision {
    /// La estrategia dio señal de compra y pasó todos los filtros
    ExecuteBuy(ExecutionParams, f64 /* confidence */, Option<f64> /* target */, f64 /* sl */),
    /// La estrategia dio señal de venta
    ExecuteSell(SellReason, u8),
    /// Rechazado por los filtros de seguridad
    RejectedByFilter(RejectionReason),
    /// Ninguna estrategia dio señal (Hold/Wait)
    Hold,
}

/// El Cerebro del Sistema
/// Coordina la evaluación de riesgos y la ejecución óptima
pub struct DecisionEngine {
    filters: Vec<Box<dyn TradeFilter + Send + Sync>>,
    strategies: Vec<Box<dyn Strategy + Send + Sync>>,
    tip_calculator: DynamicTipCalculator,
    slippage_calculator: AdaptiveSlippageCalculator,
}

impl DecisionEngine {
    /// Crea un nuevo motor con configuración estándar de seguridad
    pub fn new() -> Self {
        let mut engine = Self {
            filters: Vec::new(),
            strategies: Vec::new(),
            tip_calculator: DynamicTipCalculator::new(),
            slippage_calculator: AdaptiveSlippageCalculator::new(),
        };

        // Cargar filtros de seguridad básicos
        engine.add_filter(Box::new(CircuitBreaker::new(-10.0)));
        engine.add_filter(Box::new(TokenCooldown::new(240)));
        engine.add_filter(Box::new(AuthorityFilter));
        engine.add_filter(Box::new(WashTradingFilter::new(0.20)));
        engine.add_filter(Box::new(MomentumFilter::new(0.0)));

        engine
    }

    /// Añade un filtro personalizado al pipeline
    pub fn add_filter(&mut self, filter: Box<dyn TradeFilter + Send + Sync>) {
        self.filters.push(filter);
    }

    /// Añade una estrategia al pipeline de inteligencia
    pub fn add_strategy(&mut self, strategy: Box<dyn Strategy + Send + Sync>) {
        self.strategies.push(strategy);
    }

    /// (Legacy) Evalúa una oportunidad de trading saltándose las estrategias
    pub fn evaluate(&self, ctx: &TokenContext) -> Result<ExecutionParams, RejectionReason> {
        // 1. Ejecutar Pipeline de Filtros (Fail-Fast)
        for filter in &self.filters {
            match filter.check(ctx) {
                FilterResult::Approved => continue,
                FilterResult::Rejected(reason) => return Err(reason),
            }
        }

        // 2. Si pasa todos los filtros, calcular parámetros de ejecución
        let maturity = crate::engine::types::MaturityStage::from_age_minutes(ctx.age_minutes);

        let priority_fee = self
            .tip_calculator
            .calculate_tip(ctx.momentum_slope, maturity);
        let slippage_bps = self
            .slippage_calculator
            .calculate_slippage(ctx.momentum_slope, maturity);

        Ok(ExecutionParams {
            priority_fee_lamports: priority_fee,
            slippage_bps,
            maturity_stage: maturity,
        })
    }

    /// (Nuevo) Evalúa el contexto completo: Estrategias -> Filtros -> Actuadores
    pub fn evaluate_with_strategy(&mut self, ctx: &TokenContext) -> EngineDecision {
        // 1. Convertir TokenContext a MarketData para las Estrategias
        let market_data = MarketData {
            timestamp_ms: Utc::now().timestamp_millis() as u64,
            price: ctx.price_usd,
            volume_24h: ctx.volume_5m * 288.0, // Estimación simple
            liquidity: ctx.liquidity_usd,
        };

        // 2. Consultar al Strategy Engine
        let mut final_action = TradeAction::Hold;
        for strategy in &mut self.strategies {
            if let Ok(action) = strategy.on_price_update(&market_data) {
                if action != TradeAction::Hold {
                    final_action = action;
                    break; // Tomamos la primera estrategia que dé una señal
                }
            }
        }

        // 3. Procesar la acción propuesta
        match final_action {
            TradeAction::Hold => EngineDecision::Hold,
            TradeAction::Sell { reason, amount_percent } => {
                EngineDecision::ExecuteSell(reason, amount_percent)
            }
            TradeAction::Buy { confidence, target_price, stop_loss } => {
                // 4. Pasar por el túnel de seguridad (Filtros)
                for filter in &self.filters {
                    if let FilterResult::Rejected(reason) = filter.check(ctx) {
                        return EngineDecision::RejectedByFilter(reason);
                    }
                }

                // 5. Calcular Actuadores On-Chain
                let maturity = crate::engine::types::MaturityStage::from_age_minutes(ctx.age_minutes);
                let priority_fee = self
                    .tip_calculator
                    .calculate_tip(ctx.momentum_slope, maturity);
                let slippage_bps = self
                    .slippage_calculator
                    .calculate_slippage(ctx.momentum_slope, maturity);

                let params = ExecutionParams {
                    priority_fee_lamports: priority_fee,
                    slippage_bps,
                    maturity_stage: maturity,
                };

                EngineDecision::ExecuteBuy(params, confidence, target_price, stop_loss)
            }
        }
    }
}

/// Parámetros calculados para la ejecución
#[derive(Debug)]
pub struct ExecutionParams {
    pub priority_fee_lamports: u64,
    pub slippage_bps: u16,
    pub maturity_stage: crate::engine::types::MaturityStage,
}
