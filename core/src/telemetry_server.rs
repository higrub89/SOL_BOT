use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::{accept_async, tungstenite::Message};
use serde::Serialize;
use std::time::{SystemTime, UNIX_EPOCH};
use crate::state_manager::StateManager;
use crate::price_feed::PriceCache;
use crate::telegram::commands::HIBERNATION_MODE;
use std::sync::atomic::Ordering;

#[derive(Serialize, Clone)]
pub struct PositionUpdate {
    pub mint: String,
    pub symbol: String,
    pub entry: f64,
    pub current: f64,
    pub yield_pct: f64,
    pub sl_level: f64,
}

#[derive(Serialize, Clone)]
pub struct TelemetryTick {
    pub t: u64,
    pub net_pnl: f64,
    pub rpc_ping: u64,
    pub status: String,
    pub positions: Vec<PositionUpdate>,
}

pub struct TelemetryServer {
    state_manager: Arc<StateManager>,
    price_cache: PriceCache,
}

impl TelemetryServer {
    pub fn new(state_manager: Arc<StateManager>, price_cache: PriceCache) -> Self {
        Self { state_manager, price_cache }
    }

    pub async fn run(self: Arc<Self>, addr: &str) -> anyhow::Result<()> {
        let listener = TcpListener::bind(addr).await?;
        println!("📡 [TELEMETRY] Server online en ws://{}", addr);

        // Creamos un canal broadcast para enviar los ticks a todas las UIs conectadas
        // Esto evita hacer 1 query por segundo a la BD por *cada* usuario conectado.
        let (tx, _) = tokio::sync::broadcast::channel::<TelemetryTick>(16);
        let tx = Arc::new(tx);

        // Tarea en segundo plano para construir el tick 1 vez por segundo
        let server_clone_for_tick = Arc::clone(&self);
        let tx_tick = Arc::clone(&tx);
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_millis(1000));
            loop {
                interval.tick().await;
                // Optimizacion: Si hay al menos 1 cliente conectado, construimos y enviamos
                if tx_tick.receiver_count() > 0 {
                    match server_clone_for_tick.build_tick().await {
                        Ok(tick) => {
                            let _ = tx_tick.send(tick);
                        }
                        Err(e) => {
                            eprintln!("⚠️ [TELEMETRY] Error construyendo tick: {}", e);
                        }
                    }
                }
            }
        });

        while let Ok((stream, _)) = listener.accept().await {
            let rx = tx.subscribe();
            tokio::spawn(async move {
                if let Err(e) = Self::handle_connection(stream, rx).await {
                    eprintln!("❌ [TELEMETRY] Error en conexión: {}", e);
                }
            });
        }

        Ok(())
    }

    async fn handle_connection(stream: TcpStream, mut rx: tokio::sync::broadcast::Receiver<TelemetryTick>) -> anyhow::Result<()> {
        let mut ws_stream = accept_async(stream).await?;
        println!("✅ [TELEMETRY] UI conectada");

        loop {
            tokio::select! {
                Ok(tick) = rx.recv() => {
                    let json = serde_json::to_string(&tick)?;
                    if let Err(e) = ws_stream.send(Message::Text(json)).await {
                        eprintln!("🔴 [TELEMETRY] UI desconectada: {}", e);
                        break;
                    }
                }
                msg = ws_stream.next() => {
                    if let Some(msg) = msg {
                        match msg {
                            Ok(Message::Text(text)) => {
                                println!("⚡ [TELEMETRY] Mensaje UI: {}", text);
                            }
                            // Manejo de la trama de control para no desconectar clientes lentos / fantasmas
                            Ok(Message::Ping(p)) => {
                                let _ = ws_stream.send(Message::Pong(p)).await;
                            }
                            Ok(Message::Close(_)) | Err(_) => break,
                            _ => {}
                        }
                    } else {
                        break;
                    }
                }
            }
        }

        Ok(())
    }

    async fn build_tick(&self) -> anyhow::Result<TelemetryTick> {
        let active_positions = self.state_manager.get_active_positions().await?;
        let cache = self.price_cache.read().await;

        let mut positions = Vec::new();
        let mut total_pnl_sol = 0.0;

        for pos in active_positions {
            // Obtener precio actual del caché si existe, sino usar el de la DB
            let current_price = cache.get(&pos.token_mint)
                .map(|p| p.price_native)
                .unwrap_or(pos.current_price);

            let yield_pct = if pos.entry_price > 0.0 {
                ((current_price / pos.entry_price) - 1.0) * 100.0
            } else {
                0.0
            };

            // Cálculo aproximado de PnL en SOL
            // NOTA: Para un cálculo de PnL 100% exacto que incluya fees y slippage del DEX,
            // lo ideal a futuro es guardar `token_amount` real obtenido en la DB, en vez de retroceder.
            let tokens_held = if pos.entry_price > 0.0 { pos.amount_sol / pos.entry_price } else { 0.0 };
            let pnl_sol = (current_price - pos.entry_price) * tokens_held;
            total_pnl_sol += pnl_sol;

            positions.push(PositionUpdate {
                mint: pos.token_mint,
                symbol: pos.symbol,
                entry: pos.entry_price,
                current: current_price,
                yield_pct,
                sl_level: pos.stop_loss_percent,
            });
        }

        let is_hibernating = HIBERNATION_MODE.load(Ordering::Relaxed);
        let status = if is_hibernating { "HIBERNATING" } else { "RUNNING" };

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Ok(TelemetryTick {
            t: now,
            net_pnl: total_pnl_sol,
            rpc_ping: 35, // Latencia simulada o real si se tuviera
            status: status.to_string(),
            positions,
        })
    }
}
