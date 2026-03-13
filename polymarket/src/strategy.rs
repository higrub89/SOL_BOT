//! # Motor de Estrategias — Polymarket
//!
//! Define el trait `PredictionStrategy` y estrategias de ejemplo para mercados de predicción.
//! Adaptado del patrón de Strategy de intelligence_rs para el dominio de probabilidades.

use anyhow::Result;
use std::fmt::Debug;

use crate::types::{PredictionMarket, OutcomeType};

/// Razón de una operación sugerida
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TradeReason {
    /// Edge positivo detectado (probabilidad interna > precio mercado)
    PositiveEdge,
    /// Market making (proveer liquidez)
    MarketMaking,
    /// Arbitraje entre fuentes de probabilidad
    Arbitrage,
    /// Rebalanceo de portfolio
    Rebalance,
    /// Stop-loss activado
    StopLoss,
    /// Take-profit alcanzado
    TakeProfit,
    /// Mercado a punto de cerrar
    MarketClosing,
}

/// Acción sugerida por una estrategia
#[derive(Debug, Clone, PartialEq)]
pub enum PredictionAction {
    /// Comprar shares de un outcome
    Buy {
        outcome: OutcomeType,
        /// Confianza en la señal (0.0 - 1.0)
        confidence: f64,
        /// Tamaño sugerido (en USDC)
        suggested_size_usdc: f64,
        /// Precio máximo aceptable
        max_price: f64,
        /// Razón de la operación
        reason: TradeReason,
    },
    /// Vender shares de un outcome
    Sell {
        outcome: OutcomeType,
        /// Porcentaje de la posición a vender (0-100)
        amount_percent: u8,
        /// Razón de la venta
        reason: TradeReason,
    },
    /// No hacer nada
    Hold,
}

/// Datos de mercado enriquecidos para alimentar estrategias
#[derive(Debug, Clone)]
pub struct MarketSnapshot {
    /// Mercado de predicción con datos actuales
    pub market: PredictionMarket,
    /// Historial reciente de precios YES (más reciente al final)
    pub yes_price_history: Vec<f64>,
    /// Volumen en las últimas 24h
    pub volume_24h: f64,
    /// Horas restantes hasta cierre del mercado
    pub hours_until_close: f64,
}

/// El Trait que todas las estrategias de predicción deben implementar
pub trait PredictionStrategy: Debug + Send + Sync {
    /// Nombre de la estrategia
    fn name(&self) -> &str;

    /// Inicializa la estrategia
    fn initialize(&self) -> Result<()>;

    /// Analiza un mercado y decide si actuar
    fn analyze(&self, snapshot: &MarketSnapshot) -> Result<PredictionAction>;

    /// Probabilidad interna estimada para un evento (opcional override)
    fn estimate_probability(&self, _snapshot: &MarketSnapshot) -> Option<f64> {
        None
    }
}

// ============================================================================
// ESTRATEGIA: EDGE DETECTOR (Detecta mispricings)
// ============================================================================

/// Estrategia que compara precio de mercado con probabilidad interna
/// Si detecta un "edge" (ventaja), sugiere operar.
#[derive(Debug)]
pub struct EdgeDetectorStrategy {
    /// Edge mínimo requerido para operar (ej: 0.05 = 5%)
    min_edge: f64,
    /// Tamaño base por operación (USDC)
    base_size_usdc: f64,
}

impl EdgeDetectorStrategy {
    pub fn new(min_edge: f64, base_size_usdc: f64) -> Self {
        assert!(min_edge > 0.0, "min_edge must be > 0.0");
        assert!(base_size_usdc > 0.0, "base_size_usdc must be > 0.0");
        Self {
            min_edge,
            base_size_usdc,
        }
    }

    /// Calcula el edge: diferencia entre probabilidad interna y precio de mercado
    fn calculate_edge(&self, internal_prob: f64, market_price: f64) -> f64 {
        internal_prob - market_price
    }
}

impl PredictionStrategy for EdgeDetectorStrategy {
    fn name(&self) -> &str {
        "EdgeDetector"
    }

    fn initialize(&self) -> Result<()> {
        tracing::info!("🎯 Estrategia EdgeDetector inicializada (min_edge: {:.1}%)", self.min_edge * 100.0);
        Ok(())
    }

