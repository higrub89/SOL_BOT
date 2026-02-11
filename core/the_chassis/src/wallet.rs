//! # Wallet Engine
//! 
//! Módulo para monitoreo ultra-rápido de balances y transacciones.

use anyhow::{anyhow, Context, Result};
use solana_client::rpc_client::RpcClient;
use solana_sdk::signature::Keypair;
use solana_sdk::pubkey::Pubkey;
use std::env;
use std::str::FromStr;

pub struct WalletMonitor {
    client: RpcClient,
    pubkey: Pubkey,
}

impl WalletMonitor {
    pub fn new(rpc_url: String, wallet_addr: &str) -> Result<Self> {
        let client = RpcClient::new(rpc_url);
        let pubkey = Pubkey::from_str(wallet_addr)?;
        Ok(Self { client, pubkey })
    }

    /// Obtiene el balance de SOL en tiempo real
    pub fn get_sol_balance(&self) -> Result<f64> {
        let lamports = self.client.get_balance(&self.pubkey)?;
        Ok(lamports as f64 / 1_000_000_000.0)
    }

    /// Obtiene el balance de un token específico (SPL Token)
    /// Para simplificar esta versión, usamos directamente el mint pubkey
    pub fn get_token_balance(&self, mint_addr: &str) -> Result<f64> {
        let mint_pubkey = Pubkey::from_str(mint_addr)?;
        
        // En esta versión simplificada, intentamos obtener el balance directamente.
        // En una versión final por gRPC, recibiríamos account updates.
        match self.client.get_token_account_balance(&mint_pubkey) {
            Ok(balance) => Ok(balance.ui_amount.unwrap_or(0.0)),
            Err(_) => Ok(0.0),
        }
    }
}

/// Carga un Keypair desde variable de entorno.
/// Acepta formato Base58 o JSON array de bytes.
pub fn load_keypair_from_env(var_name: &str) -> Result<Keypair> {
    let raw = env::var(var_name)
        .with_context(|| format!("{} no encontrado en el entorno", var_name))?;
    parse_keypair(&raw)
}

fn parse_keypair(raw: &str) -> Result<Keypair> {
    let trimmed = raw.trim();

    if trimmed.starts_with('[') {
        let bytes: Vec<u8> = serde_json::from_str(trimmed)
            .context("WALLET_PRIVATE_KEY debe ser JSON array de bytes")?;
        Keypair::from_bytes(&bytes)
            .map_err(|e| anyhow!("Keypair JSON inválido: {}", e))
    } else {
        let bytes = bs58::decode(trimmed)
            .into_vec()
            .context("WALLET_PRIVATE_KEY no es Base58 válido")?;
        Keypair::from_bytes(&bytes)
            .map_err(|e| anyhow!("Keypair Base58 inválido: {}", e))
    }
}
