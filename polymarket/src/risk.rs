//! # Gestión de Riesgo — Polymarket
//!
//! Módulo de control de riesgo específico para mercados de predicción.
//! Gestiona los riesgos únicos de este tipo de activos:
//! - Riesgo de oráculo/resolución
//! - Deadlines de cierre
//! - Concentración de portfolio
//! - Límites de pérdida

use crate::config::RiskConfig;
use crate::types::{PositionInfo, MarketStatusType, PredictionMarket};

/// Resultado de una evaluación de riesgo
#[derive(Debug, Clone, PartialEq)]
pub enum RiskDecision {
    /// Operación permitida
    Approved,
    /// Operación rechazada con razón
    Rejected(String),
    /// Operación permitida pero con tamaño reducido
    ReducedSize {
        max_allowed_usdc: f64,
        reason: String,
    },
}

/// Motor de gestión de riesgo
#[derive(Debug)]
pub struct RiskManager {
    config: RiskConfig,
}

impl RiskManager {
    pub fn new(config: RiskConfig) -> Self {
        Self { config }
    }

    /// Evalúa si una nueva operación está dentro de los límites de riesgo
    pub fn evaluate_order(
        &self,
        market: &PredictionMarket,
        requested_size_usdc: f64,
        current_positions: &[PositionInfo],
    ) -> RiskDecision {
        // 1. Verificar que el mercado está activo
        if market.status != MarketStatusType::Active {
            return RiskDecision::Rejected(
                "El mercado no está activo".to_string()
            );
        }

        // 2. Verificar número máximo de posiciones abiertas
        let open_positions = current_positions.len() as u32;
        if open_positions >= self.config.max_open_positions {
            return RiskDecision::Rejected(format!(
                "Límite de posiciones alcanzado: {}/{}",
                open_positions, self.config.max_open_positions
            ));
        }

        // 3. Verificar tamaño máximo por posición
        if requested_size_usdc > self.config.max_position_usdc {
            return RiskDecision::ReducedSize {
                max_allowed_usdc: self.config.max_position_usdc,
                reason: format!(
                    "Tamaño reducido de {:.2} a {:.2} USDC (límite por posición)",
                    requested_size_usdc, self.config.max_position_usdc
                ),
            };
        }

        // 4. Verificar exposición total al mismo mercado
        let existing_exposure: f64 = current_positions.iter()
            .filter(|p| p.market_id == market.market_id)
            .map(|p| p.shares * p.avg_entry_price)
            .sum();

        let total_exposure = existing_exposure + requested_size_usdc;
        if total_exposure > self.config.max_position_usdc {
            let remaining = self.config.max_position_usdc - existing_exposure;
            if remaining <= 0.0 {
                return RiskDecision::Rejected(format!(
                    "Exposición máxima al mercado {} ya alcanzada ({:.2} USDC)",
                    market.market_id, existing_exposure
                ));
            }
            return RiskDecision::ReducedSize {
                max_allowed_usdc: remaining,
                reason: format!(
                    "Tamaño reducido a {:.2} USDC para respetar límite de exposición al mercado",
                    remaining
                ),
            };
        }

        // 5. Verificar liquidez mínima del mercado
        if market.liquidity < requested_size_usdc * 2.0 {
            return RiskDecision::Rejected(format!(
                "Liquidez insuficiente: {:.2} USDC (mínimo requerido: {:.2})",
                market.liquidity, requested_size_usdc * 2.0
            ));
        }

        RiskDecision::Approved
    }

    /// Verifica si alguna posición requiere stop-loss
    pub fn check_stop_losses(&self, positions: &[PositionInfo]) -> Vec<(String, f64)> {
        positions.iter()
            .filter(|p| p.pnl_percent < -self.config.default_stop_loss_pct)
            .map(|p| (p.market_id.clone(), p.pnl_percent))
            .collect()
    }

