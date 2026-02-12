//! # Raydium AMM v4 Direct Swap Implementation
//! 
//! Bypass de Jupiter para ejecución directa en Raydium Pools.
//! Latencia ultra-baja: Solo RPC → Blockchain.
//! 
//! Estado: PRODUCTION READY (Pool Discovery + Swap Execution)

use anyhow::{Result, Context};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    transaction::Transaction,
    commitment_config::CommitmentConfig,
};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::fs;
use std::collections::HashMap;

// ============================================================================
// CONSTANTS - Raydium & Serum Program IDs
// ============================================================================

const RAYDIUM_V4_PROGRAM_ID: &str = "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8";
const SERUM_PROGRAM_ID: &str = "9xQeWvG816bUx9EPjHmaT23yvVM2ZWbrrpZb9PusVFin";


// Swap instruction discriminator
const SWAP_BASE_IN_DISCRIMINATOR: u8 = 9;

// ============================================================================
// DATA STRUCTURES - Pool Info & Cache
// ============================================================================

/// Información completa de un pool de Raydium
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolInfo {
    pub name: String,
    pub base_mint: String,
    pub quote_mint: String,
    pub amm_id: String,
    pub amm_authority: String,
    pub amm_open_orders: String,
    pub coin_vault: String,
    pub pc_vault: String,
    pub lp_mint: String,
    pub serum_market: String,
    pub serum_bids: String,
    pub serum_asks: String,
    pub serum_event_queue: String,
    pub serum_coin_vault: String,
    pub serum_pc_vault: String,
    pub serum_vault_signer: String,
}

impl PoolInfo {
    /// Convierte los strings de direcciones a Pubkeys
    pub fn to_pubkeys(&self) -> Result<PoolKeys> {
        Ok(PoolKeys {
            amm_id: Pubkey::from_str(&self.amm_id)?,
            amm_authority: Pubkey::from_str(&self.amm_authority)?,
            amm_open_orders: Pubkey::from_str(&self.amm_open_orders)?,
            coin_vault: Pubkey::from_str(&self.coin_vault)?,
            pc_vault: Pubkey::from_str(&self.pc_vault)?,
            lp_mint: Pubkey::from_str(&self.lp_mint)?,
            serum_market: Pubkey::from_str(&self.serum_market)?,
            serum_bids: Pubkey::from_str(&self.serum_bids)?,
            serum_asks: Pubkey::from_str(&self.serum_asks)?,
            serum_event_queue: Pubkey::from_str(&self.serum_event_queue)?,
            serum_coin_vault: Pubkey::from_str(&self.serum_coin_vault)?,
            serum_pc_vault: Pubkey::from_str(&self.serum_pc_vault)?,
            serum_vault_signer: Pubkey::from_str(&self.serum_vault_signer)?,
            base_mint: Pubkey::from_str(&self.base_mint)?,
            quote_mint: Pubkey::from_str(&self.quote_mint)?,
        })
    }
}

/// Pool keys en formato Pubkey (listo para usar en instrucciones)
#[derive(Debug, Clone)]
pub struct PoolKeys {
    pub amm_id: Pubkey,
    pub amm_authority: Pubkey,
    pub amm_open_orders: Pubkey,
    pub coin_vault: Pubkey,
    pub pc_vault: Pubkey,
    pub lp_mint: Pubkey,
    pub serum_market: Pubkey,
    pub serum_bids: Pubkey,
    pub serum_asks: Pubkey,
    pub serum_event_queue: Pubkey,
    pub serum_coin_vault: Pubkey,
    pub serum_pc_vault: Pubkey,
    pub serum_vault_signer: Pubkey,
    pub base_mint: Pubkey,
    pub quote_mint: Pubkey,
}

/// Cache de pools cargado desde JSON
#[derive(Debug, Deserialize, Serialize)]
struct PoolsCache {
    version: String,
    pools: Vec<PoolInfo>,
}

// ============================================================================
// RAYDIUM CLIENT - Main Interface
// ============================================================================

pub struct RaydiumClient {
    rpc_client: RpcClient,
    program_id: Pubkey,
    serum_program_id: Pubkey,
    pool_cache: HashMap<String, PoolInfo>, // Clave: "BASE_MINT-QUOTE_MINT"
}

