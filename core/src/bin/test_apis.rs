use anyhow::Result;
use dotenvy::dotenv;
use the_chassis::telegram::TelegramNotifier;
use the_chassis::wallet::{get_env_or_secret, load_keypair_secure};
use solana_sdk::signer::Signer;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    
    println!("🔍 Iniciando diagnóstico de APIs y Conexiones...\n");

    // 1. Helius API
    match get_env_or_secret("HELIUS_API_KEY") {
        Ok(k) if !k.is_empty() => println!("✅ HELIUS_API_KEY: Configurada correctamente ({} caracteres).", k.len()),
        Ok(_) => println!("❌ HELIUS_API_KEY: Está vacía."),
        Err(e) => println!("❌ HELIUS_API_KEY: Error al cargar - {}", e),
    }

    // 2. Wallet Address
    match get_env_or_secret("WALLET_ADDRESS") {
        Ok(k) if !k.is_empty() => println!("✅ WALLET_ADDRESS: Configurada correctamente."),
        Ok(_) => println!("❌ WALLET_ADDRESS: Está vacía."),
        Err(e) => println!("❌ WALLET_ADDRESS: Error al cargar - {}", e),
    }

    // 3. Private Key
    match load_keypair_secure("WALLET_PRIVATE_KEY") {
        Ok(kp) => println!("✅ WALLET_PRIVATE_KEY: Clave cargada y descifrada exitosamente (Pubkey: {}).", kp.pubkey()),
        Err(e) => println!("❌ WALLET_PRIVATE_KEY: Error al desencriptar o cargar - {}", e),
    }

    // 4. Telegram
    match TelegramNotifier::new() {
        Ok(tg) => {
            if tg.is_enabled() {
                println!("✅ Telegram Notifier: Inicializado y habilitado.");
                match tg.send_status_update("🔍 Test de conectividad (Bot Trading): Enlaces operativos.").await {
                    Ok(_) => println!("✅ Telegram API: Mensaje de confirmación enviado al chat de configurado exitosamente."),
                    Err(e) => println!("❌ Telegram API: Error al enviar mensaje - {}", e),
                }
            } else {
                println!("⚠️  Telegram Notifier: Deshabilitado (Falta TELEGRAM_BOT_TOKEN o TELEGRAM_CHAT_ID).");
            }
        },
        Err(e) => println!("❌ Telegram Notifier: Error al inicializar - {}", e),
    }

    println!("\n✅ Diagnóstico finalizado.");
    Ok(())
}
