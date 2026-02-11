//! # Telegram Commands Handler
//! 
//! Sistema de comandos interactivos para controlar The Chassis desde Telegram
//! Incluye Health Check (/ping) y modo hibernación.

use anyhow::Result;
use std::sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}};
use std::time::Instant;
use crate::emergency::EmergencyMonitor;
use crate::wallet::WalletMonitor;
use crate::config::AppConfig;
use crate::executor_v2::TradeExecutor;
use crate::state_manager::StateManager;
use solana_sdk::signature::Keypair;
use solana_client::rpc_client::RpcClient;

/// Flag global de hibernación — cuando true, el bot no ejecuta trades
pub static HIBERNATION_MODE: AtomicBool = AtomicBool::new(false);

pub struct CommandHandler {
    bot_token: String,
    chat_id: String,
    enabled: bool,
    start_time: Instant,
}

impl CommandHandler {
    pub fn new() -> Self {
        let bot_token = std::env::var("TELEGRAM_BOT_TOKEN").unwrap_or_default();
        let chat_id = std::env::var("TELEGRAM_CHAT_ID").unwrap_or_default();
        
        let enabled = !bot_token.is_empty() && !chat_id.is_empty();
        
        Self {
            bot_token,
            chat_id,
            enabled,
            start_time: Instant::now(),
        }
    }

    /// Verifica si el bot está en modo hibernación
    pub fn is_hibernating() -> bool {
        HIBERNATION_MODE.load(Ordering::Relaxed)
    }