impl RaydiumClient {
    /// Inicializa el cliente con cache de pools
    pub fn new(rpc_url: String) -> Result<Self> {
        let rpc_client = RpcClient::new_with_commitment(
            rpc_url,
            CommitmentConfig::confirmed(),
        );
        let program_id = Pubkey::from_str(RAYDIUM_V4_PROGRAM_ID)?;
        let serum_program_id = Pubkey::from_str(SERUM_PROGRAM_ID)?;
        
        // Cargar cache de pools
        let pool_cache = Self::load_pool_cache()?;
        
        println!("✅ Raydium Client inicializado con {} pools en cache", pool_cache.len());
        
        Ok(Self {
            rpc_client,
            program_id,
            serum_program_id,
            pool_cache,
        })
    }

    /// Carga el cache de pools desde JSON
    fn load_pool_cache() -> Result<HashMap<String, PoolInfo>> {
        let cache_path = "pools_cache.json";
        
        if !std::path::Path::new(cache_path).exists() {
            println!("⚠️  pools_cache.json no encontrado. Cache vacío.");
            return Ok(HashMap::new());
        }
        
        let content = fs::read_to_string(cache_path)
            .context("Error leyendo pools_cache.json")?;
        
        let cache: PoolsCache = serde_json::from_str(&content)
            .context("Error parseando pools_cache.json")?;
        
        let mut map = HashMap::new();
        for pool in cache.pools {
            let key = format!("{}-{}", pool.base_mint, pool.quote_mint);
            map.insert(key, pool);
        }
        
        Ok(map)
    }

    /// Encuentra un pool por par de mints (primero intenta cache, luego RPC)
    pub fn find_pool(&self, base_mint: &str, quote_mint: &str) -> Result<PoolInfo> {
        // Intentar ambas direcciones (SOL/USDC y USDC/SOL)
        let key1 = format!("{}-{}", base_mint, quote_mint);
        let key2 = format!("{}-{}", quote_mint, base_mint);
        
        if let Some(pool) = self.pool_cache.get(&key1) {
            println!("✅ Pool encontrado en cache: {}", pool.name);
            return Ok(pool.clone());
        }
        
        if let Some(pool) = self.pool_cache.get(&key2) {
            println!("✅ Pool encontrado en cache (reversed): {}", pool.name);
            return Ok(pool.clone());
        }
        
        // Si no está en cache, buscar en chain (SLOW PATH)
        println!("⚠️  Pool no está en cache. Buscando on-chain...");
        self.discover_pool_on_chain(base_mint, quote_mint)
    }

