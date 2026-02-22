//! # Jito Bundle Integration
//! 
//! Cliente minimalista para enviar bundles a la Jito Block Engine.
//! Permite transacciones privadas y protegidas contra MEV (Sandwich Attacks).

use anyhow::{Result, Context};
use serde_json::json;
use solana_sdk::{
    transaction::VersionedTransaction,
    pubkey::Pubkey,
    system_instruction,
    signature::Keypair,
    signer::Signer,
};
use std::str::FromStr;
use base64::{Engine as _, engine::general_purpose};

// Tip Accounts oficiales de Jito (Mainnet)
const JITO_TIP_ACCOUNTS: [&str; 8] = [
    "96gYZGLnJYVFmbjzopPSU6QiEV5fGqZNyN9nmNhvrZU5",
    "HFqU5x63VTqvQss8hp11i4wVV8bD44PuwueBi2QtPgJc",
    "Cw8CFyM9FkoMi7K7Crf6HNQqf4uEMzpKw6QNghXLvLkY",
    "ADaUMid9yfUytqMBgopXSjbCp5R9CzowjpigYg72nZ29",
    "DfXygSm4jCyNCyb3qzK6967lsFpnStkNHU1TE1JJw2jn",
    "ADuUkR4ykGytmnb5LHydo2i1pJnBLn6W5LSvpOxhhUFy",
    "DttWaMuVvTiduZRnguLF7jNxTgiMBZ1hyAumKUiL2KRL",
    "3AVi9Tg9Uo68tJfuvoKvqKNWKkC5wPdSSdeBnIzKZ6jJ",
];

// Block Engine URL (Amsterdam es central para EU/US overlap, o usar NY)
const JITO_BLOCK_ENGINE_URL: &str = "https://amsterdam.mainnet.block-engine.jito.wtf/api/v1/bundles";

pub struct JitoClient {
    client: reqwest::Client,
}

impl JitoClient {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    /// Obtiene una cuenta de propina aleatoria
    pub fn get_random_tip_account() -> Pubkey {
        use rand::seq::SliceRandom;
        let mut rng = rand::thread_rng();
        let account_str = JITO_TIP_ACCOUNTS.choose(&mut rng).unwrap();
        Pubkey::from_str(account_str).unwrap()
    }

    /// Crea una instrucción de transferencia para la propina
    pub fn create_tip_instruction(payer: &Pubkey, lamports: u64) -> solana_sdk::instruction::Instruction {
        let tip_account = Self::get_random_tip_account();
        system_instruction::transfer(payer, &tip_account, lamports)
    }

    /// Envía un bundle de transacciones a Jito
    pub async fn send_bundle(&self, transactions: Vec<VersionedTransaction>) -> Result<String> {
        if transactions.is_empty() {
            anyhow::bail!("Bundle vacío");
        }

        // Serializar transacciones a base58 (Jito espera base58 en JSON-RPC)
        let encoded_txs: Vec<String> = transactions.iter()
            .map(|tx| {
                let bytes = bincode::serialize(tx).unwrap();
                bs58::encode(bytes).into_string()
            })
            .collect();

        // Construir request JSON-RPC
        let request = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "sendBundle",
            "params": [
                encoded_txs
            ]
        });

        println!("📡 Enviando Jito Bundle ({} txs)...", transactions.len());

        let response = self.client.post(JITO_BLOCK_ENGINE_URL)
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .context("Error conectando con Jito Block Engine")?;

        let response_text = response.text().await?;
        
        // Parsear respuesta
        let response_json: serde_json::Value = serde_json::from_str(&response_text)
            .context("Error parseando respuesta Jito")?;

        if let Some(result) = response_json.get("result") {
            // El result suele ser el Bundle ID (UUID)
            let bundle_id = result.as_str().unwrap_or("unknown").to_string();
            println!("✅ Jito Bundle Enviado. ID: {}", bundle_id);
            Ok(bundle_id)
        } else if let Some(error) = response_json.get("error") {
            let msg = error.get("message").and_then(|m| m.as_str()).unwrap_or("Unknown error");
            anyhow::bail!("Jito Error: {}", msg);
        } else {
            anyhow::bail!("Respuesta Jito inesperada: {}", response_text);
        }
    }
}
