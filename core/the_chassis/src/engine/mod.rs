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

/// El Cerebro del Sistema
/// Coordina la evaluación de riesgos y la ejecución óptima
pub struct DecisionEngine {
    filters: Vec<Box<dyn TradeFilter + Send + Sync>>,
    tip_calculator: DynamicTipCalculator,
    slippage_calculator: AdaptiveSlippageCalculator,
}

impl DecisionEngine {
    /// Crea un nuevo motor con configuración estándar de seguridad
    pub fn new() -> Self {
        let mut engine = Self {
            filters: Vec::new(),
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

    /// Evalúa una oportunidad de trading
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
}

/// Parámetros calculados para la ejecución
#[derive(Debug)]
pub struct ExecutionParams {
    pub priority_fee_lamports: u64,
    pub slippage_bps: u16,
    pub maturity_stage: crate::engine::types::MaturityStage,
}
