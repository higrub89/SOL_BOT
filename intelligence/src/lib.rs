//! # Intelligence Engine - The Brain
//!
//! Biblioteca de estrategias, backtesting y modelos de ML para el bot de trading.
//!
//! Módulos principales:
//! - strategy_engine: Define la interfaz `Strategy` y estrategias comunes.
//! - backtesting: Simulador de mercado para validar estrategias.
//! - ml_bridge: (Futuro) Conexión con modelos Python vía FFI/IPC.

pub mod backtesting;
pub mod ml_bridge;
pub mod strategy_engine;

// Re-exportar tipos comunes para facilitar uso
pub use backtesting::{BacktestResult, MarketSimulator};
pub use ml_bridge::{MlBridge, Signal};
pub use strategy_engine::{MarketData, Strategy, TradeAction};
