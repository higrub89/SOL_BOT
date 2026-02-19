//! # Find Vaults — Descubre vault accounts de un pool Raydium
//!
//! Dado un token mint, busca el pool en Raydium y extrae:
//! - AMM ID (pool account)
//! - Coin Vault (base token reserve)
//! - PC Vault (quote/SOL reserve)
//! - Decimales del token
//!
//! ## Uso:
//! ```bash
//! cargo run --bin find_vaults -- <MINT_ADDRESS>
//! ```
//!
//! ## Ejemplo:
//! ```bash
//! cargo run --bin find_vaults -- 83iBDw3ZpxqJ3pEzrbttr9fGA57tttehDAxoFyR1moon
//! ```

use anyhow::{Result, Context};
use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

const RAYDIUM_AMM_V4: &str = "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8";
const WSOL_MINT: &str = "So11111111111111111111111111111111111111112";

fn main() -> Result<()> {
    dotenv::dotenv().ok();
    
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("╔════════════════════════════════════════════════╗");
        eprintln!("║  🔍 Find Vaults — Pool Discovery Tool         ║");
        eprintln!("╚════════════════════════════════════════════════╝");
        eprintln!();
        eprintln!("Uso: cargo run --bin find_vaults -- <MINT_ADDRESS>");
        eprintln!();
        eprintln!("Ejemplo:");
        eprintln!("  cargo run --bin find_vaults -- 83iBDw3ZpxqJ3pEzrbttr9fGA57tttehDAxoFyR1moon");
        std::process::exit(1);
    }
    
    let token_mint = &args[1];
    
    // Obtener RPC URL
    let api_key = std::env::var("HELIUS_API_KEY")
        .unwrap_or_else(|_| "".to_string());
    
    let rpc_url = if api_key.is_empty() {
        "https://api.mainnet-beta.solana.com".to_string()
    } else {
        format!("https://mainnet.helius-rpc.com/?api-key={}", api_key)
    };
    
    println!("╔════════════════════════════════════════════════╗");
    println!("║  🔍 Find Vaults — Pool Discovery Tool         ║");
    println!("╚════════════════════════════════════════════════╝");
    println!();
    println!("🎯 Token Mint: {}", token_mint);
    println!("🌐 RPC: {}...{}", &rpc_url[..30], &rpc_url[rpc_url.len()-8..]);
    println!();
    
    let rpc = RpcClient::new(&rpc_url);
    
    // Verificar que el mint existe y obtener decimales
    let mint_pubkey = Pubkey::from_str(token_mint)
        .context("Mint address inválida")?;
    
    println!("📡 Consultando token info...");
    let mint_account = rpc.get_account(&mint_pubkey)
        .context("No se pudo obtener la cuenta del mint. ¿Existe el token?")?;
    
    // Parsear decimales del mint (offset 44 en el Mint layout)
    let decimals = if mint_account.data.len() >= 45 {
        mint_account.data[44]
    } else {
        6 // default
    };
    println!("   ✅ Token encontrado | Decimales: {}", decimals);
    
    // Buscar pools de Raydium que contengan este token
    println!();
    println!("🔍 Buscando pools de Raydium V4...");
    println!("   (Esto puede tardar 10-30s, estamos escaneando on-chain)");
    println!();
    
    let raydium_program = Pubkey::from_str(RAYDIUM_AMM_V4)?;
    
    // Usamos getMultipleAccounts si tenemos el pool cacheado,
    // sino getProgramAccounts con filtro por mint
    use solana_client::rpc_config::RpcProgramAccountsConfig;
    use solana_client::rpc_filter::{RpcFilterType, Memcmp, MemcmpEncodedBytes};
    use solana_account_decoder::UiAccountEncoding;
    use solana_client::rpc_config::RpcAccountInfoConfig;
    
    // Filtro 1: Buscar donde coin_mint == token_mint (offset 400)
    let filter_as_coin = RpcProgramAccountsConfig {
        filters: Some(vec![
            RpcFilterType::Memcmp(Memcmp::new(
                400, // coin_mint offset en Raydium AMM V4
                MemcmpEncodedBytes::Base58(token_mint.to_string()),
            )),
            RpcFilterType::DataSize(752),
        ]),
        account_config: RpcAccountInfoConfig {
            encoding: Some(UiAccountEncoding::Base64),
            ..Default::default()
        },
        ..Default::default()
    };
    
    // Filtro 2: Buscar donde pc_mint == token_mint (offset 432) 
    let filter_as_pc = RpcProgramAccountsConfig {
        filters: Some(vec![
            RpcFilterType::Memcmp(Memcmp::new(
                432, // pc_mint offset en Raydium AMM V4
                MemcmpEncodedBytes::Base58(token_mint.to_string()),
            )),
            RpcFilterType::DataSize(752),
        ]),
        account_config: RpcAccountInfoConfig {
            encoding: Some(UiAccountEncoding::Base64),
            ..Default::default()
        },
        ..Default::default()
    };
    
    let mut pools_found = Vec::new();
    
    // Buscar como coin_mint
    match rpc.get_program_accounts_with_config(&raydium_program, filter_as_coin) {
        Ok(accounts) => {
            for (pubkey, account) in &accounts {
                if let Some(pool) = parse_raydium_pool(pubkey, &account.data, false) {
                    pools_found.push(pool);
                }
            }
        }
        Err(e) => eprintln!("   ⚠️  Error buscando como coin_mint: {}", e),
    }
    
    // Buscar como pc_mint
    match rpc.get_program_accounts_with_config(&raydium_program, filter_as_pc) {
        Ok(accounts) => {
            for (pubkey, account) in &accounts {
                if let Some(pool) = parse_raydium_pool(pubkey, &account.data, true) {
                    pools_found.push(pool);
                }
            }
        }
        Err(e) => eprintln!("   ⚠️  Error buscando como pc_mint: {}", e),
    }
    
    if pools_found.is_empty() {
        eprintln!("❌ No se encontraron pools de Raydium V4 para este token.");
        eprintln!("   El token podría estar en Raydium V5, Orca, o Meteora.");
        std::process::exit(1);
    }
    
    // Mostrar pools encontrados
    println!("═══════════════════════════════════════════════════");
    println!("  ✅ Encontrados {} pool(s)", pools_found.len());
    println!("═══════════════════════════════════════════════════");
    
    for (i, pool) in pools_found.iter().enumerate() {
        println!();
        println!("┌─── Pool #{} ───────────────────────────────────┐", i + 1);
        println!("│ AMM ID:      {}", pool.amm_id);
        println!("│ Coin Mint:   {}", pool.coin_mint);
        println!("│ PC Mint:     {}", pool.pc_mint);
        println!("│ Coin Vault:  {}", pool.coin_vault);
        println!("│ PC Vault:    {}", pool.pc_vault);
        println!("│ Paired with: {}", if pool.pc_mint == WSOL_MINT { "SOL ✅" } else { &pool.pc_mint[..8] });
        println!("└────────────────────────────────────────────────┘");
        
        // Generar el fragmento JSON para targets.json
        let is_sol_pair = pool.pc_mint == WSOL_MINT || pool.coin_mint == WSOL_MINT;
        
        if is_sol_pair {
            // Determinar cuál vault es SOL y cuál es el token
            let (coin_v, pc_v) = if pool.coin_mint == token_mint.to_string() {
                (&pool.coin_vault, &pool.pc_vault)
            } else {
                (&pool.pc_vault, &pool.coin_vault)
            };
            
            println!();
            println!("📋 Añade esto a tu targets.json:");
            println!("─────────────────────────────────");
            println!(r#"    {{
      "symbol": "TU_SYMBOL",
      "mint": "{}",
      "entry_price": 0.0,
      "amount_sol": 0.025,
      "stop_loss_percent": -60.0,
      "panic_sell_price": 0.0,
      "active": true,
      "pool_account": "{}",
      "coin_vault": "{}",
      "pc_vault": "{}",
      "token_decimals": {},
      "trailing_enabled": true,
      "trailing_distance_percent": 25.0,
      "trailing_activation_threshold": 100.0
    }}"#, token_mint, pool.amm_id, coin_v, pc_v, decimals);
        }
    }
    
    println!();
    println!("✅ ¡Listo! Copia el JSON de arriba a tu targets.json");
    
    Ok(())
}

struct PoolDiscovery {
    amm_id: String,
    coin_mint: String,
    pc_mint: String,
    coin_vault: String,
    pc_vault: String,
}

fn parse_raydium_pool(pubkey: &Pubkey, data: &[u8], _reversed: bool) -> Option<PoolDiscovery> {
    if data.len() < 528 {
        return None;
    }
    
    let read_pubkey = |offset: usize| -> String {
        if offset + 32 > data.len() {
            return "INVALID".to_string();
        }
        Pubkey::new_from_array(
            data[offset..offset + 32].try_into().unwrap()
        ).to_string()
    };
    
    let coin_mint = read_pubkey(400);
    let pc_mint = read_pubkey(432);
    let coin_vault = read_pubkey(464);
    let pc_vault = read_pubkey(496);
    
    Some(PoolDiscovery {
        amm_id: pubkey.to_string(),
        coin_mint,
        pc_mint,
        coin_vault,
        pc_vault,
    })
}
