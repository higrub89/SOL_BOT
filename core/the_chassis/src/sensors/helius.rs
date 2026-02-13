//! # Sensor de Helius (On-Chain Truth)
//! 
//! Responsable de obtener la "Verdad On-Chain" de un token.
//! Consulta metadatos críticos de seguridad (Authorities, Supply, Decimals) directamente del RPC.
//! 
//! Latencia: Baja (RPC Directo).
//! Fiabilidad: Extrema (Datos de consenso de la red).

use anyhow::{Result, Context};
use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::commitment_config::CommitmentConfig;
use std::str::FromStr;
use std::sync::Arc;
use spl_token::state::Mint;
use solana_sdk::program_pack::Pack;

/// Datos de seguridad extraídos on-chain
#[derive(Debug, Clone)]
pub struct OnChainSecurityData {
    pub mint: String,
    pub decimals: u8,
    pub supply: u64,
    pub mint_authority: Option<String>,
    pub freeze_authority: Option<String>,
    pub is_initialized: bool,
}

pub struct HeliusSensor {
    rpc_client: Arc<RpcClient>,
}

impl HeliusSensor {
    /// Crea un nuevo sensor conectado al RPC especificado
    pub fn new(rpc_url: String) -> Self {
        let rpc_client = Arc::new(RpcClient::new_with_commitment(
            rpc_url,
            CommitmentConfig::confirmed(),
        ));
        
        Self { rpc_client }
    }

    /// Analiza un token Mint para extraer sus parámetros de seguridad
    pub async fn analyze_token(&self, mint_address: &str) -> Result<OnChainSecurityData> {
        let pubkey = Pubkey::from_str(mint_address)
            .context("Invalid Mint Address")?;

        // 1. Obtener Account Info del Mint
        // Usamos get_account del RPC estándar (compatible con cualquier nodo, Helius incluido)
        let account = self.rpc_client.get_account(&pubkey)
            .context(format!("Failed to fetch account info for {}", mint_address))?;

        // 2. Verificar que sea un Token Mint válido
        if account.owner != spl_token::id() {
            anyhow::bail!("Account is not owned by SPL Token Program");
        }

        // 3. Deserializar datos del Mint
        let mint_data = Mint::unpack(&account.data)
            .context("Failed to unpack Mint data")?;

        // 4. Extraer Autoridades (Critical Security Check)
        let mint_authority: Option<String> = {
            let opt_pubkey: Option<Pubkey> = mint_data.mint_authority.into();
            opt_pubkey.map(|p| p.to_string())
        };
        
        let freeze_authority: Option<String> = {
            let opt_pubkey: Option<Pubkey> = mint_data.freeze_authority.into();
            opt_pubkey.map(|p| p.to_string())
        };

        Ok(OnChainSecurityData {
            mint: mint_address.to_string(),
            decimals: mint_data.decimals,
            supply: mint_data.supply,
            mint_authority,
            freeze_authority,
            is_initialized: mint_data.is_initialized,
        })
    }
}
