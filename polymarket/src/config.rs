//! # Configuración — Polymarket Bot
//!
//! Carga y gestión de la configuración del bot.
//! Soporta variables de entorno y archivo settings.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

/// Configuración principal del bot de Polymarket
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolymarketConfig {
    /// Configuración de la API de Polymarket
    pub api: ApiConfig,
    /// Configuración del servidor gRPC
    pub grpc: GrpcConfig,
    /// Configuración de riesgo
    pub risk: RiskConfig,
}

/// Configuración de conexión a la API de Polymarket
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    /// URL base de la API REST de Polymarket
    pub rest_url: String,
    /// URL del WebSocket para streaming
    pub ws_url: String,
    /// API key (si aplica)
    pub api_key: String,
    /// Timeout para requests HTTP (segundos)
    pub timeout_secs: u64,
}

/// Configuración del servidor gRPC
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrpcConfig {
    /// Dirección y puerto del servidor gRPC
    pub listen_addr: String,
}

/// Configuración de gestión de riesgo
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskConfig {
    /// Inversión máxima por mercado (USDC)
    pub max_position_usdc: f64,
    /// Número máximo de posiciones abiertas simultáneas
    pub max_open_positions: u32,
    /// Stop-loss por defecto (porcentaje de pérdida máxima)
    pub default_stop_loss_pct: f64,
    /// Margen mínimo de beneficio para entrar (probabilidad)
    pub min_edge_threshold: f64,
}

impl Default for PolymarketConfig {
    fn default() -> Self {
        Self {
            api: ApiConfig {
                rest_url: "https://clob.polymarket.com".to_string(),
                ws_url: "wss://ws-subscriptions-clob.polymarket.com/ws/market".to_string(),
                api_key: String::new(),
                timeout_secs: 10,
            },
            grpc: GrpcConfig {
                listen_addr: "[::1]:50052".to_string(),
            },
            risk: RiskConfig {
                max_position_usdc: 100.0,
                max_open_positions: 10,
                default_stop_loss_pct: 30.0,
                min_edge_threshold: 0.05,
            },
        }
    }
}

impl PolymarketConfig {
    /// Carga la configuración desde variables de entorno con fallback a defaults
    pub fn from_env() -> Result<Self> {
        dotenv::dotenv().ok();

        let mut config = Self::default();

        if let Ok(url) = std::env::var("POLYMARKET_REST_URL") {
            config.api.rest_url = url;
        }
        if let Ok(url) = std::env::var("POLYMARKET_WS_URL") {
            config.api.ws_url = url;
        }
        if let Ok(key) = std::env::var("POLYMARKET_API_KEY") {
            config.api.api_key = key;
        }
        if let Ok(addr) = std::env::var("POLYMARKET_GRPC_ADDR") {
            config.grpc.listen_addr = addr;
        }
        if let Ok(val) = std::env::var("POLYMARKET_MAX_POSITION_USDC") {
            config.risk.max_position_usdc = val.parse()
                .context("POLYMARKET_MAX_POSITION_USDC debe ser un número válido")?;
        }

        Ok(config)
    }

    /// Carga la configuración desde un archivo JSON
    pub fn from_file(path: &str) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .context(format!("No se pudo leer el archivo de configuración: {}", path))?;
        let config: Self = serde_json::from_str(&content)
            .context("Error parseando configuración JSON")?;
        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = PolymarketConfig::default();
        assert_eq!(config.grpc.listen_addr, "[::1]:50052");
        assert_eq!(config.risk.max_open_positions, 10);
        assert!(config.risk.max_position_usdc > 0.0);
    }

    #[test]
    fn test_config_from_env() {
        // Should not panic even without env vars set
        let config = PolymarketConfig::from_env();
        assert!(config.is_ok());
    }
}
