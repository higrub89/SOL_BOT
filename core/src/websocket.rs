//! # WebSocket Real-Time Event Listener
//!
//! Monitoreo de eventos de Pump.fun en tiempo real via WebSocket.
//! Estándar: Calidad Suiza / Alta Frecuencia.
//! Features: Auto-reconnection, Event Detection, Low Latency

use anyhow::{Context, Result};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::time::Duration;
use tokio_tungstenite::{connect_async, tungstenite::Message};

/// Program ID de Pump.fun para monitorear eventos
const PUMP_PROGRAM_ID: &str = "6EF8rrecthR5Dkzy5fG9VGA7zF5rR9WADwpupump";

/// Número máximo de reconexiones antes de pausar
const MAX_RETRIES: u32 = 5;

/// Configuración del WebSocket
pub struct WebSocketConfig {
    pub rpc_url: String,
}

impl WebSocketConfig {
    pub fn from_env() -> Self {
        let api_key = std::env::var("HELIUS_API_KEY")
            .unwrap_or_else(|_| "1d8b1813-084e-41ed-8e93-87a503c496c6".to_string());
        let ws_url = format!("wss://mainnet.helius-rpc.com/?api-key={}", api_key);
        Self { rpc_url: ws_url }
    }
}

/// Cliente de WebSocket para Solana con reconexión automática
pub struct SolanaWebSocket {
    config: WebSocketConfig,
}

impl SolanaWebSocket {
    pub fn new(config: WebSocketConfig) -> Self {
        Self { config }
    }

    /// Escucha eventos de Pump.fun con reconexión automática
    pub async fn listen_to_pump_events(&self) -> Result<()> {
        let mut retry_count = 0;

        loop {
            match self.connect_and_listen().await {
                Ok(_) => {
                    // Conexión cerrada limpiamente, reconectar
                    println!("⚠️ Conexión cerrada. Reconectando...");
                    retry_count = 0;
                }
                Err(e) => {
                    retry_count += 1;
                    eprintln!(
                        "❌ Error en WebSocket (intento {}/{}): {}",
                        retry_count, MAX_RETRIES, e
                    );

                    if retry_count >= MAX_RETRIES {
                        eprintln!("⛔ Máximo de reintentos alcanzado. Pausando 60s...");
                        tokio::time::sleep(Duration::from_secs(60)).await;
                        retry_count = 0;
                    }
                }
            }

            // Pequeña pausa antes de reconectar
            tokio::time::sleep(Duration::from_secs(2)).await;
            println!("🔄 Reconectando al sensor...\n");
        }
    }

    /// Conexión interna al WebSocket
    async fn connect_and_listen(&self) -> Result<()> {
        println!("🔌 Conectando al Sensor de Red (Pump.fun)...");

        let (ws_stream, _) = connect_async(&self.config.rpc_url)
            .await
            .context("Error conectando a WebSocket")?;

        println!("✅ Telemetría conectada\n");

        let (mut write, mut read) = ws_stream.split();

        // Suscripción a logs
        let subscribe_msg = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "logsSubscribe",
            "params": [
                { "mentions": [PUMP_PROGRAM_ID] },
                { "commitment": "processed" }
            ]
        });

        write
            .send(Message::Text(subscribe_msg.to_string()))
            .await
            .context("Error enviando suscripción")?;

        println!("📡 Escuchando logs del programa Pump.fun...\n");
        println!("╔══════════════════════════════════════════════════════════════╗");
        println!("║              📡 LIVE TELEMETRY - Pump.fun Events             ║");
        println!("╚══════════════════════════════════════════════════════════════╝\n");

        while let Some(msg) = read.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    if let Ok(update) = serde_json::from_str::<LogUpdate>(&text) {
                        self.handle_log_update(update).await;
                    }
                }
                Ok(Message::Ping(_)) => {
                    let _ = write.send(Message::Pong(vec![])).await;
                }
                Ok(Message::Close(_)) => {
                    println!("🔴 Servidor cerró la conexión");
                    break;
                }
                Err(e) => {
                    return Err(anyhow::anyhow!("Error en stream: {}", e));
                }
                _ => {}
            }
        }
        Ok(())
    }

    /// Procesa eventos de logs
    async fn handle_log_update(&self, update: LogUpdate) {
        if let Some(params) = update.params {
            if let Some(result) = params.result {
                let logs = &result.value.logs;
                let sig = &result.value.signature;
                let slot = result.context.slot;

                for log in logs {
                    // Nuevo token creado
                    if log.contains("Program log: Instruction: Create") {
                        println!("✨ [NUEVO TOKEN] Creación detectada!");
                        println!("   Slot: {} | Sig: {}...", slot, &sig[..16]);
                    }

                    // Graduación (migración a Raydium/PumpSwap)
                    if log.contains("Program log: Instruction: Withdraw") {
                        println!("🏁 [GRADUACIÓN] ¡Token migrando a DEX!");
                        println!("   Slot: {} | Sig: {}...", slot, &sig[..16]);
                        println!("   🚀 OPORTUNIDAD DE SNIPE DETECTADA");
                    }

                    // Compra detectada
                    if log.contains("Program log: Instruction: Buy") {
                        println!("🟢 [COMPRA] Actividad de compra detectada");
                    }

                    // Venta detectada
                    if log.contains("Program log: Instruction: Sell") {
                        println!("🔴 [VENTA] Actividad de venta detectada");
                    }
                }
            }
        }
    }
}

/// Estructura para parsear mensajes de WebSocket
#[derive(Debug, Deserialize, Serialize)]
struct LogUpdate {
    params: Option<LogParams>,
}

#[derive(Debug, Deserialize, Serialize)]
struct LogParams {
    result: Option<LogResult>,
}

#[derive(Debug, Deserialize, Serialize)]
struct LogResult {
    context: LogContext,
    value: LogValue,
}

#[derive(Debug, Deserialize, Serialize)]
struct LogContext {
    slot: u64,
}

#[derive(Debug, Deserialize, Serialize)]
struct LogValue {
    signature: String,
    logs: Vec<String>,
}