    fn analyze(&self, snapshot: &MarketSnapshot) -> Result<PredictionAction> {
        // Usar probabilidad interna si está disponible; si no, usar heurística simple
        let internal_prob = self.estimate_probability(snapshot)
            .unwrap_or(snapshot.market.yes_price);

        let yes_edge = self.calculate_edge(internal_prob, snapshot.market.yes_price);
        let no_edge = self.calculate_edge(1.0 - internal_prob, snapshot.market.no_price);

        // Si el mercado está a punto de cerrar, ser más conservador
        if snapshot.hours_until_close < 1.0 {
            return Ok(PredictionAction::Hold);
        }

        if yes_edge > self.min_edge {
            let confidence = (yes_edge / self.min_edge).min(1.0);
            let size = self.base_size_usdc * confidence;

            Ok(PredictionAction::Buy {
                outcome: OutcomeType::Yes,
                confidence,
                suggested_size_usdc: size,
                max_price: snapshot.market.yes_price + (yes_edge * 0.5),
                reason: TradeReason::PositiveEdge,
            })
        } else if no_edge > self.min_edge {
            let confidence = (no_edge / self.min_edge).min(1.0);
            let size = self.base_size_usdc * confidence;

            Ok(PredictionAction::Buy {
                outcome: OutcomeType::No,
                confidence,
                suggested_size_usdc: size,
                max_price: snapshot.market.no_price + (no_edge * 0.5),
                reason: TradeReason::PositiveEdge,
            })
        } else {
            Ok(PredictionAction::Hold)
        }
    }

    /// Heurística simple: usa el precio del mercado ajustado por momentum
    fn estimate_probability(&self, snapshot: &MarketSnapshot) -> Option<f64> {
        if snapshot.yes_price_history.len() < 2 {
            return None;
        }

        let recent = &snapshot.yes_price_history;
        let len = recent.len();
        let slice = &recent[len.saturating_sub(5)..];
        if slice.is_empty() {
            return None;
        }
        let avg_recent: f64 = slice.iter().sum::<f64>() / slice.len() as f64;

        Some(avg_recent)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{PredictionMarket, MarketStatusType};

    fn make_snapshot(yes_price: f64, history: Vec<f64>) -> MarketSnapshot {
        MarketSnapshot {
            market: PredictionMarket {
                market_id: "test-1".to_string(),
                question: "Test?".to_string(),
                description: "Test market".to_string(),
                yes_price,
                no_price: 1.0 - yes_price,
                volume: 10000.0,
                liquidity: 5000.0,
                end_date_ms: 0,
                status: MarketStatusType::Active,
                category: "Test".to_string(),
            },
            yes_price_history: history,
            volume_24h: 10000.0,
            hours_until_close: 48.0,
        }
    }

    #[test]
    fn test_edge_detector_hold_no_edge() {
        let strategy = EdgeDetectorStrategy::new(0.05, 50.0);
        strategy.initialize().unwrap();

        // Precio 0.50, historial estable → sin edge → Hold
        let snapshot = make_snapshot(0.50, vec![0.50, 0.50, 0.50, 0.50, 0.50]);
        let action = strategy.analyze(&snapshot).unwrap();
        assert_eq!(action, PredictionAction::Hold);
    }

    #[test]
    fn test_edge_detector_buy_yes() {
        let strategy = EdgeDetectorStrategy::new(0.05, 50.0);

        // Historial muestra tendencia alcista pero precio no ha subido aún
        let snapshot = make_snapshot(0.50, vec![0.50, 0.55, 0.58, 0.60, 0.62]);
        let action = strategy.analyze(&snapshot).unwrap();

        match action {
            PredictionAction::Buy { outcome, confidence, reason, .. } => {
                assert_eq!(outcome, OutcomeType::Yes);
                assert!(confidence > 0.0);
                assert_eq!(reason, TradeReason::PositiveEdge);
            },
            _ => panic!("Esperaba acción Buy"),
        }
    }

    #[test]
    fn test_edge_detector_hold_near_close() {
        let strategy = EdgeDetectorStrategy::new(0.05, 50.0);

        // Mercado a punto de cerrar → siempre Hold
        let mut snapshot = make_snapshot(0.50, vec![0.50, 0.60, 0.70, 0.75, 0.80]);
        snapshot.hours_until_close = 0.5;

        let action = strategy.analyze(&snapshot).unwrap();
        assert_eq!(action, PredictionAction::Hold);
    }

    #[test]
    fn test_edge_calculation() {
        let strategy = EdgeDetectorStrategy::new(0.05, 50.0);
        let edge = strategy.calculate_edge(0.70, 0.50);
        assert!((edge - 0.20).abs() < f64::EPSILON);
    }
}
