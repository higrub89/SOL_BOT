//! # Raydium Executor - Direct DEX Access
//! 
//! Implementación del trait Executor para Raydium AMM v4.
//! Permite swaps directos sin intermediarios (Jupiter).
//! 
//! ## Ventajas
//! - ⚡ Latencia ultra-baja (<500ms total)
//! - 🛡️ Sin puntos de fallo externos
//! - 🎯 Control total sobre slippage y fees
//! 
//! ## Estado
//! 🚧 EN DESARROLLO - Sprint 1-4 del Roadmap Raydium

use anyhow::{Result, Context};
use async_trait::async_trait;
use solana_sdk::{
    pubkey::Pubkey,
    signature::{Keypair, Signature},
    instruction::{Instruction, AccountMeta},
    transaction::Transaction,
    commitment_config::CommitmentConfig,
};
use solana_client::rpc_client::RpcClient;
use std::str::FromStr;
use std::time::Instant;

use crate::executor_trait::{Executor, Quote, SwapExecution};

/// Raydium AMM v4 Program ID
const RAYDIUM_AMM_PROGRAM_ID: &str = "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8";

/// Serum DEX Program ID (usado por Raydium)
const SERUM_PROGRAM_ID: &str = "9xQeWvG816bUx9EPjHmaT23yvVM2ZWbrrpZb9PusVFin";

/// Configuración del executor de Raydium
#[derive(Debug, Clone)]
pub struct RaydiumConfig {
    pub rpc_url: String,
    pub commitment: CommitmentConfig,
}

impl Default for RaydiumConfig {
    fn default() -> Self {
        Self {
            rpc_url: "https://api.mainnet-beta.solana.com".to_string(),
            commitment: CommitmentConfig::confirmed(),
        }
    }
}

/// Executor de Raydium AMM v4
pub struct RaydiumExecutor {
    config: RaydiumConfig,
    rpc_client: RpcClient,
    program_id: Pubkey,
    /// Caché de pools para evitar llamadas RPC repetidas
    pool_cache: std::collections::HashMap<String, PoolInfo>,
}

/// Información de un pool de Raydium
#[derive(Debug, Clone)]
struct PoolInfo {
    /// AMM ID del pool
    amm_id: Pubkey,
    /// Authority del pool (PDA)
    amm_authority: Pubkey,
    /// Open orders en Serum
    amm_open_orders: Pubkey,
    /// Vault del token A
    coin_vault: Pubkey,
    /// Vault del token B (PC = Price Currency)
    pc_vault: Pubkey,
    /// Serum Market
    serum_market: Pubkey,
    /// Serum Bids
    serum_bids: Pubkey,
    /// Serum Asks
    serum_asks: Pubkey,
    /// Serum Event Queue
    serum_event_queue: Pubkey,
    /// Serum Coin Vault
    serum_coin_vault: Pubkey,
    /// Serum PC Vault
    serum_pc_vault: Pubkey,
    /// Serum Vault Signer
    serum_vault_signer: Pubkey,
}

impl RaydiumExecutor {
    pub fn new(config: RaydiumConfig) -> Self {
        let rpc_client = RpcClient::new_with_commitment(
            config.rpc_url.clone(),
            config.commitment,
        );

        let program_id = Pubkey::from_str(RAYDIUM_AMM_PROGRAM_ID)
            .expect("Invalid Raydium program ID");

        Self {
            config,
            rpc_client,
            program_id,
            pool_cache: std::collections::HashMap::new(),
        }
    }

