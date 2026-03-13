//! # Cliente API — Polymarket
//!
//! Cliente HTTP/WebSocket para comunicarse con la API de Polymarket (CLOB).
//! Abstrae las llamadas REST y el streaming de datos del mercado.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use crate::config::ApiConfig;
use crate::types::{PredictionMarket, MarketStatusType, Order, OutcomeType, SideType, OrderStatusType};

/// Cliente para la API REST de Polymarket
pub struct PolymarketClient {
    http: reqwest::Client,
    config: ApiConfig,
}

/// Respuesta de la API de mercados de Polymarket
#[derive(Debug, Deserialize)]
struct ApiMarketResponse {
    #[serde(default)]
    tokens: Vec<ApiToken>,
    #[serde(default)]
    question: String,
    #[serde(default)]
    description: String,
    #[serde(default)]
    condition_id: String,
    #[serde(default)]
    volume: String,
    #[serde(default)]
    liquidity: String,
    #[serde(default)]
    #[allow(dead_code)]
    end_date_iso: String,
    #[serde(default)]
    active: bool,
    #[serde(default)]
    closed: bool,
    #[serde(default)]
    category: String,
}

/// Token individual de un mercado
#[derive(Debug, Deserialize)]
struct ApiToken {
    #[serde(default)]
    outcome: String,
    #[serde(default)]
    price: f64,
}

/// Payload para crear una orden vía API
#[derive(Debug, Serialize)]
struct ApiOrderPayload {
    market_id: String,
    outcome: String,
    side: String,
    size: f64,
    price: f64,
}

impl PolymarketClient {
    /// Crea un nuevo cliente con la configuración proporcionada
    pub fn new(config: &ApiConfig) -> Result<Self> {
        let http = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(config.timeout_secs))
            .build()
            .context("Error creando cliente HTTP")?;

        Ok(Self {
            http,
            config: config.clone(),
        })
    }

    /// Obtiene los mercados disponibles de Polymarket
    pub async fn get_markets(&self, limit: usize) -> Result<Vec<PredictionMarket>> {
        let url = format!("{}/markets?limit={}", self.config.rest_url, limit);

        let response = self.http
            .get(&url)
            .send()
            .await
            .context("Error conectando con Polymarket API")?;

        if !response.status().is_success() {
            anyhow::bail!(
                "API respondió con error: {} - {}",
                response.status(),
                response.text().await.unwrap_or_default()
            );
        }

        let api_markets: Vec<ApiMarketResponse> = response
            .json()
            .await
            .context("Error parseando respuesta de mercados")?;

        let markets = api_markets
            .into_iter()
            .map(|m| self.convert_market(m))
            .collect();

        Ok(markets)
    }

    /// Coloca una orden en un mercado de predicción
    pub async fn place_order(
        &self,
        market_id: &str,
        outcome: OutcomeType,
        side: SideType,
        amount: f64,
        price: f64,
    ) -> Result<Order> {
        let payload = ApiOrderPayload {
            market_id: market_id.to_string(),
            outcome: match outcome {
                OutcomeType::Yes => "Yes".to_string(),
                OutcomeType::No => "No".to_string(),
            },
            side: match side {
                SideType::Buy => "BUY".to_string(),
                SideType::Sell => "SELL".to_string(),
            },
            size: amount,
            price,
        };

        let url = format!("{}/order", self.config.rest_url);

        let response = self.http
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .json(&payload)
            .send()
            .await
            .context("Error enviando orden a Polymarket")?;

        if !response.status().is_success() {
            anyhow::bail!(
                "Error colocando orden: {} - {}",
                response.status(),
                response.text().await.unwrap_or_default()
            );
        }

        let order_id: String = response
            .json::<serde_json::Value>()
            .await
            .context("Error parseando respuesta de orden")?
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();

        Ok(Order {
            order_id,
            market_id: market_id.to_string(),
            outcome,
            side,
            amount,
            limit_price: price,
            status: OrderStatusType::Pending,
            created_at_ms: chrono::Utc::now().timestamp_millis(),
        })
    }

    /// Cancela una orden existente
    pub async fn cancel_order(&self, order_id: &str) -> Result<bool> {
        let url = format!("{}/order/{}", self.config.rest_url, order_id);

        let response = self.http
            .delete(&url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .send()
            .await
            .context("Error cancelando orden")?;

        Ok(response.status().is_success())
    }

    /// Convierte la respuesta de la API a nuestro tipo interno
    fn convert_market(&self, api: ApiMarketResponse) -> PredictionMarket {
        let yes_price = api.tokens.iter()
            .find(|t| t.outcome.to_lowercase() == "yes")
            .map(|t| t.price)
            .unwrap_or(0.0);

        let no_price = api.tokens.iter()
            .find(|t| t.outcome.to_lowercase() == "no")
            .map(|t| t.price)
            .unwrap_or(0.0);

        let status = if api.closed {
            MarketStatusType::Closed
        } else if api.active {
            MarketStatusType::Active
        } else {
            MarketStatusType::Resolved
        };

        PredictionMarket {
            market_id: api.condition_id,
            question: api.question,
            description: api.description,
            yes_price,
            no_price,
            volume: api.volume.parse().unwrap_or(0.0),
            liquidity: api.liquidity.parse().unwrap_or(0.0),
            end_date_ms: 0, // Se parsearía desde end_date_iso
            status,
            category: api.category,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let config = ApiConfig {
            rest_url: "https://clob.polymarket.com".to_string(),
            ws_url: "wss://ws.polymarket.com".to_string(),
            api_key: String::new(),
            timeout_secs: 10,
        };
        let client = PolymarketClient::new(&config);
        assert!(client.is_ok());
    }

    #[test]
    fn test_convert_market() {
        let config = ApiConfig {
            rest_url: "https://clob.polymarket.com".to_string(),
            ws_url: "wss://ws.polymarket.com".to_string(),
            api_key: String::new(),
            timeout_secs: 10,
        };
        let client = PolymarketClient::new(&config).unwrap();

        let api_market = ApiMarketResponse {
            tokens: vec![
                ApiToken { outcome: "Yes".to_string(), price: 0.65 },
                ApiToken { outcome: "No".to_string(), price: 0.35 },
            ],
            question: "Will BTC reach 100k?".to_string(),
            description: "Test market".to_string(),
            condition_id: "abc123".to_string(),
            volume: "50000.0".to_string(),
            liquidity: "10000.0".to_string(),
            end_date_iso: String::new(),
            active: true,
            closed: false,
            category: "Crypto".to_string(),
        };

        let market = client.convert_market(api_market);
        assert_eq!(market.question, "Will BTC reach 100k?");
        assert!((market.yes_price - 0.65).abs() < f64::EPSILON);
        assert!((market.no_price - 0.35).abs() < f64::EPSILON);
        assert_eq!(market.status, MarketStatusType::Active);
    }
}
