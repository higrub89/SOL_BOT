use anyhow::Result;
use dotenv::dotenv;
use the_chassis::geyser::{GeyserClient, GeyserConfig};
use tokio::signal;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    
    // Configurar logging
    tracing_subscriber::fmt::init();

    println!("🧪 TEST: Yellowstone Geyser Integration");
    
    // Obtener API Key
    let api_key = std::env::var("HELIUS_API_KEY")
        .expect("HELIUS_API_KEY no encontrada en .env");
        
    // Configurar endpoint de Helius con autenticación
    // Helius usa autenticación via query param o header
    // Para gRPC, usualmente se usa `x-token` en metadata o el endpoint con token.
    // Probaremos con el endpoint directo que incluye el token si es posible, 
    // pero tonic maneja auth mejor via interceptors.
    // Por ahora, usaremos la URL base y pasaremos el token en la config.
    
    let config = GeyserConfig {
        endpoint: "https://mainnet.helius-rpc.com".to_string(),
        token: Some(api_key),
    };

    let client = GeyserClient::new(config);

    // 1. Benchmark de latencia
    println!("\n📊 Midiendo latencia de conexión...");
    match client.benchmark_latency().await {
        Ok(ms) => println!("   ✅ Latencia: {} ms", ms),
        Err(e) => eprintln!("   ❌ Error conectando: {}", e),
    }

    // 2. Suscripción a una cuenta activa (Wrapped SOL)
    // WSOL Mint: So11111111111111111111111111111111111111112
    // Nota: WSOL como cuenta mint no cambia mucho, mejor monitorear un pool de Raydium activo.
    // Pool SOL/USDC Raydium: 58oQChx4yWmvKdwLLZzBi4ChoCcKTk3KA662zndM5f6
    let target_account = "58oQChx4yWmvKdwLLZzBi4ChoCcKTk3KA662zndM5f6"; 
    
    println!("\n📡 Iniciando suscripción a SOL/USDC Pool ({})", target_account);
    println!("   Presiona Ctrl+C para salir.");

    // Lanzar en background
    let _handle = tokio::spawn(async move {
        if let Err(e) = client.subscribe_and_listen(target_account).await {
            eprintln!("❌ Error en suscripción: {}", e);
        }
    });

    // Esperar señal de salida
    signal::ctrl_c().await?;
    println!("\n🛑 Test finalizado.");
    
    Ok(())
}