    /// Busca un pool en chain usando getProgramAccounts (LENTO - solo para pools nuevos)
    fn discover_pool_on_chain(&self, base_mint: &str, quote_mint: &str) -> Result<PoolInfo> {
        use solana_client::rpc_filter::{RpcFilterType, Memcmp, MemcmpEncodedBytes};
        use solana_client::rpc_config::{RpcProgramAccountsConfig, RpcAccountInfoConfig};
        use solana_account_decoder::UiAccountEncoding;
        
        println!("🔍 Buscando pool on-chain para {}/{}", &base_mint[..8], &quote_mint[..8]);
        
        let base_mint_pubkey = Pubkey::from_str(base_mint)?;
        let quote_mint_pubkey = Pubkey::from_str(quote_mint)?;
        
        // Filtros para buscar pools de Raydium con estos mints específicos
        // Estructura del account de Raydium AMM v4:
        // - Offset 400: coin_mint (base)
        // - Offset 432: pc_mint (quote)
        let filters = vec![
            RpcFilterType::Memcmp(Memcmp::new(
                400,
                MemcmpEncodedBytes::Base58(base_mint_pubkey.to_string()),
            )),
            RpcFilterType::Memcmp(Memcmp::new(
                432,
                MemcmpEncodedBytes::Base58(quote_mint_pubkey.to_string()),
            )),
        ];
        
        let config = RpcProgramAccountsConfig {
            filters: Some(filters),
            account_config: RpcAccountInfoConfig {
                encoding: Some(UiAccountEncoding::Base64),
                commitment: Some(CommitmentConfig::confirmed()),
                ..Default::default()
            },
            with_context: Some(false),
        };
        
        println!("📡 Consultando RPC (esto puede tardar 5-10s)...");
        let accounts = self.rpc_client.get_program_accounts_with_config(
            &self.program_id,
            config,
        )?;
        
        if accounts.is_empty() {
            // Intentar con los mints invertidos
            println!("⚠️  No se encontró pool. Intentando con mints invertidos...");
            let filters_reversed = vec![
                RpcFilterType::Memcmp(Memcmp::new(
                    400,
                    MemcmpEncodedBytes::Base58(quote_mint_pubkey.to_string()),
                )),
                RpcFilterType::Memcmp(Memcmp::new(
                    432,
                    MemcmpEncodedBytes::Base58(base_mint_pubkey.to_string()),
                )),
            ];
            
            let config_reversed = RpcProgramAccountsConfig {
                filters: Some(filters_reversed),
                account_config: RpcAccountInfoConfig {
                    encoding: Some(UiAccountEncoding::Base64),
                    commitment: Some(CommitmentConfig::confirmed()),
                    ..Default::default()
                },
                with_context: Some(false),
            };
            
            let accounts_reversed = self.rpc_client.get_program_accounts_with_config(
                &self.program_id,
                config_reversed,
            )?;
            
            if accounts_reversed.is_empty() {
                anyhow::bail!(
                    "❌ Pool no encontrado on-chain para {}/{}\n\
                     Posibles causas:\n\
                     1. El pool no existe en Raydium (usa Jupiter como fallback)\n\
                     2. El pool está en otro DEX\n\
                     3. Verifica los mints en Solscan",
                    base_mint, quote_mint
                );
            }
            
            // Parsear el primer pool encontrado (invertido)
            return self.parse_pool_account(&accounts_reversed[0].0, &accounts_reversed[0].1, true);
        }
        
        // Parsear el primer pool encontrado
        println!("✅ Pool encontrado on-chain!");
        self.parse_pool_account(&accounts[0].0, &accounts[0].1, false)
    }
    
    /// Parsea un account de pool de Raydium y extrae toda la información necesaria
    fn parse_pool_account(&self, pubkey: &Pubkey, account_data: &solana_sdk::account::Account, reversed: bool) -> Result<PoolInfo> {
        let data = &account_data.data;
        
        if data.len() < 752 {
            anyhow::bail!("Account data demasiado corto para ser un pool de Raydium");
        }
        
        // Función auxiliar para leer un Pubkey desde un offset
        fn read_pubkey(data: &[u8], offset: usize) -> Result<Pubkey> {
            if offset + 32 > data.len() {
                anyhow::bail!("Offset fuera de rango");
            }
            Ok(Pubkey::new_from_array(
                data[offset..offset + 32].try_into().unwrap()
            ))
        }
        
        // Extraer campos del pool según el layout de Raydium AMM v4
        // Referencia: https://github.com/raydium-io/raydium-sdk
        let amm_id = pubkey.to_string();
        let amm_authority = read_pubkey(data, 16)?.to_string();
        let amm_open_orders = read_pubkey(data, 48)?.to_string();
        let lp_mint = read_pubkey(data, 368)?.to_string();
        let coin_mint = read_pubkey(data, 400)?.to_string();
        let pc_mint = read_pubkey(data, 432)?.to_string();
        let coin_vault = read_pubkey(data, 464)?.to_string();
        let pc_vault = read_pubkey(data, 496)?.to_string();
        let serum_market = read_pubkey(data, 176)?.to_string();
        
        // Para las cuentas de Serum, necesitamos consultarlas del market
        println!("🔍 Consultando detalles del Serum Market...");
        let serum_market_pubkey = Pubkey::from_str(&serum_market)?;
        let serum_account = self.rpc_client.get_account(&serum_market_pubkey)?;
        let serum_data = &serum_account.data;
        
        // Extraer cuentas de Serum desde el market account
        let serum_bids = read_pubkey(serum_data, 85 + 32 * 3)?.to_string();
        let serum_asks = read_pubkey(serum_data, 85 + 32 * 4)?.to_string();
        let serum_event_queue = read_pubkey(serum_data, 85 + 32 * 5)?.to_string();
        let serum_coin_vault = read_pubkey(serum_data, 85)?.to_string();
        let serum_pc_vault = read_pubkey(serum_data, 85 + 32)?.to_string();
        let serum_vault_signer = read_pubkey(serum_data, 85 + 32 * 6)?.to_string();
        
        let (base_mint, quote_mint, name) = if reversed {
            (pc_mint.clone(), coin_mint.clone(), format!("DISCOVERED/{}", &coin_mint[..6]))
        } else {
            (coin_mint.clone(), pc_mint.clone(), format!("DISCOVERED/{}", &coin_mint[..6]))
        };
        
        let pool_info = PoolInfo {
            name,
            base_mint,
            quote_mint,
            amm_id,
            amm_authority,
            amm_open_orders,
            coin_vault,
            pc_vault,
            lp_mint,
            serum_market,
            serum_bids,
            serum_asks,
            serum_event_queue,
            serum_coin_vault,
            serum_pc_vault,
            serum_vault_signer,
        };
        
        // Guardar automáticamente en cache para futuras referencias
        println!("💾 Guardando pool descubierto en cache...");
        if let Err(e) = self.save_pool_to_cache(&pool_info) {
            eprintln!("⚠️  No se pudo guardar en cache: {}", e);
        }
        
        Ok(pool_info)
    }
    
