//! # Tipos de Dominio — Polymarket
//!
//! Estructuras internas que representan los conceptos fundamentales
//! de un mercado de predicción: mercados, posiciones, órdenes y resultados.

use serde::{Deserialize, Serialize};

/// Representa un mercado de predicción en Polymarket
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictionMarket {
    /// ID único del mercado
    pub market_id: String,
    /// Pregunta del evento (ej: "¿Ganará X las elecciones?")
    pub question: String,
    /// Descripción extendida del evento
    pub description: String,
    /// Probabilidad implícita del YES (0.0 - 1.0)
    pub yes_price: f64,
    /// Probabilidad implícita del NO (0.0 - 1.0)
    pub no_price: f64,
    /// Volumen total negociado (USDC)
    pub volume: f64,
    /// Liquidez disponible (USDC)
    pub liquidity: f64,
    /// Fecha de cierre (Unix timestamp ms)
    pub end_date_ms: i64,
    /// Estado actual
    pub status: MarketStatusType,
    /// Categoría del evento
    pub category: String,
}

/// Estado de un mercado de predicción
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MarketStatusType {
    /// Mercado abierto y aceptando órdenes
    Active,
    /// Mercado cerrado, pendiente de resolución
    Closed,
    /// Mercado resuelto con resultado final
    Resolved,
}

/// Resultado de un mercado binario (Sí/No)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OutcomeType {
    Yes,
    No,
}

/// Dirección de una operación
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SideType {
    Buy,
    Sell,
}

/// Representa una orden en un mercado de predicción
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    /// ID de la orden
    pub order_id: String,
    /// ID del mercado
    pub market_id: String,
    /// Outcome (YES/NO)
    pub outcome: OutcomeType,
    /// Dirección (Buy/Sell)
    pub side: SideType,
    /// Cantidad de shares
    pub amount: f64,
    /// Precio límite (probabilidad)
    pub limit_price: f64,
    /// Estado de la orden
    pub status: OrderStatusType,
    /// Timestamp de creación (Unix ms)
    pub created_at_ms: i64,
}

/// Estado de una orden
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderStatusType {
    Pending,
    Filled,
    PartiallyFilled,
    Cancelled,
    Rejected,
}

/// Representa una posición abierta en un mercado
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionInfo {
    /// ID del mercado
    pub market_id: String,
    /// Pregunta del mercado
    pub question: String,
    /// Outcome que se tiene (YES o NO)
    pub outcome: OutcomeType,
    /// Cantidad de shares
    pub shares: f64,
    /// Precio medio de entrada
    pub avg_entry_price: f64,
    /// Precio actual
    pub current_price: f64,
    /// PnL en porcentaje
    pub pnl_percent: f64,
    /// PnL en USDC
    pub pnl_usdc: f64,
}

impl PositionInfo {
    /// Calcula el PnL actualizado con un nuevo precio
    pub fn update_pnl(&mut self, new_price: f64) {
        self.current_price = new_price;
        if self.avg_entry_price > 0.0 {
            self.pnl_percent = ((new_price - self.avg_entry_price) / self.avg_entry_price) * 100.0;
            self.pnl_usdc = (new_price - self.avg_entry_price) * self.shares;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_pnl_calculation() {
        let mut pos = PositionInfo {
            market_id: "market-1".to_string(),
            question: "Test market".to_string(),
            outcome: OutcomeType::Yes,
            shares: 100.0,
            avg_entry_price: 0.50,
            current_price: 0.50,
            pnl_percent: 0.0,
            pnl_usdc: 0.0,
        };

        // Precio sube a 0.70 → +40% PnL
        pos.update_pnl(0.70);
        assert!((pos.pnl_percent - 40.0).abs() < 0.01);
        assert!((pos.pnl_usdc - 20.0).abs() < 0.01);
    }

    #[test]
    fn test_position_pnl_negative() {
        let mut pos = PositionInfo {
            market_id: "market-2".to_string(),
            question: "Another market".to_string(),
            outcome: OutcomeType::No,
            shares: 50.0,
            avg_entry_price: 0.60,
            current_price: 0.60,
            pnl_percent: 0.0,
            pnl_usdc: 0.0,
        };

        // Precio baja a 0.30 → -50% PnL
        pos.update_pnl(0.30);
        assert!((pos.pnl_percent - (-50.0)).abs() < 0.01);
        assert!((pos.pnl_usdc - (-15.0)).abs() < 0.01);
    }

    #[test]
    fn test_market_status_equality() {
        assert_eq!(MarketStatusType::Active, MarketStatusType::Active);
        assert_ne!(MarketStatusType::Active, MarketStatusType::Closed);
    }
}
