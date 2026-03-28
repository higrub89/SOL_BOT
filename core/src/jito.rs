//! # Jito Bundle Integration
//!
//! Cliente minimalista para enviar bundles a la Jito Block Engine.
//! Permite transacciones privadas y protegidas contra MEV (Sandwich Attacks).

use anyhow::{Context, Result};
use serde_json::json;
use solana_sdk::{pubkey::Pubkey, transaction::VersionedTransaction};
use solana_system_interface::instruction as system_instruction;
use std::str::FromStr;
// use base64::{Engine as _, engine::general_purpose};

// Tip Accounts oficiales de Jito (Mainnet)
const JITO_TIP_ACCOUNTS: [&str; 8] = [
    "96gYZGLnJYVFmbjzopPSU6QiEV5fGqZNyN9nmNhvrZU5",
    "HFqU5x63VTqvQss8hp11i4wVV8bD44PvwucfZ2bU7gRe",
    "Cw8CFyM9FkoMi7K7Crf6HNQqf4uEMzpKw6QNghXLvLkY",
    "ADaUMid9yfUytqMBgopwjb2DTLSokTSzL1zt6iGPaS49",
    "DfXygSm4jCyNCybVYYK6DwvWqjKee8pbDmJGcLWNDXjh",
    "ADuUkR4vqLUMWXxW9gh6D6L8pMSawimctcNZ5pGwDcEt",
    "DttWaMuVvTiduZRnguLF7jNxTgiMBZ1hyAumKUiL2KRL",
    "3AVi9Tg9Uo68tJfuvoKvqKNWKkC5wPdSSdeBnizKZ6jT",
];

// Endpoints oficiales del Block Engine de Jito para rotación/resiliencia
const JITO_ENDPOINTS: [&str; 4] = [
    "https://amsterdam.mainnet.block-engine.jito.wtf/api/v1/bundles",
    "https://frankfurt.mainnet.block-engine.jito.wtf/api/v1/bundles",
    "https://ny.mainnet.block-engine.jito.wtf/api/v1/bundles",
    "https://tokyo.mainnet.block-engine.jito.wtf/api/v1/bundles",
];

pub struct JitoClient {
    client: reqwest::Client,
}

impl Default for JitoClient {
    fn default() -> Self {
        Self::new()
    }
}

impl JitoClient {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    /// Obtiene una cuenta de propina aleatoria
    pub fn get_random_tip_account() -> Result<Pubkey> {
        use rand::seq::SliceRandom;
        let mut rng = rand::thread_rng();
        let account_str = JITO_TIP_ACCOUNTS
            .choose(&mut rng)
            .context("Lista de JITO_TIP_ACCOUNTS está vacía")?;

        Pubkey::from_str(account_str).map_err(|e| {
            anyhow::anyhow!("Error parseando Jito Tip Account '{}': {}", account_str, e)
        })
    }

    /// Crea una instrucción de transferencia para la propina
    pub fn create_tip_instruction(
        payer: &Pubkey,
        lamports: u64,
    ) -> Result<solana_sdk::instruction::Instruction> {
        let tip_account = Self::get_random_tip_account()?;
        Ok(system_instruction::transfer(payer, &tip_account, lamports))
    }

    /// Envía un bundle de transacciones a Jito
    pub async fn send_bundle(&self, transactions: Vec<VersionedTransaction>) -> Result<String> {
        if transactions.is_empty() {
            anyhow::bail!("Bundle vacío");
        }

        // Serializar transacciones a base58 (Jito espera base58 en JSON-RPC)
        let mut encoded_txs = Vec::with_capacity(transactions.len());
        for tx in &transactions {
            let bytes = postcard::to_allocvec(tx)
                .context("Error serializando transacción con postcard para Jito")?;
            encoded_txs.push(bs58::encode(bytes).into_string());
        }

        // Construir request JSON-RPC
        let request = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "sendBundle",
            "params": [
                encoded_txs
            ]
        });

        let mut max_retries = 3;
        let mut current_endpoint_idx = 0;
        let mut delay = std::time::Duration::from_millis(150);

        loop {
            let endpoint = JITO_ENDPOINTS[current_endpoint_idx % JITO_ENDPOINTS.len()];
            println!(
                "📡 Enviando Jito Bundle ({} txs) a {}...",
                transactions.len(),
                endpoint
            );

            let res = self
                .client
                .post(endpoint)
                .header("Content-Type", "application/json")
                .json(&request)
                .send()
                .await;

            match res {
                Ok(response) => {
                    if response.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
                        if max_retries > 0 {
                            eprintln!(
                                "⚠️ [Jito] 429 Too Many Requests -> Rotando y reintentando en {:?}",
                                delay
                            );
                            max_retries -= 1;
                            current_endpoint_idx += 1;
                            tokio::time::sleep(delay).await;
                            delay *= 2; // Exp backoff
                            continue;
                        } else {
                            anyhow::bail!("Jito Error: 429 Too Many Requests (Exhausted retries)");
                        }
                    }

                    let response_text = response.text().await?;
                    let response_json: serde_json::Value = serde_json::from_str(&response_text)
                        .context("Error parseando respuesta Jito")?;

                    if let Some(result) = response_json.get("result") {
                        let bundle_id = result.as_str().unwrap_or("unknown").to_string();
                        println!("✅ Jito Bundle Enviado. ID: {}", bundle_id);
                        return Ok(bundle_id);
                    } else if let Some(error) = response_json.get("error") {
                        let msg = error
                            .get("message")
                            .and_then(|m| m.as_str())
                            .unwrap_or("Unknown error");

                        if msg.contains("Rate limit") && max_retries > 0 {
                            eprintln!("⚠️ [Jito] Rate limit body -> Rotando");
                            max_retries -= 1;
                            current_endpoint_idx += 1;
                            tokio::time::sleep(delay).await;
                            delay *= 2;
                            continue;
                        }

                        anyhow::bail!("Jito Error: {}", msg);
                    } else {
                        anyhow::bail!("Respuesta Jito inesperada: {}", response_text);
                    }
                }
                Err(e) => {
                    if max_retries > 0 {
                        eprintln!("⚠️ [Jito] Network error -> Rotando: {}", e);
                        max_retries -= 1;
                        current_endpoint_idx += 1;
                        tokio::time::sleep(delay).await;
                        delay *= 2;
                        continue;
                    } else {
                        anyhow::bail!("Error fatal conectando a Jito: {}", e);
                    }
                }
            }
        }
    }
}
