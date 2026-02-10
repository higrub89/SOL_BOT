//! # Intelligence Engine - The Brain
//! 
//! Biblioteca de estrategias, backtesting y modelos de ML para el bot de trading.
//! 
//! Módulos principales:
//! - strategy_engine: Define la interfaz `Strategy` y estrategias comunes.
//! - backtesting: Simulador de mercado para validar estrategias.
//! - ml_bridge: (Futuro) Conexión con modelos Python vía FFI/IPC.

pub mod strategy_engine;
pub mod backtesting;

// Re-exportar tipos comunes para facilitar uso
pub use strategy_engine::{Strategy, MarketData, TradeAction};
pub use backtesting::{MarketSimulator, BacktestResult};