    /// Calcula la exposición total del portfolio
    pub fn total_exposure(&self, positions: &[PositionInfo]) -> f64 {
        positions.iter()
            .map(|p| p.shares * p.avg_entry_price)
            .sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::OutcomeType;

    fn default_risk_config() -> RiskConfig {
        RiskConfig {
            max_position_usdc: 100.0,
            max_open_positions: 5,
            default_stop_loss_pct: 30.0,
            min_edge_threshold: 0.05,
        }
    }

    fn make_market(id: &str, status: MarketStatusType, liquidity: f64) -> PredictionMarket {
        PredictionMarket {
            market_id: id.to_string(),
            question: "Test".to_string(),
            description: "Test".to_string(),
            yes_price: 0.50,
            no_price: 0.50,
            volume: 10000.0,
            liquidity,
            end_date_ms: 0,
            status,
            category: "Test".to_string(),
        }
    }

    fn make_position(market_id: &str, shares: f64, entry: f64, pnl: f64) -> PositionInfo {
        PositionInfo {
            market_id: market_id.to_string(),
            question: "Test".to_string(),
            outcome: OutcomeType::Yes,
            shares,
            avg_entry_price: entry,
            current_price: entry,
            pnl_percent: pnl,
            pnl_usdc: 0.0,
        }
    }

    #[test]
    fn test_approved_order() {
        let rm = RiskManager::new(default_risk_config());
        let market = make_market("m1", MarketStatusType::Active, 1000.0);
        let decision = rm.evaluate_order(&market, 50.0, &[]);
        assert_eq!(decision, RiskDecision::Approved);
    }

    #[test]
    fn test_rejected_inactive_market() {
        let rm = RiskManager::new(default_risk_config());
        let market = make_market("m1", MarketStatusType::Closed, 1000.0);
        let decision = rm.evaluate_order(&market, 50.0, &[]);
        assert!(matches!(decision, RiskDecision::Rejected(_)));
    }

    #[test]
    fn test_rejected_max_positions() {
        let rm = RiskManager::new(default_risk_config());
        let market = make_market("m6", MarketStatusType::Active, 1000.0);
        let positions: Vec<PositionInfo> = (0..5)
            .map(|i| make_position(&format!("m{}", i), 10.0, 0.5, 0.0))
            .collect();
        let decision = rm.evaluate_order(&market, 50.0, &positions);
        assert!(matches!(decision, RiskDecision::Rejected(_)));
    }

    #[test]
    fn test_reduced_size_over_limit() {
        let rm = RiskManager::new(default_risk_config());
        let market = make_market("m1", MarketStatusType::Active, 1000.0);
        let decision = rm.evaluate_order(&market, 150.0, &[]);
        match decision {
            RiskDecision::ReducedSize { max_allowed_usdc, .. } => {
                assert!((max_allowed_usdc - 100.0).abs() < f64::EPSILON);
            },
            _ => panic!("Expected ReducedSize"),
        }
    }

    #[test]
    fn test_rejected_low_liquidity() {
        let rm = RiskManager::new(default_risk_config());
        let market = make_market("m1", MarketStatusType::Active, 50.0);
        let decision = rm.evaluate_order(&market, 50.0, &[]);
        assert!(matches!(decision, RiskDecision::Rejected(_)));
    }

    #[test]
    fn test_stop_loss_detection() {
        let rm = RiskManager::new(default_risk_config());
        let positions = vec![
            make_position("m1", 10.0, 0.5, -35.0), // SL triggered
            make_position("m2", 10.0, 0.5, -10.0), // OK
            make_position("m3", 10.0, 0.5, -50.0), // SL triggered
        ];
        let triggered = rm.check_stop_losses(&positions);
        assert_eq!(triggered.len(), 2);
    }

    #[test]
    fn test_total_exposure() {
        let rm = RiskManager::new(default_risk_config());
        let positions = vec![
            make_position("m1", 100.0, 0.50, 0.0),
            make_position("m2", 200.0, 0.30, 0.0),
        ];
        let exposure = rm.total_exposure(&positions);
        assert!((exposure - 110.0).abs() < f64::EPSILON); // 100*0.5 + 200*0.3 = 110
    }
}