    /// Procesa comandos recibidos del usuario
    pub async fn process_commands(
        &self,
        emergency_monitor: Arc<Mutex<EmergencyMonitor>>,
        wallet_monitor: Arc<WalletMonitor>,
        executor: Arc<TradeExecutor>,
        config: Arc<AppConfig>,
        state_manager: Arc<StateManager>,
    ) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }

        let mut next_offset: i64 = 0;

        loop {
            // Obtener actualizaciones recientes de Telegram usando el offset
            match self.get_updates(next_offset).await {
                Ok(updates) => {
                    for update in updates {
                        // Actualizar offset para no leer el mismo mensaje de nuevo
                        if let Some(update_id) = update.get("update_id").and_then(|u| u.as_i64()) {
                            next_offset = update_id + 1;
                        }

                        if let Some(command) = update.get("message")
                            .and_then(|m| m.get("text"))
                            .and_then(|t| t.as_str()) 
                        {
                            self.handle_command(
                                command,
                                Arc::clone(&emergency_monitor),
                                Arc::clone(&wallet_monitor),
                                Arc::clone(&executor),
                                Arc::clone(&config),
                                Arc::clone(&state_manager),
                            ).await?;
                        }
                    }
                }
                Err(e) => {
                    eprintln!("⚠️  Error obteniendo comandos: {}", e);
                }
            }

            // Esperar un poco antes de volver a chequear
            tokio::time::sleep(std::time::Duration::from_millis(2000)).await;
        }
    }

    /// Maneja comandos individuales
    async fn handle_command(
        &self,
        command: &str,
        emergency_monitor: Arc<Mutex<EmergencyMonitor>>,
        wallet_monitor: Arc<WalletMonitor>,
        executor: Arc<TradeExecutor>,
        config: Arc<AppConfig>,
        state_manager: Arc<StateManager>,
    ) -> Result<()> {
        match command.trim() {
            "/start" => {
                self.send_message("🏎️ **The Chassis Bot v2.0.0**\n\n\
                    ⚡ *Comandos disponibles:*\n\n\
                    🏓 `/ping` - Health check completo\n\
                    💰 `/buy <MINT> <SOL>` - Comprar token\n\
                    📊 `/status` - Estado de posiciones (legacy)\n\
                    📋 `/positions` - Posiciones activas (DB)\n\
                    📜 `/history` - Historial de trades\n\
                    📈 `/stats` - Estadísticas de PnL\n\
                    💵 `/balance` - Balance de wallet\n\
                    🎯 `/targets` - Tokens monitoreados\n\
                    🛑 `/hibernate` - Modo hibernación (detener ejecución)\n\
                    🟢 `/wake` - Salir de hibernación\n\
                    ❓ `/help` - Ver ayuda completa\n\n\
                    _El bot protege tus posiciones 24/7 con Trailing Stop-Loss._").await?;
            }

            "/ping" => {
                self.cmd_ping(Arc::clone(&wallet_monitor)).await?;
            }

            "/status" => {
                self.cmd_status(emergency_monitor).await?;
            }

            "/balance" => {
                self.cmd_balance(wallet_monitor).await?;
            }

            "/targets" => {
                self.cmd_targets(config).await?;
            }

            "/positions" => {
                self.cmd_positions(Arc::clone(&state_manager)).await?;
            }

            "/history" => {
                self.cmd_history(Arc::clone(&state_manager)).await?;
            }

            "/stats" => {
                self.cmd_stats(Arc::clone(&state_manager)).await?;
            }

            "/hibernate" => {
                HIBERNATION_MODE.store(true, Ordering::Relaxed);
                self.send_message("🛑 **MODO HIBERNACIÓN ACTIVADO**\n\n\
                    El bot seguirá monitoreando pero NO ejecutará trades.\n\
                    Usa `/wake` para reactivar.").await?;
            }

            "/wake" => {
                HIBERNATION_MODE.store(false, Ordering::Relaxed);
                self.send_message("🟢 **HIBERNACIÓN DESACTIVADA**\n\n\
                    El bot ha vuelto al modo operativo normal.").await?;
            }

            "/help" => {
                self.send_message("📚 **Ayuda de The Chassis v2.0**\n\n\
                    • 🏓 `/ping` - Health check: RPC, wallet, uptime\n\
                    • 📊 `/status` - Drawdown y SL de cada token (legacy)\n\
                    • 📋 `/positions` - Posiciones activas desde DB\n\
                    • 📜 `/history` - Últimos 10 trades ejecutados\n\
                    • 📈 `/stats` - Estadísticas completas de PnL\n\
                    • 💵 `/balance` - Balance de SOL en tu wallet\n\
                    • 🎯 `/targets` - Lista de tokens monitoreados\n\
                    • 💰 `/buy <MINT> <SOL>` - Compra un token\n\
                    • 🚨 `/panic <MINT>` - Venta de emergencia 100%\n\
                    • 🛑 `/hibernate` - Detener toda ejecución\n\
                    • 🟢 `/wake` - Reactivar ejecución\n\n\
                    El bot monitorea automáticamente tus tokens 24/7.").await?;
            }

            cmd if cmd.starts_with("/buy ") => {
                if Self::is_hibernating() {
                    self.send_message("🛑 Bot en HIBERNACIÓN. Usa `/wake` primero.").await?;
                } else {
                    self.cmd_buy(cmd, Arc::clone(&executor)).await?;
                }
            }

            cmd if cmd.starts_with("/panic ") => {
                self.cmd_panic(cmd, Arc::clone(&executor)).await?;
            }

            _ => {
                // Comando no reconocido, ignorar silenciosamente
            }
        }

        Ok(())
    }

    /// Comando /ping - Health Check institucional
    async fn cmd_ping(&self, wallet_monitor: Arc<WalletMonitor>) -> Result<()> {
        let uptime = self.start_time.elapsed();
        let hours = uptime.as_secs() / 3600;
        let minutes = (uptime.as_secs() % 3600) / 60;
        let secs = uptime.as_secs() % 60;

        // Check RPC
        let rpc_status = if let Ok(api_key) = std::env::var("HELIUS_API_KEY") {
            let rpc_url = format!("https://mainnet.helius-rpc.com/?api-key={}", api_key);
            let start = Instant::now();
            let client = RpcClient::new(rpc_url);
            match client.get_slot() {
                Ok(slot) => {
                    let latency = start.elapsed().as_millis();
                    let quality = if latency < 200 { "🟢" } else if latency < 500 { "🟡" } else { "🔴" };
                    format!("{} Helius RPC: {}ms (Slot: {})", quality, latency, slot)
                }
                Err(e) => format!("🔴 Helius RPC: ERROR ({})", e),
            }
        } else {
            "🔴 Helius RPC: API KEY no configurada".to_string()
        };

        // Check Wallet
        let wallet_status = match wallet_monitor.get_sol_balance() {
            Ok(balance) => {
                let emoji = if balance > 0.1 { "🟢" } else if balance > 0.05 { "🟡" } else { "🔴" };
                format!("{} Wallet: {:.4} SOL", emoji, balance)
            }
            Err(e) => format!("🔴 Wallet: ERROR ({})", e),
        };

        // Hibernation status
        let hibernate_status = if Self::is_hibernating() {
            "🛑 HIBERNANDO"
        } else {
            "🟢 OPERATIVO"
        };

        let response = format!(
            "🏓 **PONG — Health Check**\n\n\
            ⏱ Uptime: {}h {}m {}s\n\
            {}\n\
            {}\n\
            🤖 Estado: {}\n\
            📋 Versión: v2.0.0-alpha",
            hours, minutes, secs,
            rpc_status,
            wallet_status,
            hibernate_status
        );

        self.send_message(&response).await?;
        Ok(())
    }

    /// Comando /buy - Ejecuta una compra de token
    async fn cmd_buy(&self, command: &str, executor: Arc<TradeExecutor>) -> Result<()> {
        let parts: Vec<&str> = command.split_whitespace().collect();
        
        if parts.len() < 3 {
            self.send_message("❌ **Uso:** `/buy <MINT> <SOL>`").await?;
            return Ok(());
        }

        let mint = parts[1];
        let amount: f64 = parts[2].parse().unwrap_or(0.0);

        if amount < 0.01 {
            self.send_message("❌ Mínimo: 0.01 SOL").await?;
            return Ok(());
        }

        self.send_message(&format!("🚀 **Iniciando Compra**\nToken: `{}`\nCantidad: `{} SOL`...", mint, amount)).await?;

        // Cargar keypair temporalmente
        let kp_opt = if let Ok(pk) = std::env::var("WALLET_PRIVATE_KEY") {
             Some(Keypair::from_base58_string(&pk))
        } else {
             None 
        };

        // Ejecutar compra
        match executor.execute_buy(mint, kp_opt.as_ref(), amount).await {
            Ok(res) => {
                let msg = format!(
                    "✅ **COMPRA EXITOSA**\n\n💰 {:.4} SOL\n💎 {:.2} Tokens\n🔗 [Solscan](https://solscan.io/tx/{})",
                    res.sol_spent, res.tokens_received, res.signature
                );
                self.send_message(&msg).await?;
            }
            Err(e) => {
                self.send_message(&format!("❌ **Error:** {}", e)).await?;
            }
        }

        Ok(())
    }

    /// Comando /panic - Vende TODO inmediatamente
    async fn cmd_panic(&self, command: &str, executor: Arc<TradeExecutor>) -> Result<()> {
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.len() < 2 {
            self.send_message("❌ **Uso:** `/panic <MINT>`").await?;
            return Ok(());
        }
        
        let mint = parts[1];
        self.send_message(&format!("🚨 **PANIC SELL ACTIVADO**\nVendiendo 100% de `{}`...", mint)).await?;

        let kp_opt = if let Ok(pk) = std::env::var("WALLET_PRIVATE_KEY") {
             Some(Keypair::from_base58_string(&pk))
        } else {
             None 
        };

        match executor.execute_emergency_sell(mint, kp_opt.as_ref(), 100).await {
            Ok(res) => self.send_message(&format!("✅ **VENTA COMPLETADA**\nTx: `{}`", res.signature)).await?,
            Err(e) => self.send_message(&format!("❌ **FALLO CRÍTICO:** {}", e)).await?,
        }

        Ok(())
    }

    /// Comando /status - Muestra el estado de todos los tokens
    async fn cmd_status(&self, emergency_monitor: Arc<Mutex<EmergencyMonitor>>) -> Result<()> {
        let positions = {
            let monitor = emergency_monitor.lock().unwrap();
            monitor.get_all_positions()
        };

        if positions.is_empty() {
            self.send_message("⚠️ No hay posiciones activas").await?;
            return Ok(());
        }

        let mut response = "📊 **STATUS DE POSICIONES**\n\n".to_string();

        for pos in positions {
            let dd = pos.drawdown_percent();
            let status_emoji = if dd > 0.0 { "🟢" } else if dd > -20.0 { "🟡" } else { "🔴" };
            
            response.push_str(&format!(
                "{} **{}**\n\
                └─ Precio: ${:.8}\n\
                └─ Entrada: ${:.8}\n\
                └─ Drawdown: {}{:.2}%\n\
                └─ Valor: {:.4} SOL\n\n",
                status_emoji,
                pos.token_mint,
                pos.current_price,
                pos.entry_price,
                if dd > 0.0 { "+" } else { "" },
                dd,
                pos.current_value
            ));
        }

        self.send_message(&response).await?;
        Ok(())
    }

    /// Comando /balance - Muestra el balance de la wallet
    async fn cmd_balance(&self, wallet_monitor: Arc<WalletMonitor>) -> Result<()> {
        match wallet_monitor.get_sol_balance() {
            Ok(balance) => {
                let message = format!(
                    "💰 **BALANCE DE WALLET**\n\n\
                    SOL: {:.4}\n\
                    USD (aprox): ${:.2}",
                    balance,
                    balance * 100.0 // Aproximación, precio de SOL real requeriría otra API
                );
                self.send_message(&message).await?;
            }
            Err(e) => {
                self.send_message(&format!("❌ Error obteniendo balance: {}", e)).await?;
            }
        }
        Ok(())
    }

    /// Comando /targets - Muestra la lista de tokens monitoreados
    async fn cmd_targets(&self, config: Arc<AppConfig>) -> Result<()> {
        let mut response = "🎯 **TARGETS CONFIGURADOS**\n\n".to_string();

        for target in &config.targets {
            let status = if target.active { "✅ Activo" } else { "⏸️ Pausado" };
            response.push_str(&format!(
                "**{}** ({})\n\
                └─ SL: {:.1}%\n\
                └─ Inversión: {:.4} SOL\n\
                └─ Estado: {}\n\n",
                target.symbol,
                &target.mint[..8],
                target.stop_loss_percent,
                target.amount_sol,
                status
            ));
        }

        response.push_str(&format!(
            "**Configuración Global:**\n\
            └─ Auto-Execute: {}\n\
            └─ Intervalo: {}s",
            if config.global_settings.auto_execute { "🔴 ON" } else { "🟡 OFF" },
            config.global_settings.monitor_interval_sec
        ));

        self.send_message(&response).await?;
        Ok(())
    }

    /// Obtiene actualizaciones de Telegram
    async fn get_updates(&self, offset: i64) -> Result<Vec<serde_json::Value>> {
        let mut url = format!(
            "https://api.telegram.org/bot{}/getUpdates",
            self.bot_token
        );
        
        if offset != 0 {
            url.push_str(&format!("?offset={}", offset));
        } else {
            // Si es el inicio, obtener solo los nuevos (evitar procesar historial viejo)
            url.push_str("?offset=-1");
        }

        let client = reqwest::Client::new();
        let response = client.get(&url).send().await?;
        let data: serde_json::Value = response.json().await?;

        if let Some(result) = data.get("result").and_then(|r| r.as_array()) {
            Ok(result.clone())
        } else {
            Ok(vec![])
        }
    }

    /// Envía un mensaje
    async fn send_message(&self, text: &str) -> Result<()> {
        let url = format!(
            "https://api.telegram.org/bot{}/sendMessage",
            self.bot_token
        );

        let client = reqwest::Client::new();
        let payload = serde_json::json!({
            "chat_id": self.chat_id,
            "text": text,
            "parse_mode": "Markdown"
        });

        client.post(&url).json(&payload).send().await?;
        Ok(())
    }

    /// Comando /positions - Muestra posiciones activas desde la DB
    async fn cmd_positions(&self, state_manager: Arc<StateManager>) -> Result<()> {
        match state_manager.get_active_positions() {
            Ok(positions) => {
                if positions.is_empty() {
                    self.send_message("📋 **POSICIONES ACTIVAS**\n\n⚠️ No hay posiciones activas en la base de datos.").await?;
                    return Ok(());
                }

                let mut response = "📋 **POSICIONES ACTIVAS** (DB Persistente)\n\n".to_string();

                for pos in positions {
                    let dd = ((pos.current_price - pos.entry_price) / pos.entry_price) * 100.0;
                    let status_emoji = if dd > 20.0 { "🟢" } else if dd > 0.0 { "🟡" } else { "🔴" };
                    let tokens_held = pos.amount_sol / pos.entry_price;
                    let current_value_sol = tokens_held * pos.current_price;
                    let pnl = current_value_sol - pos.amount_sol;

                    response.push_str(&format!(
                        "{} **{}**\n\
                        └─ Entrada: ${:.8} ({:.4} SOL)\n\
                        └─ Actual: ${:.8}\n\
                        └─ Tokens: {:.2}\n\
                        └─ Drawdown: {}{:.2}%\n\
                        └─ PnL: {}{:.4} SOL\n\
                        └─ SL: {:.1}%{}\n\n",
                        status_emoji,
                        pos.symbol,
                        pos.entry_price,
                        pos.amount_sol,
                        pos.current_price,
                        tokens_held,
                        if dd > 0.0 { "+" } else { "" },
                        dd,
                        if pnl > 0.0 { "+" } else { "" },
                        pnl,
                        pos.stop_loss_percent,
                        if pos.trailing_enabled { " (Trailing)" } else { "" }
                    ));
                }

                self.send_message(&response).await?;
            }
            Err(e) => {
                self.send_message(&format!("❌ Error obteniendo posiciones: {}", e)).await?;
            }
        }
        Ok(())
    }

    /// Comando /history - Muestra historial de trades (últimos 10)
    async fn cmd_history(&self, state_manager: Arc<StateManager>) -> Result<()> {
        match state_manager.get_trade_history(10) {
            Ok(trades) => {
                if trades.is_empty() {
                    self.send_message("📜 **HISTORIAL DE TRADES**\n\n⚠️ No hay trades registrados todavía.").await?;
                    return Ok(());
                }

                let mut response = "📜 **HISTORIAL DE TRADES** (Últimos 10)\n\n".to_string();

                for trade in trades {
                    let pnl_sol = trade.pnl_sol.unwrap_or(0.0);
                    let pnl_percent = trade.pnl_percent.unwrap_or(0.0);
                    
                    let pnl_emoji = if pnl_sol > 0.0 { "🟢" } else { "🔴" };
                    let timestamp = chrono::DateTime::<chrono::Utc>::from_timestamp(trade.timestamp, 0)
                        .map(|dt| dt.format("%m/%d %H:%M").to_string())
                        .unwrap_or_else(|| "N/A".to_string());

                    response.push_str(&format!(
                        "{} **{}** ({})\n\
                        └─ Tipo: {}\n\
                        └─ Precio: ${:.8}\n\
                        └─ PnL: {}{:.4} SOL ({}{:.2}%)\n\
                        └─ Tx: `{}`\n\n",
                        pnl_emoji,
                        trade.symbol,
                        timestamp,
                        trade.trade_type,
                        trade.price,
                        if pnl_sol > 0.0 { "+" } else { "" },
                        pnl_sol,
                        if pnl_percent > 0.0 { "+" } else { "" },
                        pnl_percent,
                        &trade.signature[..8]
                    ));
                }

                self.send_message(&response).await?;
            }
            Err(e) => {
                self.send_message(&format!("❌ Error obteniendo historial: {}", e)).await?;
            }
        }
        Ok(())
    }

    /// Comando /stats - Muestra estadísticas completas
    async fn cmd_stats(&self, state_manager: Arc<StateManager>) -> Result<()> {
        match state_manager.get_stats() {
            Ok(stats) => {
                let avg_pnl = if stats.total_trades > 0 {
                    stats.total_pnl_sol / stats.total_trades as f64
                } else {
                    0.0
                };

                let status_emoji = if stats.total_pnl_sol > 0.0 { "🟢" } else if stats.total_pnl_sol == 0.0 { "🟡" } else { "🔴" };

                let response = format!(
                    "📈 **ESTADÍSTICAS COMPLETAS**\n\n\
                    {} **PnL Total:** {}{:.4} SOL\n\
                    📊 **Trades Ejecutados:** {}\n\
                    📋 **Posiciones Activas:** {}\n\
                    📉 **Promedio/Trade:** {}{:.4} SOL\n\n\
                    _Datos desde la inicialización de la base de datos._",
                    status_emoji,
                    if stats.total_pnl_sol > 0.0 { "+" } else { "" },
                    stats.total_pnl_sol,
                    stats.total_trades,
                    stats.active_positions,
                    if avg_pnl > 0.0 { "+" } else { "" },
                    avg_pnl
                );

                self.send_message(&response).await?;
            }
            Err(e) => {
                self.send_message(&format!("❌ Error obteniendo estadísticas: {}", e)).await?;
            }
        }
        Ok(())
    }
}
