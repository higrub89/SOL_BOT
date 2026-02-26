//! # Tipos y Contratos del Engine
//!
//! Define el lenguaje común que usan los sensores y los filtros para comunicarse.
//! Este archivo es el "Manual de Protocolo" del sistema.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Etapa de madurez de un Token (Lifecycle)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MaturityStage {
    /// 0-15 min: Zona de Alto Riesgo (Solo snipers expertos)
    EarlyHighRisk,
    /// 15-45 min: Zona de Momentum (Core Strategy)
    MomentumCore,
    /// >45 min: Zona de Reversal / Consolidación
    LateReversal,
}

impl MaturityStage {
    /// Determina la etapa según la edad en minutos
    pub fn from_age_minutes(minutes: u64) -> Self {
        if minutes < 15 {
            MaturityStage::EarlyHighRisk
        } else if minutes < 45 {
            MaturityStage::MomentumCore
        } else {
            MaturityStage::LateReversal
        }
    }
}

/// Razones por las que el Engine rechaza una operación
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RejectionReason {
    /// Token demasiado joven sin suficientes garantías (Burn/Lock)
    TooEarlyNoBurn,
    /// Autoridades (Mint/Freeze) no revocadas
    AuthoritiesNotRevoked,
    /// Liquidez no bloqueada o quemada (<95%)
    LpNotBurned,
    /// Concentración excesiva en Top 10 Holders (>20%)
    HighConcentration,
    /// Momentum (Slope de Precio) insuficiente (<0.20/min)
    LowMomentum,
    /// Tendencia bajista detectada (Slope negativa)
    NegativeSlope,
    /// Wash Trading detectado (Unique Wallets ratio bajo)
    WashTradingDetected,
    /// Narrativa débil o inexistente (Socials vacíos)
    NarrativeWeak,
    /// Circuit Breaker activado (PnL diario negativo límite)
    CircuitBreakerTriggered,
    /// Token en Cooldown (Revenge Trading prevention)
    TokenCooldownActive,
    /// Exposición excesiva a la misma narrativa (>20%)
    NarrativeExposureLimit,
    /// Congestión de red extrema (Slot lag alto)
    NetworkCongestion,
}

impl fmt::Display for RejectionReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// Resultado de evaluar un filtro
#[derive(Debug, Clone)]
pub enum FilterResult {
    /// Aprobado (Pasa al siguiente filtro)
    Approved,
    /// Rechazado (Abortar operación)
    Rejected(RejectionReason),
}

/// Contexto completo de un Token para evaluación
#[derive(Debug, Clone)]
pub struct TokenContext {
    pub mint: String,
    pub symbol: String,
    pub age_minutes: u64,
    pub liquidity_usd: f64,
    pub volume_5m: f64,
    pub price_usd: f64,
    pub momentum_slope: f64,       // Calculado por MomentumSensor
    pub unique_wallets_ratio: f64, // Calculado por WashTradingSensor
    pub top_10_holders_pct: f64,
    pub dev_wallet_pct: f64,
    pub mint_authority: Option<String>,
    pub freeze_authority: Option<String>,
    pub lp_burned_pct: f64,
}

impl TokenContext {
    /// Helper para verificar si las autoridades están revocadas (None)
    pub fn authorities_revoked(&self) -> bool {
        self.mint_authority.is_none() && self.freeze_authority.is_none()
    }
}

/// Trait que todo filtro debe implementar
pub trait TradeFilter: Send + Sync {
    /// Nombre del filtro para logs
    fn name(&self) -> &'static str;

    /// Ejecuta la lógica del filtro
    fn check(&self, ctx: &TokenContext) -> FilterResult;
}