    /// Guarda un pool descubierto en el archivo de cache
    fn save_pool_to_cache(&self, pool: &PoolInfo) -> Result<()> {
        let cache_path = "pools_cache.json";
        
        // Leer cache existente o crear uno nuevo
        let mut cache: PoolsCache = if std::path::Path::new(cache_path).exists() {
            let content = fs::read_to_string(cache_path)?;
            serde_json::from_str(&content)?
        } else {
            PoolsCache {
                version: "1.0".to_string(),
                pools: Vec::new(),
            }
        };
        
        // Agregar pool si no existe
        let key = format!("{}-{}", pool.base_mint, pool.quote_mint);
        if !cache.pools.iter().any(|p| format!("{}-{}", p.base_mint, p.quote_mint) == key) {
            cache.pools.push(pool.clone());
            
            // Guardar
            let json = serde_json::to_string_pretty(&cache)?;
            fs::write(cache_path, json)?;
            
            println!("✅ Pool guardado en cache: {}", pool.name);
        }
        
        Ok(())
    }

    /// Construye una instrucción de Swap con todas las cuentas requeridas
    pub fn build_swap_instruction(
        &self,
        pool_keys: &PoolKeys,
        user_source_token_account: Pubkey,
        user_destination_token_account: Pubkey,
        user_owner: Pubkey,
        amount_in: u64,
        min_amount_out: u64,
    ) -> Result<Instruction> {
        // Datos de la instrucción: [discriminator, amount_in, min_amount_out]
        let mut data = Vec::with_capacity(17);
        data.push(SWAP_BASE_IN_DISCRIMINATOR);
        data.extend_from_slice(&amount_in.to_le_bytes());
        data.extend_from_slice(&min_amount_out.to_le_bytes());

        // Cuentas en orden ESTRICTO según el programa de Raydium
        let accounts = vec![
            AccountMeta::new_readonly(spl_token::id(), false),                    // 0. Token Program
            AccountMeta::new(pool_keys.amm_id, false),                             // 1. AMM ID
            AccountMeta::new_readonly(pool_keys.amm_authority, false),             // 2. AMM Authority
            AccountMeta::new(pool_keys.amm_open_orders, false),                    // 3. AMM Open Orders
            AccountMeta::new(pool_keys.coin_vault, false),                         // 4. Pool Coin Vault
            AccountMeta::new(pool_keys.pc_vault, false),                           // 5. Pool PC Vault
            AccountMeta::new_readonly(self.serum_program_id, false),               // 6. Serum Program
            AccountMeta::new(pool_keys.serum_market, false),                       // 7. Serum Market
            AccountMeta::new(pool_keys.serum_bids, false),                         // 8. Serum Bids
            AccountMeta::new(pool_keys.serum_asks, false),                         // 9. Serum Asks
            AccountMeta::new(pool_keys.serum_event_queue, false),                  // 10. Serum Event Queue
            AccountMeta::new(pool_keys.serum_coin_vault, false),                   // 11. Serum Coin Vault
            AccountMeta::new(pool_keys.serum_pc_vault, false),                     // 12. Serum PC Vault
            AccountMeta::new_readonly(pool_keys.serum_vault_signer, false),        // 13. Serum Vault Signer
            AccountMeta::new(user_source_token_account, false),                    // 14. User Source Token Account
            AccountMeta::new(user_destination_token_account, false),               // 15. User Dest Token Account
            AccountMeta::new_readonly(user_owner, true),                           // 16. User Owner (Signer)
        ];

        Ok(Instruction {
            program_id: self.program_id,
            accounts,
            data,
        })
    }

