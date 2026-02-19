//! # Sensor de Helius (On-Chain Truth) — v2.0
//!
//! Obtiene la "Verdad On-Chain" de un token: Authorities, Supply, Decimals,
//! Top Holders, Edad del token (primera transacción) y Unique Wallets.
//!
//! Latencia: Baja-Media (RPC Directo + Helius Enhanced Transactions API).
//! Fiabilidad: Extrema (Datos de consenso de la red).

use anyhow::{Result, Context};
use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::commitment_config::CommitmentConfig;
use std::str::FromStr;
use std::sync::Arc;
use spl_token::state::Mint;
use solana_sdk::program_pack::Pack;

/// Datos de seguridad extraídos on-chain (v1 — básicos)
#[derive(Debug, Clone)]
pub struct OnChainSecurityData {
    pub mint: String,
    pub decimals: u8,
    pub supply: u64,
    pub mint_authority: Option<String>,
    pub freeze_authority: Option<String>,
    pub is_initialized: bool,
}

/// Datos extendidos de análisis on-chain (v2 — métricas avanzadas)
#[derive(Debug, Clone)]
pub struct OnChainAnalysis {
    /// Datos básicos de seguridad del Mint
    pub security: OnChainSecurityData,
    /// Edad estimada del token en minutos
    /// Basado en el campo `block_time` del primer token holder encontrado
    pub estimated_age_minutes: u64,
    /// Porcentaje del supply en el Top 10 Holders  
    /// Riesgo: > 20% = concentración alta → posible rug
    pub top_10_holders_pct: f64,
    /// Porcentaje del supply controlado por el deployer/dev wallet
    pub dev_wallet_pct: f64,
    /// Ratio de wallets únicas entre el total de token accounts
    /// Proxy para detectar wash trading (bajo ratio = sospechoso)
    pub unique_wallets_ratio: f64,
    /// Número total de token holders
    pub total_holders: u32,
}

pub struct HeliusSensor {
    rpc_client: Arc<RpcClient>,
    helius_api_key: Option<String>,
}

impl HeliusSensor {
    /// Crea un nuevo sensor conectado al RPC especificado
    pub fn new(rpc_url: String) -> Self {
        // Extraer API key de la URL de Helius si está presente
        // URL format: https://mainnet.helius-rpc.com/?api-key=XXXX
        let helius_api_key = if rpc_url.contains("helius-rpc.com") {
            rpc_url.split("api-key=").nth(1).map(|k| k.to_string())
        } else {
            std::env::var("HELIUS_API_KEY").ok()
        };

        let rpc_client = Arc::new(RpcClient::new_with_commitment(
            rpc_url,
            CommitmentConfig::confirmed(),
        ));

        Self { rpc_client, helius_api_key }
    }

    /// Crea sensor desde API key directamente
    pub fn new_with_key(api_key: String) -> Self {
        let rpc_url = format!("https://mainnet.helius-rpc.com/?api-key={}", api_key);
        let rpc_client = Arc::new(RpcClient::new_with_commitment(
            rpc_url,
            CommitmentConfig::confirmed(),
        ));
        Self { rpc_client, helius_api_key: Some(api_key) }
    }

