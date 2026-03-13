//! # Polymarket Bot — The Prediction Engine
//!
//! Motor de trading automático para mercados de predicción en Polymarket.
//! Arquitectura gRPC inspirada en The Chassis (SOL_BOT), adaptada para
//! el dominio de probabilidades y tokens YES/NO.
//!
//! ## Módulos
//!
//! - **config**: Configuración del bot (API, gRPC, riesgo).
//! - **types**: Tipos de dominio (Market, Position, Order, Outcome).
//! - **client**: Cliente HTTP/WebSocket para Polymarket API.
//! - **strategy**: Trait `PredictionStrategy` y estrategias de ejemplo.
//! - **risk**: Gestión de riesgo específica para mercados de predicción.
//! - **generated**: Código generado por protobuf (gRPC service).

pub mod config;
pub mod types;
pub mod client;
pub mod strategy;
pub mod risk;
pub mod generated;

// Re-exportar tipos principales para uso externo
pub use config::PolymarketConfig;
pub use types::{PredictionMarket, PositionInfo, Order, OutcomeType, SideType, MarketStatusType};
pub use strategy::{PredictionStrategy, PredictionAction, MarketSnapshot, EdgeDetectorStrategy};
pub use risk::{RiskManager, RiskDecision};
