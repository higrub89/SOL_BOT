//! # WebSocket Real-Time Streaming
//! 
//! Monitoreo en tiempo real de cuentas de Solana via WebSocket.
//! Latencia esperada: 50-80ms (vs 150ms HTTP)

use anyhow::{Result, Context};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use std::time::{SystemTime, UNIX_EPOCH};

/// Configuración del WebSocket
pub struct WebSocketConfig {
    pub rpc_url: String,
    pub api_key: String,
}

impl WebSocketConfig {
    pub fn from_env() -> Self {
        let api_key = std::env::var("HELIUS_API_KEY")
            .unwrap_or_else(|_| "1d8b1813-084e-41ed-8e93-87a503c496c6".to_string());
        
        // Helius WebSocket endpoint
        let ws_url = format!("wss://mainnet.helius-rpc.com/?api-key={}", api_key);
        
        Self {
            rpc_url: ws_url,
            api_key,
        }
    }
}

/// Cliente de WebSocket para Solana
pub struct SolanaWebSocket {
    config: WebSocketConfig,
}

impl SolanaWebSocket {
    pub fn new(config: WebSocketConfig) -> Self {
        Self { config }
    }

    /// Conecta al WebSocket y se suscribe a una wallet
    pub async fn subscribe_to_account(&self, pubkey: &str) -> Result<()> {
        println!("🔌 Conectando a Solana WebSocket...");
        println!("   Endpoint: {}", self.config.rpc_url.split('?').next().unwrap());
        println!("   Cuenta:   {}...{}\n", &pubkey[..8], &pubkey[pubkey.len()-4..]);

        // Conectar al WebSocket
        let (ws_stream, _) = connect_async(&self.config.rpc_url)
            .await
            .context("Error conectando a WebSocket")?;

        println!("✅ WebSocket conectado\n");

        let (mut write, mut read) = ws_stream.split();

        // Crear mensaje de suscripción
        let subscribe_msg = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "accountSubscribe",
            "params": [
                pubkey,
                {
                    "encoding": "jsonParsed",
                    "commitment": "confirmed"
                }
            ]
        });

        // Enviar suscripción
        write.send(Message::Text(subscribe_msg.to_string()))
            .await
            .context("Error enviando suscripción")?;

        println!("📡 Suscripción activada. Monitoreando cambios...\n");
        println!("╔════════════════════════════════════════════════════════════╗");
        println!("║              🔴 LIVE STREAM - Account Updates             ║");
        println!("╚════════════════════════════════════════════════════════════╝\n");

        let mut update_count = 0;

        // Leer mensajes del WebSocket
        while let Some(msg) = read.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    match serde_json::from_str::<AccountUpdate>(&text) {
                        Ok(update) => {
                            update_count += 1;
                            self.handle_account_update(update, update_count).await;
                        }
                        Err(_) => {
                            // Podría ser un mensaje de confirmación de suscripción
                            if text.contains("\"result\"") {
                                println!("✅ Confirmación de suscripción recibida\n");
                            }
                        }
                    }
                }
                Ok(Message::Ping(_)) => {
                    // Responder a ping para mantener conexión viva
                    write.send(Message::Pong(vec![]))
                        .await
                        .context("Error enviando pong")?;
                }
                Ok(Message::Close(_)) => {
                    println!("\n🔴 WebSocket cerrado por el servidor");
                    break;
                }
                Err(e) => {
                    eprintln!("❌ Error en WebSocket: {}", e);
                    break;
                }
                _ => {}
            }
        }

        Ok(())
    }

    /// Procesa un update de cuenta
    async fn handle_account_update(&self, update: AccountUpdate, count: usize) {
        let timestamp = Self::now();
        
        println!("┌──────────────────────────────────────────────────────────┐");
        println!("│ UPDATE #{:<3}                     {} │", count, timestamp);
        println!("├──────────────────────────────────────────────────────────┤");
        
        if let Some(params) = update.params {
            if let Some(result) = params.result {
                if let Some(value) = result.value {
                    let lamports = value.lamports.unwrap_or(0);
                    let sol = lamports as f64 / 1_000_000_000.0;
                    
                    println!("│ 💰 Balance: {:.4} SOL ({} lamports)", sol, lamports);
                    println!("│ 🔢 Slot:    {}", result.context.slot);
                    
                    // Detectar cambios
                    let change_type = if lamports > 0 {
                        "🟢 ENTRADA"
                    } else {
                        "🔴 SALIDA"
                    };
                    
                    println!("│ 📊 Tipo:    {}", change_type);
                }
            }
        }
        
        println!("└──────────────────────────────────────────────────────────┘\n");
    }

    fn now() -> String {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap();
        
        let secs = now.as_secs();
        let hours = (secs / 3600) % 24;
        let mins = (secs / 60) % 60;
        let secs = secs % 60;
        
        format!("{:02}:{:02}:{:02} UTC", hours, mins, secs)
    }
}

/// Estructura para parsear mensajes de WebSocket
#[derive(Debug, Deserialize, Serialize)]
struct AccountUpdate {
    jsonrpc: Option<String>,
    method: Option<String>,
    params: Option<UpdateParams>,
}

#[derive(Debug, Deserialize, Serialize)]
struct UpdateParams {
    result: Option<UpdateResult>,
    subscription: Option<u64>,
}

#[derive(Debug, Deserialize, Serialize)]
struct UpdateResult {
    context: UpdateContext,
    value: Option<AccountValue>,
}

#[derive(Debug, Deserialize, Serialize)]
struct UpdateContext {
    slot: u64,
}

#[derive(Debug, Deserialize, Serialize)]
struct AccountValue {
    lamports: Option<u64>,
    owner: Option<String>,
    executable: Option<bool>,
}