    /// 🚧 Sprint 1: Pool Discovery (IMPLEMENTADO)
    /// Encuentra el pool ID para un par de tokens escaneando la blockchain
    async fn find_pool(
        &mut self,
        mint_a: &Pubkey,
        mint_b: &Pubkey,
    ) -> Result<PoolInfo> {
        let cache_key = format!("{}_{}", mint_a, mint_b);
        let reverse_key = format!("{}_{}", mint_b, mint_a);
        
        // 1. Revisar caché primero (O(1))
        if let Some(pool) = self.pool_cache.get(&cache_key) {
            tracing::debug!("🎯 Pool found in cache: {}", cache_key);
            return Ok(pool.clone());
        }
        if let Some(pool) = self.pool_cache.get(&reverse_key) {
            tracing::debug!("🎯 Pool found in cache (reverse): {}", reverse_key);
            return Ok(pool.clone());
        }

        tracing::info!("🔍 Scanning Raydium for pool {} / {}...", mint_a, mint_b);

        // 2. Definir offsets de memoria (Layout AMM v4)
        // coin_mint_offset = 400
        // pc_mint_offset = 432
        const COIN_MINT_OFFSET: usize = 400;
        const PC_MINT_OFFSET: usize = 432;
        const AMM_ACCOUNT_SIZE: u64 = 752;

        // Configuración de la consulta RPC
        let config = solana_client::rpc_config::RpcProgramAccountsConfig {
            filters: Some(vec![
                // Filtro 1: Tamaño exacto de cuenta AMM (752 bytes)
                solana_client::rpc_filter::RpcFilterType::DataSize(AMM_ACCOUNT_SIZE),
                // Filtro 2: Coin Mint == mint_a
                solana_client::rpc_filter::RpcFilterType::Memcmp(
                    solana_client::rpc_filter::Memcmp::new_base58_encoded(
                        COIN_MINT_OFFSET,
                        &mint_a.to_bytes(),
                    ),
                ),
                // Filtro 3: PC Mint == mint_b
                solana_client::rpc_filter::RpcFilterType::Memcmp(
                    solana_client::rpc_filter::Memcmp::new_base58_encoded(
                        PC_MINT_OFFSET,
                        &mint_b.to_bytes(),
                    ),
                ),
            ]),
            account_config: solana_client::rpc_config::RpcAccountInfoConfig {
                encoding: Some(solana_account_decoder::UiAccountEncoding::Base64),
                ..Default::default()
            },
            with_context: None,
        };

        // 3. Ejecutar consulta principal
        let accounts = self.rpc_client.get_program_accounts_with_config(
            &self.program_id,
            config.clone(),
        )?;

        if let Some((pubkey, account)) = accounts.first() {
            tracing::info!("✅ Pool found: {}", pubkey);
            let pool_info = self.parse_pool_account(pubkey, &account.data)?;
            
            // Guardar en caché
            self.pool_cache.insert(cache_key, pool_info.clone());
            return Ok(pool_info);
        }

        // 4. Si falla, intentar broadcast inverso (mint_b, mint_a)
        tracing::debug!("🔄 Trying reverse pair check...");
        
        let config_reverse = solana_client::rpc_config::RpcProgramAccountsConfig {
            filters: Some(vec![
                solana_client::rpc_filter::RpcFilterType::DataSize(AMM_ACCOUNT_SIZE),
                solana_client::rpc_filter::RpcFilterType::Memcmp(
                    solana_client::rpc_filter::Memcmp::new_base58_encoded(
                        COIN_MINT_OFFSET,
                        &mint_b.to_bytes(),
                    ),
                ),
                solana_client::rpc_filter::RpcFilterType::Memcmp(
                    solana_client::rpc_filter::Memcmp::new_base58_encoded(
                        PC_MINT_OFFSET,
                        &mint_a.to_bytes(),
                    ),
                ),
            ]),
            account_config: solana_client::rpc_config::RpcAccountInfoConfig {
                encoding: Some(solana_account_decoder::UiAccountEncoding::Base64),
                ..Default::default()
            },
            with_context: None,
        };

        let accounts_rev = self.rpc_client.get_program_accounts_with_config(
            &self.program_id,
            config_reverse,
        )?;

        if let Some((pubkey, account)) = accounts_rev.first() {
            tracing::info!("✅ Pool found (reverse): {}", pubkey);
            let pool_info = self.parse_pool_account(pubkey, &account.data)?;
            
            self.pool_cache.insert(reverse_key, pool_info.clone());
            return Ok(pool_info);
        }
        
        anyhow::bail!("Pool not found for pair {} / {}", mint_a, mint_b)
    }