    /// Calcula el min_amount_out basado en slippage
    pub fn calculate_min_amount_out(&self, expected_out: u64, slippage_bps: u16) -> u64 {
        let slippage_multiplier = 1.0 - (slippage_bps as f64 / 10000.0);
        (expected_out as f64 * slippage_multiplier) as u64
    }

    /// Ejecuta un swap completo (construcción + firma + envío)
    pub fn execute_swap(
        &self,
        base_mint: &str,
        quote_mint: &str,
        amount_in: u64,
        min_amount_out: u64,
        user_keypair: &Keypair,
    ) -> Result<String> {
        println!("🚀 Iniciando swap directo en Raydium...");
        println!("   {} → {}", base_mint, quote_mint);
        println!("   Amount In: {}", amount_in);
        println!("   Min Amount Out: {}", min_amount_out);

        // 1. Encontrar pool
        let pool_info = self.find_pool(base_mint, quote_mint)?;
        let pool_keys = pool_info.to_pubkeys()?;

        // 2. Derivar token accounts del usuario
        let base_mint_pubkey = Pubkey::from_str(base_mint)?;
        let quote_mint_pubkey = Pubkey::from_str(quote_mint)?;
        
        let user_source = spl_associated_token_account::get_associated_token_address(
            &user_keypair.pubkey(),
            &base_mint_pubkey,
        );
        
        let user_dest = spl_associated_token_account::get_associated_token_address(
            &user_keypair.pubkey(),
            &quote_mint_pubkey,
        );

        // 3. Construir instrucción
        let swap_ix = self.build_swap_instruction(
            &pool_keys,
            user_source,
            user_dest,
            user_keypair.pubkey(),
            amount_in,
            min_amount_out,
        )?;

        // 4. Construir transacción
        let recent_blockhash = self.rpc_client.get_latest_blockhash()?;
        let transaction = Transaction::new_signed_with_payer(
            &[swap_ix],
            Some(&user_keypair.pubkey()),
            &[user_keypair],
            recent_blockhash,
        );

        // 5. Enviar
        println!("📡 Enviando transacción...");
        let signature = self.rpc_client.send_and_confirm_transaction(&transaction)?;

        println!("✅ Swap ejecutado: {}", signature);
        println!("🔗 https://solscan.io/tx/{}", signature);

        Ok(signature.to_string())
    }

    /// Lista todos los pools en cache
    pub fn list_cached_pools(&self) -> Vec<String> {
        self.pool_cache.values()
            .map(|p| format!("{} ({}/{})", p.name, &p.base_mint[..8], &p.quote_mint[..8]))
            .collect()
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pool_cache_loading() {
        let client = RaydiumClient::new("https://api.mainnet-beta.solana.com".to_string());
        assert!(client.is_ok());
        
        let client = client.unwrap();
        assert!(client.pool_cache.len() > 0);
    }

    #[test]
    fn test_find_sol_usdc_pool() {
        let client = RaydiumClient::new("https://api.mainnet-beta.solana.com".to_string()).unwrap();
        
        let pool = client.find_pool(
            "So11111111111111111111111111111111111111112",
            "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
        );
        
        assert!(pool.is_ok());
        assert_eq!(pool.unwrap().name, "SOL/USDC");
    }

    #[test]
    fn test_calculate_min_amount_out() {
        let client = RaydiumClient::new("https://api.mainnet-beta.solana.com".to_string()).unwrap();
        
        // 1% slippage (100 bps)
        let min_out = client.calculate_min_amount_out(1_000_000, 100);
        assert_eq!(min_out, 990_000); // 1% menos
        
        // 0.5% slippage (50 bps)
        let min_out = client.calculate_min_amount_out(1_000_000, 50);
        assert_eq!(min_out, 995_000);
    }
}
pub struct RaydiumExecutor;
