//! # WebSocket Real-Time Event Listener
//!
//! Monitoreo de eventos de Pump.fun en tiempo real via WebSocket.
//! Estándar: Calidad Suiza / Alta Frecuencia.
//! Features: Auto-reconnection, Event Detection, Low Latency

use crate::auto_buyer::{AutoBuyConfig, AutoBuyer};
use anyhow::{Context, Result};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::json;
use solana_sdk::signature::Keypair;
use std::collections::{HashSet, VecDeque};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
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
        let api_key = crate::wallet::get_env_or_secret("HELIUS_API_KEY");
        let ws_url = std::env::var("SOLANA_WS_URL").unwrap_or_else(|_| format!("wss://mainnet.helius-rpc.com/?api-key={}", api_key));
        Self { rpc_url: ws_url }
    }
}

/// Cliente de WebSocket para Solana con reconexión automática e HunterLoop
pub struct SolanaWebSocket {
    config: WebSocketConfig,
    auto_buyer: Arc<AutoBuyer>,
    wallet: Arc<Keypair>,
    seen_mints: Arc<RwLock<HashSet<String>>>,
    mint_queue: Arc<RwLock<VecDeque<String>>>, // Buffer circular para memoria
    last_tokens_timestamps: Arc<RwLock<Vec<Instant>>>,
    devnet_mode: bool,
}

impl SolanaWebSocket {
    pub fn new(
        config: WebSocketConfig,
        auto_buyer: Arc<AutoBuyer>,
        wallet: Arc<Keypair>,
        devnet_mode: bool,
    ) -> Self {
        Self {
            config,
            auto_buyer,
            wallet,
            seen_mints: Arc::new(RwLock::new(HashSet::new())),
            mint_queue: Arc::new(RwLock::new(VecDeque::with_capacity(10000))),
            last_tokens_timestamps: Arc::new(RwLock::new(Vec::new())),
            devnet_mode,
        }
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

    /// Procesa eventos de logs y activa el HunterLoop
    async fn handle_log_update(&self, update: LogUpdate) {
        if let Some(params) = update.params {
            if let Some(result) = params.result {
                let logs = &result.value.logs;
                let sig = &result.value.signature;
                let slot = result.context.slot;

                for log in logs {
                    // 1. Detectar Intención (Creación o Graduación)
                    let is_creation = log.contains("Program log: Instruction: Create");
                    let is_migration = log.contains("Program log: Instruction: Withdraw");

                    if is_creation || is_migration {
                        // 2. Extraer Mint del log (Pump.fun format: "... InitializeMint [MINT] ...")
                        // NOTA: En logs complejos, el mint suele aparecer después de la instrucción.
                        // Como fallback/mejora, buscamos el patrón típico de Pump.fun.
                        if let Some(mint) = self.extract_mint_from_logs(logs) {
                            self.process_candidate_token(mint, is_creation, slot, sig).await;
                        }
                    }
                }
            }
        }
    }

    /// Extrae la dirección del mint de los logs de la instrucción
    fn extract_mint_from_logs(&self, logs: &[String]) -> Option<String> {
        for log in logs {
            // Buscamos strings que parecen direcciones de Solana (.endswith("pump"))
            for word in log.split_whitespace() {
                if word.len() >= 32 && word.ends_with("pump") {
                    return Some(word.to_string());
                }
            }
        }
        None
    }

    /// Orquestador del HunterLoop para un token candidato
    async fn process_candidate_token(&self, mint: String, is_new: bool, slot: u64, sig: &str) {
        // A. DEDUPLICACIÓN
        {
            let seen = self.seen_mints.read().await;
            if seen.contains(&mint) {
                return;
            }
        }

        // B. RATE LIMITING (Max 10 tokens por minuto)
        {
            let mut timestamps = self.last_tokens_timestamps.write().await;
            let now = Instant::now();
            
            // Limpiar timestamps antiguos (> 60s)
            timestamps.retain(|&t| now.duration_since(t) < Duration::from_secs(60));
            
            if timestamps.len() >= 10 {
                println!("⚠️ [HunterLoop] Rate limit alcanzado. Omitiendo mint: {}", mint);
                return;
            }
            
            timestamps.push(now);
        }

        // Registrar como visto con límite de buffer circular (10,000 mints)
        {
            let mut seen = self.seen_mints.write().await;
            let mut queue = self.mint_queue.write().await;

            if !seen.contains(&mint) {
                seen.insert(mint.clone());
                queue.push_back(mint.clone());

                if queue.len() > 10000 {
                    if let Some(oldest) = queue.pop_front() {
                        seen.remove(&oldest);
                    }
                }
            }
        }

        let event_type = if is_new { "✨ CREACIÓN" } else { "🏁 GRADUACIÓN" };
        println!("{} detectada!", event_type);
        println!("   Mint: {}", mint);
        println!("   Slot: {} | Sig: {}...", slot, &sig[..16]);

        // C. EJECUCIÓN ASÍNCRONA (Non-blocking)
        let buyer = Arc::clone(&self.auto_buyer);
        let wallet = Arc::clone(&self.wallet);
        let devnet = self.devnet_mode;
        let mint_clone = mint.clone();

        tokio::spawn(async move {
            let config = AutoBuyConfig {
                token_mint: mint_clone,
                symbol: None,
                amount_sol: 0.02, // Configuración base para HunterLoop
                slippage_bps: 500,  // 5% Slippage en Pump.fun es estándar por volatilidad
                add_to_monitoring: true,
                stop_loss_percent: -50.0,
                trailing_enabled: true,
                fast_mode: true, // HFT mode
            };

            if devnet {
                println!("🧪 [DEVNET] Simulando compra para {}...", mint);
                // En modo devnet podríamos usar un dry_run real si el executor lo soporta
                tokio::time::sleep(Duration::from_secs(1)).await;
                println!("✅ [DEVNET] Simulación completada para {}", mint);
            } else {
                match buyer.buy(&config, &wallet).await {
                    Ok(res) => println!("🎯 [HunterLoop] COMPRA EJECUTADA: {}", res.signature),
                    Err(e) => eprintln!("❌ [HunterLoop] Error procesando {}: {}", mint, e),
                }
            }
        });
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
