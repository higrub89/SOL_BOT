//! # Price Scanner
//!
//! Módulo para obtener precios en tiempo real de tokens desde Dexscreener.
//! Actualiza automáticamente las posiciones del Emergency Monitor.

use crate::validation::FinancialValidator;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::time::sleep;

/// Cliente para obtener precios de tokens
pub struct PriceScanner {
    client: reqwest::Client,
}

impl Default for PriceScanner {
    fn default() -> Self {
        Self::new()
    }
}

impl PriceScanner {
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .expect("Failed to create HTTP client - this should never fail");

        Self { client }
    }

    /// Obtiene el precio actual de un token desde Dexscreener
    pub async fn get_token_price(&self, token_address: &str) -> Result<TokenPrice> {
        let url = format!(
            "https://api.dexscreener.com/latest/dex/tokens/{}",
            token_address
        );

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .context("Error fetching price from Dexscreener")?;

        let data: DexScreenerResponse = response
            .json()
            .await
            .context("Error parsing Dexscreener response")?;

        // Intentar encontrar el mejor par, priorizando liquidez > volumen > precio
        let best_pair = data.pairs.iter().max_by(|a, b| {
            let liq_a = a.liquidity.as_ref().and_then(|l| l.usd).unwrap_or(0.0);
            let liq_b = b.liquidity.as_ref().and_then(|l| l.usd).unwrap_or(0.0);
            liq_a
                .partial_cmp(&liq_b)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        if let Some(pair) = best_pair {
            // Extraer valores con validación estricta, pero permitir fallback a 0.0 para evaluar tokens colapsados
            let price_usd_str = pair
                .price_usd
                .as_ref()
                .map(|s| s.as_str())
                .unwrap_or("0.0");

            let price_usd =
                FinancialValidator::parse_price_safe(price_usd_str, "DexScreener price_usd")?;

            let price_native = pair
                .price_native
                .as_ref()
                .and_then(|s| s.parse::<f64>().ok())
                .unwrap_or(0.0);

            let liquidity_usd = pair.liquidity.as_ref().and_then(|l| l.usd).unwrap_or(0.0);

            // Validar liquidez mínima (protección. Lo comentamos/logueamos pero NO fallamos)
            // Si la liquidez colapsa (rug pull), necesitamos informar el precio a 0 para que salte el stop-loss.
            if let Err(e) = FinancialValidator::validate_liquidity(
                liquidity_usd,
                100.0, // Mínimo $100 de liquidez
                "DexScreener liquidity",
            ) {
                eprintln!("⚠️ [Scanner] Liquidity warning: {}", e);
            }

            let volume_24h = pair.volume.as_ref().and_then(|v| v.h24).unwrap_or(0.0);

            let price_change_24h = pair
                .price_change
                .as_ref()
                .and_then(|c| c.h24)
                .unwrap_or(0.0);

            Ok(TokenPrice {
                price_usd,
                price_native,
                liquidity_usd,
                volume_24h,
                price_change_24h,
                symbol: pair.base_token.symbol.clone(),
                pair_address: pair.pair_address.clone(),
            })
        } else {
            eprintln!("⚠️ [Scanner] No trading pairs found for token {} (Rug pull/Dead). Emitting 0 price.", token_address);
            Ok(TokenPrice {
                price_usd: 0.0,
                price_native: 0.0,
                liquidity_usd: 0.0,
                volume_24h: 0.0,
                price_change_24h: -100.0,
                symbol: "UNKNOWN".to_string(),
                pair_address: "".to_string(),
            })
        }
    }

    /// Monitorea el precio de un token en loop continuo
    pub async fn monitor_price(
        &self,
        token_address: &str,
        symbol: &str,
        interval_secs: u64,
        mut callback: impl FnMut(TokenPrice) -> bool,
    ) -> Result<()> {
        println!("🔍 Iniciando monitoreo de precio para ${}", symbol);
        println!(
            "   • Address: {}...{}",
            &token_address[..8],
            &token_address[token_address.len() - 4..]
        );
        println!("   • Intervalo: {}s\n", interval_secs);

        let mut iteration = 0;

        loop {
            iteration += 1;

            match self.get_token_price(token_address).await {
                Ok(price) => {
                    // Llamar al callback con el precio
                    let should_continue = callback(price.clone());

                    if !should_continue {
                        println!("\n🛑 Monitor detenido por callback");
                        break;
                    }

                    // Mostrar update
                    if iteration % 5 == 0 {
                        self.print_price_update(&price, iteration);
                    }
                }
                Err(e) => {
                    eprintln!("⚠️  Error obteniendo precio: {}", e);
                }
            }

            sleep(Duration::from_secs(interval_secs)).await;
        }

        Ok(())
    }

    fn print_price_update(&self, price: &TokenPrice, iteration: usize) {
        let change_emoji = if price.price_change_24h >= 0.0 {
            "🟢"
        } else {
            "🔴"
        };

        println!("┌────────────────────────────────────────────────────────┐");
        println!(
            "│ 📊 Price Update #{:<3}                                │",
            iteration
        );
        println!("├────────────────────────────────────────────────────────┤");
        println!(
            "│ {} Price:      ${:.8}                    │",
            price.symbol, price.price_usd
        );
        println!(
            "│ {} 24h Change: {}{:.2}%                      │",
            change_emoji,
            if price.price_change_24h >= 0.0 {
                "+"
            } else {
                ""
            },
            price.price_change_24h
        );
        println!(
            "│ 💧 Liquidity:  ${:.0}                           │",
            price.liquidity_usd
        );
        println!(
            "│ 📈 Volume 24h: ${:.0}                           │",
            price.volume_24h
        );
        println!("└────────────────────────────────────────────────────────┘\n");
    }
}

/// Información de precio de un token
#[derive(Debug, Clone)]
pub struct TokenPrice {
    pub price_usd: f64,
    pub price_native: f64,
    pub liquidity_usd: f64,
    pub volume_24h: f64,
    pub price_change_24h: f64,
    pub symbol: String,
    pub pair_address: String,
}

// Estructuras para parsear respuesta de Dexscreener
#[derive(Debug, Deserialize, Serialize)]
struct DexScreenerResponse {
    pairs: Vec<DexPair>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct DexPair {
    pair_address: String,
    price_usd: Option<String>,
    price_native: Option<String>,
    liquidity: Option<Liquidity>,
    volume: Option<Volume>,
    price_change: Option<PriceChange>,
    base_token: BaseToken,
}

#[derive(Debug, Deserialize, Serialize)]
struct Liquidity {
    usd: Option<f64>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Volume {
    h24: Option<f64>,
}

#[derive(Debug, Deserialize, Serialize)]
struct PriceChange {
    h24: Option<f64>,
}

#[derive(Debug, Deserialize, Serialize)]
struct BaseToken {
    symbol: String,
}
