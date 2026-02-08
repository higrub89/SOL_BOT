//! # Jupiter Aggregator Integration
//! 
//! Módulo para ejecutar swaps usando Jupiter Aggregator API v6.
//! Proporciona las mejores rutas de intercambio con slippage mínimo.

use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use reqwest::Client;

/// Cliente para interactuar con Jupiter Aggregator
pub struct JupiterClient {
    client: Client,
    base_url: String,
}

impl JupiterClient {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            base_url: "https://quote-api.jup.ag/v6".to_string(),
        }
    }

    /// Obtiene un quote para un swap
    pub async fn get_quote(
        &self,
        input_mint: &str,
        output_mint: &str,
        amount: u64,
        slippage_bps: u16, // Basis points (100 = 1%)
    ) -> Result<QuoteResponse> {
        let url = format!(
            "{}/quote?inputMint={}&outputMint={}&amount={}&slippageBps={}",
            self.base_url, input_mint, output_mint, amount, slippage_bps
        );

        let response = self.client
            .get(&url)
            .send()
            .await
            .context("Error al obtener quote de Jupiter")?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("Jupiter API error: {}", error_text);
        }

        let quote: QuoteResponse = response
            .json()
            .await
            .context("Error al parsear respuesta de Jupiter")?;

        Ok(quote)
    }

    /// Obtiene una transacción swap serializada
    pub async fn get_swap_transaction(
        &self,
        quote: &QuoteResponse,
        user_public_key: &str,
        wrap_unwrap_sol: bool,
    ) -> Result<SwapTransactionResponse> {
        let url = format!("{}/swap", self.base_url);

        let request = SwapRequest {
            quote_response: quote.clone(),
            user_public_key: user_public_key.to_string(),
            wrap_and_unwrap_sol: Some(wrap_unwrap_sol),
            compute_unit_price_micro_lamports: Some(50000), // Priority fee
            as_legacy_transaction: Some(false),
        };

        let response = self.client
            .post(&url)
            .json(&request)
            .send()
            .await
            .context("Error al obtener transacción de Jupiter")?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("Jupiter swap error: {}", error_text);
        }

        let swap_tx: SwapTransactionResponse = response
            .json()
            .await
            .context("Error al parsear transacción de swap")?;

        Ok(swap_tx)
    }

    /// Calcula el precio efectivo del swap (incluyendo fees)
    pub fn calculate_effective_price(&self, quote: &QuoteResponse) -> f64 {
        let in_amount = quote.in_amount.parse::<f64>().unwrap_or(0.0);
        let out_amount = quote.out_amount.parse::<f64>().unwrap_or(0.0);

        if in_amount == 0.0 {
            return 0.0;
        }

        out_amount / in_amount
    }

    /// Muestra información detallada del quote
    pub fn print_quote_summary(&self, quote: &QuoteResponse) {
        println!("╔════════════════════════════════════════════════════════════╗");
        println!("║               📊 JUPITER QUOTE SUMMARY                    ║");
        println!("╚════════════════════════════════════════════════════════════╝\n");
        println!("🔄 Route:");
        for (i, route) in quote.route_plan.iter().enumerate() {
            println!("   {}. {} → {}", i + 1, route.swap_info.label, route.percent);
        }
        println!("\n💰 Amounts:");
        println!("   • Input:       {} tokens", quote.in_amount);
        println!("   • Output:      {} tokens", quote.out_amount);
        println!("   • Other fees:  {} lamports", quote.other_amount_threshold);
        println!("\n📉 Slippage:");
        println!("   • Configured:  {}%", quote.slippage_bps as f64 / 100.0);
        println!("   • Price Impact: {:.2}%", quote.price_impact_pct.parse::<f64>().unwrap_or(0.0));
    }
}

/// Respuesta de Jupiter Quote API
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuoteResponse {
    pub input_mint: String,
    pub in_amount: String,
    pub output_mint: String,
    pub out_amount: String,
    pub other_amount_threshold: String,
    pub swap_mode: String,
    pub slippage_bps: u16,
    pub price_impact_pct: String,
    pub route_plan: Vec<RoutePlan>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RoutePlan {
    pub swap_info: SwapInfo,
    pub percent: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SwapInfo {
    pub amm_key: String,
    pub label: String,
    pub input_mint: String,
    pub output_mint: String,
    pub in_amount: String,
    pub out_amount: String,
    pub fee_amount: String,
    pub fee_mint: String,
}

/// Request para obtener transacción de swap
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SwapRequest {
    quote_response: QuoteResponse,
    user_public_key: String,
    wrap_and_unwrap_sol: Option<bool>,
    compute_unit_price_micro_lamports: Option<u64>,
    as_legacy_transaction: Option<bool>,
}

/// Respuesta con la transacción serializada
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SwapTransactionResponse {
    pub swap_transaction: String, // Base64 encoded transaction
    pub last_valid_block_height: u64,
}

/// Información de resultado de swap ejecutado
#[derive(Debug, Clone)]
pub struct SwapResult {
    pub signature: String,
    pub input_amount: f64,
    pub output_amount: f64,
    pub route: String,
    pub price_impact_pct: f64,
}

impl SwapResult {
    pub fn print_summary(&self) {
        println!("╔════════════════════════════════════════════════════════════╗");
        println!("║               ✅ SWAP EJECUTADO CON ÉXITO                ║");
        println!("╚════════════════════════════════════════════════════════════╝\n");
        println!("📝 Signature:     {}", self.signature);
        println!("🔄 Route:         {}", self.route);
        println!("💎 Input:         {:.4} tokens", self.input_amount);
        println!("💰 Output:        {:.6} SOL", self.output_amount);
        println!("📉 Price Impact:  {:.2}%\n", self.price_impact_pct);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_jupiter_client_creation() {
        let client = JupiterClient::new();
        assert_eq!(client.base_url, "https://quote-api.jup.ag/v6");
    }

    #[tokio::test]
    #[ignore] // Requiere conexión a internet
    async fn test_get_quote() {
        let client = JupiterClient::new();
        
        // SOL mint address (wrapped SOL)
        let sol_mint = "So11111111111111111111111111111111111111112";
        // USDC mint address
        let usdc_mint = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";
        
        let result = client.get_quote(
            sol_mint,
            usdc_mint,
            1_000_000_000, // 1 SOL
            50, // 0.5% de slippage
        ).await;
        
        assert!(result.is_ok());
    }
}
