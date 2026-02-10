//! Test de Raydium Client - Pool Discovery
//! 
//! Uso: cargo run --example raydium_test

use the_chassis::raydium::RaydiumClient;
use dotenv::dotenv;
use std::env;

fn main() -> anyhow::Result<()> {
    dotenv().ok();
    
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║         🏎️  RAYDIUM CLIENT - TEST DE DISCOVERY            ║");
    println!("╚════════════════════════════════════════════════════════════╝\n");

    // Cargar RPC URL
    let api_key = env::var("HELIUS_API_KEY")
        .unwrap_or_else(|_| "demo".to_string());
    let rpc_url = format!("https://mainnet.helius-rpc.com/?api-key={}", api_key);

    // Inicializar cliente
    println!("🔧 Inicializando Raydium Client...\n");
    let client = RaydiumClient::new(rpc_url)?;

    // Listar pools en cache
    println!("📋 Pools disponibles en cache:");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    for pool in client.list_cached_pools() {
        println!("  • {}", pool);
    }
    println!();

    // Test 1: Buscar pool SOL/USDC
    println!("🔍 TEST 1: Buscando pool SOL/USDC...");
    let sol_mint = "So11111111111111111111111111111111111111112";
    let usdc_mint = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";
    
    match client.find_pool(sol_mint, usdc_mint) {
        Ok(pool) => {
            println!("✅ Pool encontrado: {}", pool.name);
            println!("   AMM ID: {}", pool.amm_id);
            println!();
        }
        Err(e) => {
            println!("❌ Error: {}\n", e);
        }
    }

    // Test 2: Buscar pool SOL/USDT
    println!("🔍 TEST 2: Buscando pool SOL/USDT...");
    let usdt_mint = "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB";
    
    match client.find_pool(sol_mint, usdt_mint) {
        Ok(pool) => {
            println!("✅ Pool encontrado: {}", pool.name);
            println!("   AMM ID: {}", pool.amm_id);
            println!();
        }
        Err(e) => {
            println!("❌ Error: {}\n", e);
        }
    }

    // Test 3: Buscar pool inexistente
    println!("🔍 TEST 3: Buscando pool no cacheado (debería fallar)...");
    let fake_mint = "FakeTokenMint1111111111111111111111111111111";
    
    match client.find_pool(sol_mint, fake_mint) {
        Ok(pool) => {
            println!("✅ Pool encontrado: {}", pool.name);
        }
        Err(e) => {
            println!("✅ Error esperado: {}\n", e);
        }
    }

    // Test 4: Calcular min_amount_out
    println!("🔍 TEST 4: Cálculo de slippage...");
    let expected_out = 1_000_000_u64;
    
    let min_1pct = client.calculate_min_amount_out(expected_out, 100); // 1%
    let min_half_pct = client.calculate_min_amount_out(expected_out, 50); // 0.5%
    
    println!("   Expected: {}", expected_out);
    println!("   Min (1% slippage): {}", min_1pct);
    println!("   Min (0.5% slippage): {}", min_half_pct);
    println!();

    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║                  ✅ TESTS COMPLETADOS                       ║");
    println!("╚════════════════════════════════════════════════════════════╝");

    Ok(())
}