    /// Parsea la data binaria de la cuenta para extraer las claves necesarias
    fn parse_pool_account(&self, amm_id: &Pubkey, data: &[u8]) -> Result<PoolInfo> {
        if data.len() != 752 {
            anyhow::bail!("Invalid data length for AMM account");
        }

        // Layout offsets (según Raydium SDK)
        // status: u64 (0)
        // nonce: u64 (8)
        // orderNum: u64 (16)
        // depth: u64 (24)
        // coinDecimals: u64 (32)
        // pcDecimals: u64 (40)
        // state: u64 (48)
        // resetFlag: u64 (56)
        // minSize: u64 (64)
        // volMaxCutRatio: u64 (72)
        // amountWaveRatio: u64 (80)
        // coinLotSize: u64 (88)
        // pcLotSize: u64 (96)
        // minPriceMultiplier: u64 (104)
        // maxPriceMultiplier: u64 (112)
        // systemDecimalsValue: u64 (120)
        // minSeparateNumerator: u64 (128)
        // minSeparateDenominator: u64 (136)
        // tradeFeeNumerator: u64 (144)
        // tradeFeeDenominator: u64 (152)
        // pnlNumerator: u64 (160)
        // pnlDenominator: u64 (168)
        // swapFeeNumerator: u64 (176)
        // swapFeeDenominator: u64 (184)
        // needTakePnlCoin: u64 (192)
        // needTakePnlPc: u64 (200)
        // totalPnlCoin: u64 (208)
        // totalPnlPc: u64 (216)
        // poolTotalDepositPc: u128 (224)
        // poolTotalDepositCoin: u128 (240)
        // swapCoinInAmount: u128 (256)
        // swapPcOutAmount: u128 (272)
        // swapCoin2PcFee: u64 (288)
        // swapPcInAmount: u128 (296)
        // swapCoinOutAmount: u128 (312)
        // swapPc2CoinFee: u64 (328)

        // poolCoinTokenAccount: Pubkey (32 bytes) @ 336
        // poolPcTokenAccount: Pubkey (32 bytes) @ 368
        // coinMint: Pubkey (32 bytes) @ 400
        // pcMint: Pubkey (32 bytes) @ 432
        // lpMint: Pubkey (32 bytes) @ 464
        // openOrders: Pubkey (32 bytes) @ 496
        // market: Pubkey (32 bytes) @ 528
        // serumMarket: Pubkey (32 bytes) @ 560
        // targetOrders: Pubkey (32 bytes) @ 592
        // withdrawQueue: Pubkey (32 bytes) @ 624
        // lpVault: Pubkey (32 bytes) @ 656
        // ammOwner: Pubkey (32 bytes) @ 688
        // pnlOwner: Pubkey (32 bytes) @ 720
        
        // Helper para leer pubkey
        let read_pubkey = |offset: usize| -> Pubkey {
            let slice: [u8; 32] = data[offset..offset+32].try_into().unwrap();
            Pubkey::from(slice)
        };

        let coin_vault = read_pubkey(336);
        let pc_vault = read_pubkey(368);
        let open_orders = read_pubkey(496);
        let market_id = read_pubkey(528);
        let serum_market = read_pubkey(560);
        let target_orders = read_pubkey(592);
        let amm_authority = read_pubkey(688); // owner

        // Para las cuentas de Serum, necesitamos leer el Market Account también
        // Esto será parte del Sprint 2. Por ahora usamos placeholders o segunda llamada
        
        // TEMPORAL: Asumimos values standard para Serum (esto se refinará en Sprint 2)
        // Necesitamos leer la cuenta del Serum Market para obtener bids, asks, event_queue
        
        // TODO: Sprint 2 - Leer Serum Market
        // Por ahora devolvemos lo que tenemos, pero incompleto para ejecución real
        
        Ok(PoolInfo {
            amm_id: *amm_id,
            amm_authority,
            amm_open_orders: open_orders,
            coin_vault,
            pc_vault,
            serum_market,
            // Placeholders que se rellenarán en Sprint 2
            serum_bids: Pubkey::default(), 
            serum_asks: Pubkey::default(),
            serum_event_queue: Pubkey::default(),
            serum_coin_vault: Pubkey::default(),
            serum_pc_vault: Pubkey::default(),
            serum_vault_signer: Pubkey::default(),
        })
    }

    /// 🚧 Sprint 2: Deserialización del Estado AMM
    /// Lee y parsea el estado de un pool
    async fn read_pool_state(&self, amm_id: &Pubkey) -> Result<AmmState> {
        // TODO: Implementar lectura y deserialización
        // Ver docs/RAYDIUM_IMPLEMENTATION.md Sprint 2
        
        anyhow::bail!("Pool state deserialization not yet implemented - Sprint 2 pending")
    }

