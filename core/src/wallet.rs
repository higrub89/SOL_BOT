//! # Wallet Engine
//!
//! Módulo para monitoreo ultra-rápido de balances y transacciones.

use anyhow::{anyhow, Context, Result};
use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Keypair;
use std::env;
use std::process::Command;
use std::str::FromStr;

const GCP_PROJECT_ID: &str = "project-828d4ae0-6385-40d2-aa6";

pub struct WalletMonitor {
    rpc_url: String,
    pubkey: Pubkey,
}

impl WalletMonitor {
    pub fn new(rpc_url: String, wallet_addr: &str) -> Result<Self> {
        let pubkey = Pubkey::from_str(wallet_addr)?;
        Ok(Self { rpc_url, pubkey })
    }

    /// Obtiene el balance de SOL en tiempo real
    pub fn get_sol_balance(&self) -> Result<f64> {
        let client = RpcClient::new(&self.rpc_url);
        let lamports = client.get_balance(&self.pubkey)?;
        Ok(lamports as f64 / 1_000_000_000.0)
    }

    /// Obtiene el balance de un token específico (SPL Token)
    /// Para simplificar esta versión, usamos directamente el mint pubkey
    pub fn get_token_balance(&self, mint_addr: &str) -> Result<f64> {
        let mint_pubkey = Pubkey::from_str(mint_addr)?;
        let client = RpcClient::new(&self.rpc_url);

        // En esta versión simplificada, intentamos obtener el balance directamente.
        // En una versión final por gRPC, recibiríamos account updates.
        match client.get_token_account_balance(&mint_pubkey) {
            Ok(balance) => Ok(balance.ui_amount.unwrap_or(0.0)),
            Err(_) => Ok(0.0),
        }
    }
}

/// Obtiene una variable de entorno o, si no existe, intenta recuperarla de GCP Secret Manager.
pub fn get_env_or_secret(name: &str) -> Result<String> {
    // 1. Intentar desde variable de entorno
    if let Ok(val) = env::var(name) {
        return Ok(val);
    }

    // 2. Intentar desde GCP
    println!(
        "🔐 {} no encontrado en ENV. Intentando recuperar desde GCP Secret Manager...",
        name
    );

    // Mapeo especial para compatibilidad si es necesario
    let secret_name = match name {
        "WALLET_PRIVATE_KEY" => "CHASSIS_WALLET_KEY",
        "WALLET_ADDRESS" => "WALLET_ADDRESS",
        _ => name,
    };

    fetch_secret_from_gcp(secret_name)
        .with_context(|| format!("❌ Error crítico: No se pudo obtener el secreto '{}' ni de las variables de entorno ni de GCP Secret Manager. Asegúrate de que la variable esté configurada o que el bot tenga permisos suficientes.", name))
}

/// Carga un Keypair buscando primero en el entorno y luego en GCP Secret Manager si es necesario.
pub fn load_keypair_secure(var_name: &str) -> Result<Keypair> {
    let raw = get_env_or_secret(var_name)?;
    parse_keypair(&raw)
}

pub fn fetch_secret_from_gcp(secret_name: &str) -> Result<String> {
    let output = Command::new("gcloud")
        .args([
            "secrets",
            "versions",
            "access",
            "latest",
            &format!("--secret={}", secret_name),
            &format!("--project={}", GCP_PROJECT_ID),
        ])
        .output()
        .context("Fallo al ejecutar el comando gcloud")?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("gcloud error: {}", err));
    }

    Ok(String::from_utf8(output.stdout)?.trim().to_string())
}

fn parse_keypair(raw: &str) -> Result<Keypair> {
    let trimmed = raw.trim();

    if trimmed.is_empty() {
        return Err(anyhow!("Keypair string is empty"));
    }

    if trimmed.starts_with('[') {
        let bytes: Vec<u8> = serde_json::from_str(trimmed).map_err(|e| {
            anyhow!(
                "Failed to parse JSON keypair: {} (Starts with: {})",
                e,
                &trimmed[..5.min(trimmed.len())]
            )
        })?;
        Keypair::from_bytes(&bytes).map_err(|e| anyhow!("Invalid JSON keypair: {}", e))
    } else {
        let bytes = bs58::decode(trimmed).into_vec().map_err(|e| {
            anyhow!(
                "Failed to decode Base58 keypair: {} (Starts with: {})",
                e,
                &trimmed[..5.min(trimmed.len())]
            )
        })?;
        Keypair::from_bytes(&bytes).map_err(|e| anyhow!("Invalid Base58 keypair: {}", e))
    }
}
