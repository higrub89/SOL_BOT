//! # Strategy Engine
//! 
//! El cerebro modular del bot. Define la interfaz estándar para todas las estrategias de trading.
//! Permite backtesting seguro y ejecución en tiempo real con la misma lógica.

use anyhow::Result;

use std::fmt::Debug;
use std::sync::RwLock;

/// Razones estandarizadas y libres de heap para una venta
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SellReason {
    TakeProfit,
    StopLoss,
    MomentumLoss,
    SignalReversal,
    Emergency,
}

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
        reason: SellReason,
        /// Porcentaje a vender (0-100)
        amount_percent: u8,
    },
    /// Mantener posición (Wait)
    Hold,
}

/// Datos de mercado estandarizados para alimentar las estrategias
#[derive(Debug, Clone)]
pub struct MarketData {
    pub timestamp_ms: u64,
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
    fn initialize(&self) -> Result<()>;
    
    /// Procesa una actualización de precio y devuelve una decisión
    fn on_price_update(&self, data: &MarketData) -> Result<TradeAction>;
    
    /// Opcional: Procesa eventos arbitrarios (noticias, tweets)
    fn on_event(&self, _event_type: &str, _payload: &str) -> Result<TradeAction> {
        Ok(TradeAction::Hold)
    }
}

// ----------------------------------------------------------------------------
// EJEMPLO DE ESTRATEGIA: SIMPLE MOMENTUM
// ----------------------------------------------------------------------------

#[derive(Debug)]
pub struct SimpleMomentumStrategy {
    symbol: String,
    last_price: RwLock<Option<f64>>,
    momentum_threshold: f64,
}

impl SimpleMomentumStrategy {
    pub fn new(symbol: String, threshold: f64) -> Self {
        Self {
            symbol,
            last_price: RwLock::new(None),
            momentum_threshold: threshold,
        }
    }
}

impl Strategy for SimpleMomentumStrategy {
    fn name(&self) -> &str {
        "SimpleMomentum"
    }

    fn initialize(&self) -> Result<()> {
        println!("🚀 Estrategia SimpleMomentum inicializada para {}", self.symbol);
        Ok(())
    }

    fn on_price_update(&self, data: &MarketData) -> Result<TradeAction> {
        let current_price = data.price;
        
        let mut last_price_guard = self.last_price.write().unwrap();
        
        let action = if let Some(last) = *last_price_guard {
            let change_pct = (current_price - last) / last * 100.0;
            
            if change_pct > self.momentum_threshold {
                TradeAction::Buy {
                    confidence: 0.8,
                    target_price: Some(current_price * 1.05), // TP +5%
                    stop_loss: current_price * 0.98,          // SL -2%
                }
            } else if change_pct < -self.momentum_threshold {
                TradeAction::Sell {
                    reason: SellReason::MomentumLoss,
                    amount_percent: 100,
                }
            } else {
                TradeAction::Hold
            }
        } else {
            TradeAction::Hold
        };
        
        *last_price_guard = Some(current_price);
        Ok(action)
    }
}
