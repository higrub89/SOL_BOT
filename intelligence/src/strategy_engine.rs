//! # Strategy Engine
//! 
//! El cerebro modular del bot. Define la interfaz estándar para todas las estrategias de trading.
//! Permite backtesting seguro y ejecución en tiempo real con la misma lógica.

use anyhow::Result;
use chrono::{DateTime, Utc};
use std::fmt::Debug;

/// Representa una acción de trading sugerida por una estrategia
#[derive(Debug, Clone, PartialEq)]
pub enum TradeAction {
    /// Comprar (Entry Long)
    Buy {
        /// Confianza de la señal (0.0 - 1.0)
        confidence: f64,
        /// Precio objetivo (Take Profit)
        target_price: Option<f64>,
        /// Stop Loss inicial recomendado
        stop_loss: f64,
    },
    /// Vender (Exit Long)
    Sell {
        /// Razón de la venta (Take Profit, Stop Loss, Signal Reversal)
        reason: String,
        /// Porcentaje a vender (0-100)
        amount_percent: u8,
    },
    /// Mantener posición (Wait)
    Hold,
}

/// Datos de mercado estandarizados para alimentar las estrategias
#[derive(Debug, Clone)]
pub struct MarketData {
    pub timestamp: DateTime<Utc>,
    pub price: f64,
    pub volume_24h: f64,
    pub liquidity: f64,
    // Puedes añadir más campos aquí (RSI, MA, sentiment, etc.)
}

/// El Trait Sagrado que todas las estrategias deben implementar
pub trait Strategy: Debug + Send + Sync {
    /// Nombre de la estrategia (para logs y reportes)
    fn name(&self) -> &str;
    
    /// Inicializa la estrategia (carga modelos, configura indicadores)
    fn initialize(&mut self) -> Result<()>;
    
    /// Procesa una actualización de precio y devuelve una decisión
    fn on_price_update(&mut self, data: &MarketData) -> Result<TradeAction>;
    
    /// Opcional: Procesa eventos arbitrarios (noticias, tweets)
    fn on_event(&mut self, _event_type: &str, _payload: &str) -> Result<TradeAction> {
        Ok(TradeAction::Hold)
    }
}

// ----------------------------------------------------------------------------
// EJEMPLO DE ESTRATEGIA: SIMPLE MOMENTUM
// ----------------------------------------------------------------------------

#[derive(Debug)]
pub struct SimpleMomentumStrategy {
    symbol: String,
    last_price: Option<f64>,
    momentum_threshold: f64,
}

impl SimpleMomentumStrategy {
    pub fn new(symbol: String, threshold: f64) -> Self {
        Self {
            symbol,
            last_price: None,
            momentum_threshold: threshold,
        }
    }
}

impl Strategy for SimpleMomentumStrategy {
    fn name(&self) -> &str {
        "SimpleMomentum"
    }

    fn initialize(&mut self) -> Result<()> {
        println!("🚀 Estrategia SimpleMomentum inicializada para {}", self.symbol);
        Ok(())
    }

    fn on_price_update(&mut self, data: &MarketData) -> Result<TradeAction> {
        let current_price = data.price;
        
        let action = if let Some(last) = self.last_price {
            let change_pct = (current_price - last) / last * 100.0;
            
            if change_pct > self.momentum_threshold {
                TradeAction::Buy {
                    confidence: 0.8,
                    target_price: Some(current_price * 1.05), // TP +5%
                    stop_loss: current_price * 0.98,          // SL -2%
                }
            } else if change_pct < -self.momentum_threshold {
                TradeAction::Sell {
                    reason: "Momentum Loss".to_string(),
                    amount_percent: 100,
                }
            } else {
                TradeAction::Hold
            }
        } else {
            TradeAction::Hold
        };
        
        self.last_price = Some(current_price);
        Ok(action)
    }
}