    /// 🚧 Sprint 3: Construcción de Instrucción Swap
    /// Crea la instrucción de swap con todas las cuentas necesarias
    fn build_swap_instruction(
        &self,
        pool: &PoolInfo,
        user_source: &Pubkey,
        user_dest: &Pubkey,
        user_wallet: &Pubkey,
        amount_in: u64,
        min_amount_out: u64,
    ) -> Result<Instruction> {
        // Discriminador para SwapBaseIn
        let mut data = Vec::with_capacity(17);
        data.push(9); // SwapBaseIn opcode
        data.extend_from_slice(&amount_in.to_le_bytes());
        data.extend_from_slice(&min_amount_out.to_le_bytes());

        // Orden ESTRICTO de cuentas (ver RAYDIUM_IMPLEMENTATION.md)
        let accounts = vec![
            AccountMeta::new_readonly(spl_token::id(), false),
            AccountMeta::new(pool.amm_id, false),
            AccountMeta::new_readonly(pool.amm_authority, false),
            AccountMeta::new(pool.amm_open_orders, false),
            AccountMeta::new(pool.coin_vault, false),
            AccountMeta::new(pool.pc_vault, false),
            AccountMeta::new_readonly(Pubkey::from_str(SERUM_PROGRAM_ID).unwrap(), false),
            AccountMeta::new(pool.serum_market, false),
            AccountMeta::new(pool.serum_bids, false),
            AccountMeta::new(pool.serum_asks, false),
            AccountMeta::new(pool.serum_event_queue, false),
            AccountMeta::new(pool.serum_coin_vault, false),
            AccountMeta::new(pool.serum_pc_vault, false),
            AccountMeta::new_readonly(pool.serum_vault_signer, false),
            AccountMeta::new(*user_source, false),
            AccountMeta::new(*user_dest, false),
            AccountMeta::new_readonly(*user_wallet, true),
        ];

        Ok(Instruction {
            program_id: self.program_id,
            accounts,
            data,
        })
    }

    /// Calcula el min_amount_out con slippage
    fn calculate_min_amount_out(
        &self,
        expected_out: u64,
        slippage_bps: u16,
    ) -> u64 {
        let slippage_factor = 1.0 - (slippage_bps as f64 / 10000.0);
        (expected_out as f64 * slippage_factor) as u64
    }
}

/// Estado del AMM (simplificado para Sprint 2)
#[derive(Debug)]
struct AmmState {
    status: u64,
    coin_decimals: u8,
    pc_decimals: u8,
    coin_vault_amount: u64,
    pc_vault_amount: u64,
}

#[async_trait]
impl Executor for RaydiumExecutor {
    fn name(&self) -> &str {
        "Raydium-AMM-v4"
    }

    async fn get_quote(
        &self,
        input_mint: &Pubkey,
        output_mint: &Pubkey,
        amount: u64,
        slippage_bps: u16,
    ) -> Result<Quote> {
        tracing::info!("🔍 [RAYDIUM] Getting quote for {} → {}", input_mint, output_mint);
        
        // TODO: Implementar lógica completa en Sprint 1-3
        // Por ahora, devolver error indicando que está en desarrollo
        
        anyhow::bail!(
            "Raydium executor is under development. Use JupiterExecutor for now. \
            Progress: Pool Discovery (Sprint 1) pending."
        )
    }

    async fn execute_swap(
        &self,
        quote: &Quote,
        wallet: &Keypair,
        auto_unwrap: bool,
    ) -> Result<SwapExecution> {
        let start = Instant::now();
        
        tracing::info!("⚡ [RAYDIUM] Executing swap");
        
        // TODO: Implementar ejecución completa en Sprint 3-4
        
        anyhow::bail!(
            "Raydium executor is under development. Use JupiterExecutor for now. \
            Progress: Swap execution (Sprint 3-4) pending."
        )
    }

    async fn is_healthy(&self) -> bool {
        // Verificar si el RPC responde
        self.rpc_client.get_health().is_ok()
    }

    async fn avg_latency_ms(&self) -> u64 {
        // Placeholder - en producción, mantener promedio móvil
        450 // Target: <500ms
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_raydium_executor_creation() {
        let config = RaydiumConfig::default();
        let executor = RaydiumExecutor::new(config);
        assert_eq!(executor.name(), "Raydium-AMM-v4");
    }

    #[test]
    fn test_min_amount_out_calculation() {
        let config = RaydiumConfig::default();
        let executor = RaydiumExecutor::new(config);
        
        // 1% slippage sobre 1000 tokens
        let min_out = executor.calculate_min_amount_out(1000, 100);
        assert_eq!(min_out, 990); // 1% less
    }
}
