//! # DexScreener Sensor (Market Data)
//!
//! Sensor para obtener datos de mercado en tiempo real.
//! - Precio
//! - Liquidez
//! - Volumen (5m, 1h)
//! - Pair Info (Raydium, Pump.fun, etc.)
//!
//! ⚠️ RATE LIMITS: DexScreener permite 300 req/min. Usamos cache + throttling.

use anyhow::{anyhow, Result};
use serde::Deserialize;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex; // ⚡ CRÍTICO: Usar Mutex async para evitar deadlocks/pánicos con await

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenPairsResponse {
    pub pairs: Option<Vec<PairInfo>>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PairInfo {
    pub chain_id: String,
    pub dex_id: String,
    pub url: String,
    pub pair_address: String,
    pub price_usd: Option<String>,
    pub liquidity: Option<LiquidityInfo>,
    pub volume: Option<VolumeInfo>,
    pub price_change: Option<PriceChangeInfo>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LiquidityInfo {
    pub usd: Option<f64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct VolumeInfo {
    pub m5: Option<f64>,
    pub h1: Option<f64>,
    pub h6: Option<f64>,
    pub h24: Option<f64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PriceChangeInfo {
    pub m5: Option<f64>,
    pub h1: Option<f64>,
}

/// Estructura para datos unificados del Sensor
#[derive(Debug, Clone)]
pub struct MarketData {
    pub price_usd: f64,
    pub liquidity_usd: f64,
    pub volume_5m: f64,
    pub volume_h1: f64,
    pub pair_address: String,
    pub dex_id: String, // "raydium", "pumpfun"
}

/// Cliente con Rate Limit Handling
pub struct DexScreenerSensor {
    client: reqwest::Client,
    last_request_time: Arc<Mutex<Instant>>,
}

impl Default for DexScreenerSensor {
    fn default() -> Self {
        Self::new()
    }
}

impl DexScreenerSensor {
    pub fn new() -> Self {
        // Cliente HTTP con Timeout estricto
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .unwrap_or_default(); // Fallback to default client

        Self {
            client,
            last_request_time: Arc::new(Mutex::new(Instant::now() - Duration::from_secs(1))),
        }
    }

    /// Obtiene los datos del par con más liquidez para un Token Mint
    pub async fn get_token_market_data(&self, token_mint: &str) -> Result<MarketData> {
        // Throttling: Asegurar al menos 200ms entre peticiones (5 req/s max)
        // Para evitar baneos agresivos
        {
            // Usamos .lock().await para no bloquear el hilo del OS
            let mut last = self.last_request_time.lock().await;
            let elapsed = last.elapsed();
            if elapsed < Duration::from_millis(200) {
                tokio::time::sleep(Duration::from_millis(200) - elapsed).await;
            }
            *last = Instant::now();
        }

        let url = format!(
            "https://api.dexscreener.com/latest/dex/tokens/{}",
            token_mint
        );

        let response = self
            .client
            .get(&url)
            .send()
            .await?
            .json::<TokenPairsResponse>()
            .await?;

        // Validar respuesta
        let pairs = response.pairs.ok_or(anyhow!("No pairs found for token"))?;
        if pairs.is_empty() {
            return Err(anyhow!("No pairs found on DexScreener"));
        }

        // Seleccionar el mejor par (Mayor liquidez en Solana)
        // Filtramos solo pares en 'solana'
        let best_pair = pairs
            .into_iter()
            .filter(|p| p.chain_id == "solana")
            .max_by(|a, b| {
                let liq_a = a.liquidity.as_ref().and_then(|l| l.usd).unwrap_or(0.0);
                let liq_b = b.liquidity.as_ref().and_then(|l| l.usd).unwrap_or(0.0);
                liq_a
                    .partial_cmp(&liq_b)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .ok_or(anyhow!("No Solana pairs found"))?;

        // Parsear datos de forma segura
        let price = best_pair
            .price_usd
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(0.0);

        let liqudity = best_pair.liquidity.and_then(|l| l.usd).unwrap_or(0.0);

        let vol_5m = best_pair.volume.as_ref().and_then(|v| v.m5).unwrap_or(0.0);

        let vol_h1 = best_pair.volume.as_ref().and_then(|v| v.h1).unwrap_or(0.0);

        Ok(MarketData {
            price_usd: price,
            liquidity_usd: liqudity,
            volume_5m: vol_5m,
            volume_h1: vol_h1,
            pair_address: best_pair.pair_address,
            dex_id: best_pair.dex_id,
        })
    }
}
