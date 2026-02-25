//! # Filtros de Seguridad HFT
//!
//! Implementación de los módulos de seguridad para el Decision Engine.
//! Cada struc implementa el trait `TradeFilter`.

use crate::engine::types::{FilterResult, RejectionReason, TokenContext, TradeFilter};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// 1. Circuit Breaker Global
/// Detiene todo si el PnL diario supera un límite
pub struct CircuitBreaker {
    max_daily_drawdown: f64,
    current_day_pnl: Arc<Mutex<f64>>,
}

impl CircuitBreaker {
    pub fn new(max_drawdown: f64) -> Self {
        Self {
            max_daily_drawdown: max_drawdown, // Ej: -10.0%
            current_day_pnl: Arc::new(Mutex::new(0.0)),
        }
    }

    pub fn update_pnl(&self, pnl: f64) {
        let mut total = self.current_day_pnl.lock().unwrap();
        *total += pnl;
    }
}

impl TradeFilter for CircuitBreaker {
    fn name(&self) -> &'static str {
        "CircuitBreaker"
    }

    fn check(&self, _ctx: &TokenContext) -> FilterResult {
        let pnl = *self.current_day_pnl.lock().unwrap();
        if pnl <= self.max_daily_drawdown {
            return FilterResult::Rejected(RejectionReason::CircuitBreakerTriggered);
        }
        FilterResult::Approved
    }
}

/// 2. Token Cooldown (Anti-Revenge Trading)
/// Evita re-entrar en un token que nos dio pérdidas recientes
pub struct TokenCooldown {
    cooldown_duration: Duration,
    /// Mapa de Mint -> Momento en que expira el cooldown
    blacklisted_tokens: Arc<Mutex<HashMap<String, Instant>>>,
}

impl TokenCooldown {
    pub fn new(duration_minutes: u64) -> Self {
        Self {
            cooldown_duration: Duration::from_secs(duration_minutes * 60),
            blacklisted_tokens: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Añade un token a la lista negra temporalmente
    pub fn blacklist(&self, mint: &str) {
        let expiry = Instant::now() + self.cooldown_duration;
        let mut map = self.blacklisted_tokens.lock().unwrap();
        map.insert(mint.to_string(), expiry);
    }
}

impl TradeFilter for TokenCooldown {
    fn name(&self) -> &'static str {
        "TokenCooldown"
    }

    fn check(&self, ctx: &TokenContext) -> FilterResult {
        let mut map = self.blacklisted_tokens.lock().unwrap();

        // Limpiar expirados
        let now = Instant::now();
        map.retain(|_, expiry| *expiry > now);

        if map.contains_key(&ctx.mint) {
            return FilterResult::Rejected(RejectionReason::TokenCooldownActive);
        }
        FilterResult::Approved
    }
}

/// 3. Authority Filter (Mint & Freeze)
/// Verifica que las autoridades críticas estén revocadas (null)
pub struct AuthorityFilter;

impl TradeFilter for AuthorityFilter {
    fn name(&self) -> &'static str {
        "AuthorityCheck"
    }

    fn check(&self, ctx: &TokenContext) -> FilterResult {
        if ctx.mint_authority.is_some() || ctx.freeze_authority.is_some() {
            return FilterResult::Rejected(RejectionReason::AuthoritiesNotRevoked);
        }
        FilterResult::Approved
    }
}

/// 4. Wash Trading Filter
/// Detecta volumen falso usando ratio de wallets únicas
pub struct WashTradingFilter {
    min_unique_ratio: f64, // Ej: 0.15 (150 wallets unicas por 1000 tx)
}

impl WashTradingFilter {
    pub fn new(min_ratio: f64) -> Self {
        Self {
            min_unique_ratio: min_ratio,
        }
    }
}

impl TradeFilter for WashTradingFilter {
    fn name(&self) -> &'static str {
        "WashTrading"
    }

    fn check(&self, ctx: &TokenContext) -> FilterResult {
        if ctx.unique_wallets_ratio < self.min_unique_ratio {
            return FilterResult::Rejected(RejectionReason::WashTradingDetected);
        }
        FilterResult::Approved
    }
}

/// 5. Momentum Filter (Slope Check)
/// Verifica que la tendencia sea positiva y suficiente
pub struct MomentumFilter {
    min_slope: f64,
}

impl MomentumFilter {
    pub fn new(min_slope: f64) -> Self {
        Self { min_slope }
    }
}

impl TradeFilter for MomentumFilter {
    fn name(&self) -> &'static str {
        "MomentumCheck"
    }

    fn check(&self, ctx: &TokenContext) -> FilterResult {
        if ctx.momentum_slope < 0.0 {
            return FilterResult::Rejected(RejectionReason::NegativeSlope);
        }
        if ctx.momentum_slope < self.min_slope {
            return FilterResult::Rejected(RejectionReason::LowMomentum);
        }
        FilterResult::Approved
    }
}
