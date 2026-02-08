//! # Wallet Engine
//! 
//! Módulo para monitoreo ultra-rápido de balances y transacciones.

use anyhow::Result;
use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
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