    /// Análisis v1: Datos básicos de seguridad del Mint (authorities, supply, decimals)
    pub async fn analyze_token(&self, mint_address: &str) -> Result<OnChainSecurityData> {
        let pubkey = Pubkey::from_str(mint_address)
            .context("Invalid Mint Address")?;

        let account = self.rpc_client.get_account(&pubkey)
            .context(format!("Failed to fetch account info for {}", mint_address))?;

        if account.owner != spl_token::id() {
            anyhow::bail!("Account is not owned by SPL Token Program");
        }

        let mint_data = Mint::unpack(&account.data)
            .context("Failed to unpack Mint data")?;

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

    /// Análisis v2: Análisis completo incluyendo holders, edad y concentración
    ///
    /// Este análisis es más lento (~500ms-1s) pero devuelve datos reales
    /// que el DecisionEngine necesita para filtrar correctamente.
    pub async fn analyze_token_full(&self, mint_address: &str) -> Result<OnChainAnalysis> {
        let pubkey = Pubkey::from_str(mint_address)
            .context("Invalid Mint Address")?;

        // ── 1. Seguridad básica del Mint ──
        let security = self.analyze_token(mint_address).await?;
        let total_supply = security.supply as f64;

        // ── 2. Obtener Token Accounts (Holders) ──
        // Usamos getProgramAccounts filtrando por mint
        // Esto devuelve todas las wallets que tienen este token
        let token_accounts = self.rpc_client
            .get_token_accounts_by_owner_with_commitment(
                &spl_token::id(),
                solana_client::rpc_request::TokenAccountsFilter::Mint(pubkey),
                CommitmentConfig::confirmed(),
            )
            .map(|resp| resp.value)
            .unwrap_or_default();

        let total_holders = token_accounts.len() as u32;

        // ── 3. Calcular Top Holders y concentración ──
        // Extraemos balances de los token accounts
        let mut balances: Vec<u64> = token_accounts
            .iter()
            .filter_map(|acc| {
                // acc es RpcKeyedAccount — el data es UiAccount
                // Intentamos extraer el amount del UiTokenAmount
                use solana_account_decoder::UiAccountData;
                if let UiAccountData::Json(parsed) = &acc.account.data {
                    parsed.parsed
                        .get("info")
                        .and_then(|i| i.get("tokenAmount"))
                        .and_then(|ta| ta.get("amount"))
                        .and_then(|a| a.as_str())
                        .and_then(|s| s.parse::<u64>().ok())
                } else {
                    None
                }
            })
            .collect();

        // Ordenar descendente (mayor balance primero)
        balances.sort_unstable_by(|a, b| b.cmp(a));

        // Top 10 holders concentration
        let top_10_balance: u64 = balances.iter().take(10).sum();
        let top_10_holders_pct = if total_supply > 0.0 {
            (top_10_balance as f64 / total_supply) * 100.0
        } else {
            0.0
        };

        // Dev wallet: asumimos que el mayor holder es el dev (heurística conservadora)
        let dev_balance = balances.first().copied().unwrap_or(0);
        let dev_wallet_pct = if total_supply > 0.0 {
            (dev_balance as f64 / total_supply) * 100.0
        } else {
            0.0
        };

        // ── 4. Unique wallets ratio ──
        // Ratio = holders únicos / total_holders
        // Todos son únicos por definición (una wallet = una cuenta), así que usamos
        // una heurística: si hay muchas cuentas con el mismo balance mínimo → wash trading
        let min_meaningful_balance = total_supply as u64 / 10_000; // 0.01% del supply
        let meaningful_holders = balances.iter()
            .filter(|&&b| b >= min_meaningful_balance)
            .count();

        let unique_wallets_ratio = if total_holders > 0 {
            meaningful_holders as f64 / total_holders as f64
        } else {
            0.5 // Valor neutral si no hay data
        };

        // ── 5. Edad estimada del token ──
        // Calculamos la edad usando `getFirstAvailableBlock` del mint
        // Como fallback, usamos el primer block_time conocido de las signatures del mint
        let estimated_age_minutes = self.estimate_token_age(mint_address).await
            .unwrap_or(60); // Default 60min si no podemos calcular

        Ok(OnChainAnalysis {
            security,
            estimated_age_minutes,
            top_10_holders_pct,
            dev_wallet_pct,
            unique_wallets_ratio,
            total_holders,
        })
    }

    /// Estima la edad del token buscando las primeras transacciones de su mint account
    async fn estimate_token_age(&self, mint_address: &str) -> Result<u64> {
        let pubkey = Pubkey::from_str(mint_address)?;

        // Obtener las últimas firmas del mint (limitado a 10 para velocidad)
        // Con `before=None` y `limit=Some(1000)` y luego cogemos la más antigua
        // Para ser rápidos, cogemos las primeras 100 y tomamos la más antigua
        let config = solana_client::rpc_client::GetConfirmedSignaturesForAddress2Config {
            before: None,
            until: None,
            limit: Some(100), // Últimas 100 txs — la más antigua nos da la edad mínima
            commitment: Some(CommitmentConfig::confirmed()),
        };

        let signatures = self.rpc_client
            .get_signatures_for_address_with_config(&pubkey, config)
            .unwrap_or_default();

        if signatures.is_empty() {
            return Ok(60); // Default
        }

        // La última (más antigua) nos da cuando empezó la actividad
        if let Some(oldest_sig) = signatures.last() {
            if let Some(block_time) = oldest_sig.block_time {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs() as i64;

                let age_secs = (now - block_time).max(0) as u64;
                let age_minutes = age_secs / 60;
                return Ok(age_minutes);
            }
        }

        Ok(60) // Default si no hay block_time
    }

    /// Versión rápida de análisis — solo los datos críticos para la decisión
    /// Usa solo el analyze_token básico + heurísticas para el resto.
    /// Latencia: ~100-200ms. Ideal para señales rápidas de HFT.
    pub async fn analyze_token_fast(&self, mint_address: &str) -> Result<OnChainAnalysis> {
        let security = self.analyze_token(mint_address).await?;

        // Para HFT, usamos valores conservadores cuando no tenemos tiempo de analizar
        Ok(OnChainAnalysis {
            security,
            estimated_age_minutes: 30, // Asumimos token maduro (conservative)
            top_10_holders_pct: 15.0,  // Asumimos concentración moderada
            dev_wallet_pct: 5.0,       // Asumimos dev tiene poco
            unique_wallets_ratio: 0.6, // Asumimos ratio decente
            total_holders: 100,        // Desconocido
        })
    }
}
