//! # Telegram Commands Handler
//! 
//! Sistema de comandos interactivos para controlar The Chassis desde Telegram
//! Incluye Health Check (/ping) y modo hibernación.

use anyhow::Result;
use std::sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}};
use std::time::Instant;
use crate::emergency::EmergencyMonitor;
use crate::wallet::{WalletMonitor, load_keypair_from_env};
use crate::config::AppConfig;
use crate::executor_v2::TradeExecutor;
use crate::state_manager::StateManager;
use solana_client::rpc_client::RpcClient;

/// Flag global de hibernación — cuando true, el bot no ejecuta trades
pub static HIBERNATION_MODE: AtomicBool = AtomicBool::new(false);

pub struct CommandHandler {
    bot_token: String,
    chat_id: String,
    enabled: bool,
    start_time: Instant,
}

impl Default for CommandHandler {
    fn default() -> Self {
        Self::new()
    }
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
    /// Procesa comandos recibidos del usuario
    pub async fn process_commands(
        &self,
        emergency_monitor: Arc<Mutex<EmergencyMonitor>>,
        wallet_monitor: Arc<WalletMonitor>,
        executor: Arc<TradeExecutor>,
        config: Arc<AppConfig>,
        state_manager: Arc<StateManager>,
    ) -> Result<()> {
        println!("🚀 INICIANDO SISTEMA DE TELEGRAM COMMANDS (POLLING MANUAL)...");

        if !self.enabled {
            println!("⚠️ Telegram desactivado (Faltan variables)");
            return Ok(());
        }

        // Test de Conexión Inicial (GetMe) para verificar token
        let token = std::env::var("TELEGRAM_BOT_TOKEN").unwrap_or_default();
        if !token.is_empty() {
             println!("📝 Token detectado: {}...", &token[..5]);
             // Podríamos hacer un reqwest::get("getMe") aquí para validar, 
             // pero el loop de abajo fallará rápido si no hay conexión.
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
                            println!("📩 CMD RECIBIDO: {}", command); // LOGUEAMOS EL COMANDO

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
                    eprintln!("⚠️  Error obteniendo comandos (Polling): {}", e);
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
                self.send_message("<b>⚜️ THE CHASSIS v2.0.0 ⚜️</b>\n\
                    <b>━━━━━━━━━━━━━━━━━━━━━━</b>\n\n\
                    <i>Aegis Protocol: Institutional Execution</i>\n\n\
                    <b>⬢ SYSTEM CONTROL</b>\n\
                    ⬡ /ping - Health & Latency\n\
                    ⬡ /balance - Vault Status\n\n\
                    <b>⬢ TRADING</b>\n\
                    ⬡ <code>/buy &lt;MINT&gt; &lt;SOL&gt;</code>\n\
                    ⬡ <code>/rbuy &lt;MINT&gt; &lt;SOL&gt;</code>
                    ⬡ <code>/panic &lt;MINT&gt;</code>\n\n\
                    <b>⬢ MONITORING</b>\n\
                    ⬡ /positions - Live Ledger\n\
                    ⬡ /targets - Traceability\n\
                    ⬡ /history - Execution Log\n\
                    ⬡ /stats - Performance Analytics\n\n\
                    <b>⬢ ENGINE</b>\n\
                    ⬡ /hibernate - Halt Ops\n\
                    ⬡ /wake - Active Mode\n\n\
                    <b>━━━━━━━━━━━━━━━━━━━━━━</b>").await?;
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
                self.cmd_targets(Arc::clone(&config), Arc::clone(&state_manager)).await?;
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
                self.send_message("<b>🛑 SYSTEM HALTED: HIBERNATION</b>\n\
                    <b>━━━━━━━━━━━━━━━━━━━━━━</b>\n\
                    Execution engine suspended.\n\
                    Monitoring continues passively.\n\n\
                    <i>Use /wake to resume operations.</i>").await?;
            }

            "/wake" => {
                HIBERNATION_MODE.store(false, Ordering::Relaxed);
                self.send_message("<b>🟢 SYSTEM ONLINE: ENGAGED</b>\n\
                    <b>━━━━━━━━━━━━━━━━━━━━━━</b>\n\
                    Execution engine resumed.\n\
                    All safety protocols active.").await?;
            }

            "/help" => {
                self.send_message("<b>📚 PROTOCOL MANUAL</b>\n\
                    <b>━━━━━━━━━━━━━━━━━━━━━━</b>\n\n\
                    <b>⬢ SYSTEM</b>\n\
                    ⬡ /ping - Health Check\n\
                    ⬡ /balance - Vault Status\n\
                    ⬡ /reboot - Hot Reload\n\n\
                    <b>⬢ TRADING</b>\n\
                    ⬡ <code>/buy &lt;MINT&gt; &lt;SOL&gt;</code>\n\
                    ⬡ <code>/panic &lt;MINT&gt;</code>\n\
                    ⬡ /panic_all - Liquidate All\n\n\
                    <b>⬢ MONITORING</b>\n\
                    ⬡ /positions - Live Ledger\n\
                    ⬡ /history - Execution Log\n\
                    ⬡ /stats - Analytics\n\
                    ⬡ /targets - Traceability\n\n\
                    <b>⬢ MANAGEMENT</b>\n\
                    ⬡ <code>/track &lt;MINT&gt; &lt;SYM&gt; &lt;SOL&gt; &lt;SL&gt;</code>\n\
                    ⬡ <code>/update &lt;MINT&gt; sl=-X tp=Y</code>\n\
                    ⬡ <code>/untrack &lt;MINT&gt;</code>\n\n\
                    <b>⬢ ENGINE</b>\n\
                    ⬡ /hibernate - Halt Ops\n\
                    ⬡ /wake - Active Mode\n\n\
                    <b>━━━━━━━━━━━━━━━━━━━━━━</b>").await?;
            }

            cmd if cmd.starts_with("/buy ") => {
                if Self::is_hibernating() {
                    self.send_message("🛑 Bot en HIBERNACIÓN. Usa `/wake` primero.").await?;
                } else {
                    self.cmd_buy(cmd, Arc::clone(&executor), Arc::clone(&state_manager)).await?;
                }
            }

            cmd if cmd.starts_with("/rbuy ") => {
                if Self::is_hibernating() {
                    self.send_message("🛑 Bot en HIBERNACIÓN. Usa `/wake` primero.").await?;
                } else {
                    let parts: Vec<&str> = cmd.split_whitespace().collect();
                    if parts.len() < 3 {
                        self.send_message("❌ <b>Syntax:</b> <code>/rbuy &lt;MINT&gt; &lt;SOL&gt;</code>").await?;
                    } else {
                        let mint = parts[1];
                        let amount: f64 = parts[2].parse().unwrap_or(0.0);
                        self.send_message(&format!("<b>☢️ DEGENERATE RAYDIUM ENTRY</b>\n<b>Asset:</b> <code>{}</code>\n<b>Amount:</b> <code>{} SOL</code>\n<i>Bypassing all guards...</i>", mint, amount)).await?;
                        
                        let kp_opt = crate::wallet::load_keypair_from_env("WALLET_PRIVATE_KEY").ok();
                        match executor.execute_raydium_buy(mint, kp_opt.as_ref(), amount).await {
                            Ok(res) => {
                                self.send_message(&format!("<b>✅ DEGEN SUCCESS</b>\nTx: <a href='https://solscan.io/tx/{}'>VIEW</a>", res.signature)).await?;
                                
                                // ARM MONITORING
                                let pos = crate::state_manager::PositionState {
                                    id: None,
                                    token_mint: mint.to_string(),
                                    symbol: "DEGEN".to_string(),
                                    entry_price: res.price_per_token,
                                    current_price: res.price_per_token,
                                    amount_sol: res.sol_spent,
                                    stop_loss_percent: -50.0,
                                    trailing_enabled: true,
                                    trailing_distance_percent: 25.0,
                                    trailing_activation_threshold: 15.0,
                                    trailing_highest_price: Some(res.price_per_token),
                                    trailing_current_sl: Some(-50.0),
                                    tp_percent: Some(100.0),
                                    tp_amount_percent: Some(50.0),
                                    tp_triggered: false,
                                    tp2_percent: Some(200.0),
                                    tp2_amount_percent: Some(100.0),
                                    tp2_triggered: false,
                                    active: true,
                                    created_at: chrono::Utc::now().timestamp(),
                                    updated_at: chrono::Utc::now().timestamp(),
                                };
                                let _ = state_manager.upsert_position(pos).await;
                                self.send_message("<b>🛡️ MONITORING ARMED</b>\nPosition saved to ledger.").await?;
                            }
                            Err(e) => {
                                self.send_message(&format!("❌ <b>DEGEN FAIL:</b> {}", e)).await?;
                            }
                        }
                    }
                }
            }

            cmd if cmd.starts_with("/track ") => {
                self.cmd_track(cmd, Arc::clone(&state_manager)).await?;
            }

            cmd if cmd.starts_with("/untrack ") => {
                self.cmd_untrack(cmd, Arc::clone(&state_manager)).await?;
            }

            cmd if cmd.starts_with("/update ") => {
                self.cmd_update(cmd, Arc::clone(&state_manager)).await?;
            }

            "/reboot" => {
                self.send_message("<b>🔄 SYSTEM REBOOT INITIATED</b>\nRestarting process...").await?;
                std::process::exit(0);
            }

            cmd if cmd.starts_with("/panic ") => {
                self.cmd_panic(cmd, Arc::clone(&executor)).await?;
            }

            "/panic_all" => {
                self.cmd_panic_all(Arc::clone(&executor), Arc::clone(&state_manager)).await?;
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
            "🛑 <b>SUSPENDED</b>"
        } else {
            "🟢 <b>ENGAGED</b>"
        };

        let response = format!(
            "<b>🏓 SYSTEM DIAGNOSTICS</b>\n\
            <b>━━━━━━━━━━━━━━━━━━━━━━</b>\n\n\
            <b>⋄ Uptime:</b> <code>{}h {}m {}s</code>\n\
            <b>⋄ {}</b>\n\
            <b>⋄ {}</b>\n\
            <b>⋄ Health:</b> {}\n\
            <b>⋄ Engine:</b> <code>v2.0.0-institutional</code>\n\
            <b>⋄ Marker:</b> <code>DIAG_CODE: b08ad</code>\n\n\
            <b>━━━━━━━━━━━━━━━━━━━━━━</b>",
            hours, minutes, secs,
            rpc_status,
            wallet_status,
            hibernate_status
        );

        self.send_message(&response).await?;
        Ok(())
    }

    /// Comando /buy - Ejecuta una compra de token
    async fn cmd_buy(&self, command: &str, executor: Arc<TradeExecutor>, state_manager: Arc<StateManager>) -> Result<()> {
        let parts: Vec<&str> = command.split_whitespace().collect();
        
        if parts.len() < 3 {
            self.send_message("❌ <b>Syntax Error:</b> <code>/buy &lt;MINT&gt; &lt;SOL&gt;</code>").await?;
            return Ok(());
        }

        let mint = parts[1];
        let amount: f64 = parts[2].parse().unwrap_or(0.0);

        if amount < 0.01 {
            self.send_message("❌ <b>Error:</b> Minimum allocation is 0.01 SOL").await?;
            return Ok(());
        }

        self.send_message(&format!("<b>🚀 TACTICAL ENTRY INITIATED</b>\n<b>⬢ Asset:</b> <code>{}</code>\n<b>⬢ Allocation:</b> <code>{} SOL</code>\n<i>Executing payload...</i>", mint, amount)).await?;

        // Cargar keypair temporalmente
        let kp_opt = match load_keypair_from_env("WALLET_PRIVATE_KEY") {
            Ok(kp) => Some(kp),
            Err(e) => {
                self.send_message(&format!("⚠️ <b>Key Vault Error:</b> {}", e)).await?;
                None
            }
        };

        // Ejecutar compra
        match executor.execute_buy(mint, kp_opt.as_ref(), amount).await {
            Ok(res) => {
                let msg = format!(
                    "<b>✅ ACQUISITION SUCCESSFUL</b>\n\
                    <b>━━━━━━━━━━━━━━━━━━━━━━</b>\n\
                    <b>⬡ Cost:</b> <code>{:.4} SOL</code>\n\
                    <b>⬡ Yield:</b> <code>{:.2} Tokens</code>\n\
                    <b>━━━━━━━━━━━━━━━━━━━━━━</b>\n\
                    <a href='https://solscan.io/tx/{}'>[ 🔗 VIEW ON SOLSCAN ]</a>",
                    res.sol_spent, res.tokens_received, res.signature
                );
                self.send_message(&msg).await?;

                // Intentar obtener el símbolo para el monitor
                let scanner = crate::scanner::PriceScanner::new();
                let symbol = match scanner.get_token_price(mint).await {
                    Ok(p) => p.symbol,
                    Err(_) => "BOUGHT".to_string(),
                };

                // Guardar en base de datos para monitoreo automático al reiniciar
                let pos = crate::state_manager::PositionState {
                    id: None,
                    token_mint: mint.to_string(),
                    symbol,
                    entry_price: res.price_per_token,
                    current_price: res.price_per_token,
                    amount_sol: res.sol_spent,
                    stop_loss_percent: -50.0, // Default SL
                    trailing_enabled: true,
                    trailing_distance_percent: 25.0,
                    trailing_activation_threshold: 15.0,
                    trailing_highest_price: Some(res.price_per_token),
                    trailing_current_sl: Some(-50.0),
                    tp_percent: Some(100.0), // Default TP 2x
                    tp_amount_percent: Some(50.0), // Sell 50%
                    tp_triggered: false,
                    tp2_percent: Some(200.0), // TP2 Moonbag
                    tp2_amount_percent: Some(100.0), // Sell remaining
                    tp2_triggered: false,
                    active: true,
                    created_at: chrono::Utc::now().timestamp(),
                    updated_at: chrono::Utc::now().timestamp(),
                };

                let _ = state_manager.upsert_position(pos).await;
                self.send_message("<b>✅ MONITORING ARMED</b>\n<b>TP:</b> +100% (Sell 50%)\nUse /reboot to activate tracking.").await?;
            }
            Err(e) => {
                self.send_message(&format!("❌ <b>Execution Failure:</b> {}", e)).await?;
            }
        }

        Ok(())
    }

    /// Comando /track - Añade un token manualmente al DB para monitoreo
    async fn cmd_track(&self, command: &str, state_manager: Arc<StateManager>) -> Result<()> {
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.len() < 5 {
            self.send_message("❌ <b>Syntax Error:</b> <code>/track &lt;MINT&gt; &lt;SYMBOL&gt; &lt;SOL&gt; &lt;SL&gt; [TP]</code>\nExample: <code>/track 3GEz... SCRAT 0.1 -50 100</code>").await?;
            return Ok(());
        }

        let mint = parts[1];
        let symbol = parts[2];
        let sol: f64 = parts[3].parse().unwrap_or(0.0);
        let sl: f64 = parts[4].parse().unwrap_or(-50.0);
        let tp: f64 = if parts.len() > 5 { parts[5].parse().unwrap_or(100.0) } else { 100.0 };

        self.send_message(&format!("🔍 <b>Indexing Asset: {}...</b>", symbol)).await?;

        let scanner = crate::scanner::PriceScanner::new();
        match scanner.get_token_price(mint).await {
            Ok(price_data) => {
                let pos = crate::state_manager::PositionState {
                    id: None,
                    token_mint: mint.to_string(),
                    symbol: symbol.to_string(),
                    entry_price: price_data.price_usd,
                    current_price: price_data.price_usd,
                    amount_sol: sol,
                    stop_loss_percent: sl,
                    trailing_enabled: true,
                    trailing_distance_percent: 25.0,
                    trailing_activation_threshold: 20.0,
                    trailing_highest_price: Some(price_data.price_usd),
                    trailing_current_sl: Some(sl),
                    tp_percent: Some(tp),
                    tp_amount_percent: Some(50.0), // Default sell 50%
                    tp_triggered: false,
                    tp2_percent: Some(200.0), // TP2 Moonbag default
                    tp2_amount_percent: Some(100.0), // Sell remaining
                    tp2_triggered: false,
                    active: true,
                    created_at: chrono::Utc::now().timestamp(),
                    updated_at: chrono::Utc::now().timestamp(),
                };

                state_manager.upsert_position(pos).await?;

                self.send_message(&format!(
                    "<b>✅ ASSET TRACKED SUCCESSFULLY</b>\n\
                    <b>━━━━━━━━━━━━━━━━━━━━━━</b>\n\
                    <b>⬢ Symbol:</b> <code>{}</code>\n\
                    <b>⬢ Entry:</b> <code>${:.8}</code>\n\
                    <b>⬢ SL / TP:</b> <code>{}% / {}%</code>\n\
                    <b>━━━━━━━━━━━━━━━━━━━━━━</b>\n\
                    <i>🔄 Note: Restart tracking active once PriceFeed refreshes.</i>",
                    symbol, price_data.price_usd, sl, tp
                )).await?;
            }
            Err(e) => {
                self.send_message(&format!("❌ <b>Tracking Error:</b> {}", e)).await?;
            }
        }

        Ok(())
    }

    /// Comando /untrack - Elimina instantáneamente de SQLite y silencia al monitor
    async fn cmd_untrack(&self, command: &str, state_manager: Arc<StateManager>) -> Result<()> {
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.len() < 2 {
            self.send_message("❌ <b>Syntax Error:</b> <code>/untrack &lt;MINT&gt;</code>").await?;
            return Ok(());
        }

        let mint = parts[1];
        match state_manager.close_position(mint).await {
            Ok(_) => {
                self.send_message(&format!("🔴 <b>ASSET UNTRACKED</b>\n<code>{}</code> will no longer trigger trading events.", mint)).await?;
            }
            Err(e) => {
                self.send_message(&format!("❌ <b>DB Fault:</b> {}", e)).await?;
            }
        }
        Ok(())
    }

    /// Comando /update - Hot swap parameters in DB
    async fn cmd_update(&self, command: &str, state_manager: Arc<StateManager>) -> Result<()> {
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.len() < 3 {
            self.send_message("❌ <b>Syntax Error:</b> <code>/update &lt;MINT&gt; sl=-X tp=Y</code>").await?;
            return Ok(());
        }

        let mint = parts[1];
        
        // Fetch current position and mutate in place
        match state_manager.get_position(mint).await {
            Ok(Some(mut pos)) => {
                let mut updated_sl = false;
                let mut updated_tp = false;

                for param in &parts[2..] {
                    if param.starts_with("sl=") {
                        if let Ok(val) = param[3..].parse::<f64>() {
                            pos.stop_loss_percent = val;
                            updated_sl = true;
                        }
                    } else if param.starts_with("tp=") {
                        if let Ok(val) = param[3..].parse::<f64>() {
                            pos.tp_percent = Some(val);
                            updated_tp = true;
                        }
                    }
                }

                if let Err(e) = state_manager.upsert_position(pos.clone()).await {
                    self.send_message(&format!("❌ <b>DB Fault:</b> {}", e)).await?;
                    return Ok(());
                }

                let msg = format!(
                    "⚙️ <b>HOT-SWAP COMPLETE</b> for {}\n\
                     <b>━━━━━━━━━━━━━━━━━━━━━━</b>\n\
                     ⬡ <b>SL:</b> {}\n\
                     ⬡ <b>TP:</b> {}\n\
                     <i>Execution engine updated without reboot.</i>",
                    pos.symbol,
                    if updated_sl { format!("<code>{:.1}%</code> ✅", pos.stop_loss_percent) } else { "Unchanged".to_string() },
                    if updated_tp { format!("<code>{:.1}%</code> ✅", pos.tp_percent.unwrap_or(0.0)) } else { "Unchanged".to_string() }
                );
                self.send_message(&msg).await?;
            }
            Ok(None) => {
                self.send_message(&format!("⚠️ <b>Position Not Found:</b> <code>{}</code>", mint)).await?;
            }
            Err(e) => {
                self.send_message(&format!("❌ <b>DB Fault:</b> {}", e)).await?;
            }
        }

        Ok(())
    }

    /// Comando /panic - Vende TODO inmediatamente
    async fn cmd_panic(&self, command: &str, executor: Arc<TradeExecutor>) -> Result<()> {
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.len() < 2 {
            self.send_message("❌ <b>Syntax Error:</b> <code>/panic &lt;MINT&gt;</code>").await?;
            return Ok(());
        }
        
        let mint = parts[1];
        self.send_message(&format!("<b>🚨 EMERGENCY LIQUIDATION</b>\nLiquidating 100% of <code>{}</code> via Jito...", mint)).await?;

        let kp_opt = match load_keypair_from_env("WALLET_PRIVATE_KEY") {
            Ok(kp) => Some(kp),
            Err(e) => {
                self.send_message(&format!("⚠️ <b>Key Vault Error:</b> {}", e)).await?;
                None
            }
        };

        match executor.execute_emergency_sell(mint, kp_opt.as_ref(), 100).await {
            Ok(res) => self.send_message(&format!("<b>✅ LIQUIDATION COMPLETE</b>\n<b>⬢ Tx:</b> <code>{}</code>", res.signature)).await?,
            Err(e) => self.send_message(&format!("❌ <b>CRITICAL FAILURE:</b> {}", e)).await?,
        }

        Ok(())
    }

    /// Comando /panic_all - Liquida TODAS las posiciones activas en un bundle
    async fn cmd_panic_all(&self, executor: Arc<TradeExecutor>, state_manager: Arc<StateManager>) -> Result<()> {
        self.send_message("<b>🚨 GLOBAL LIQUIDATION INITIATED</b>\nGathering all active positions for Jito Bundling...").await?;

        let active_positions = state_manager.get_active_positions().await?;
        if active_positions.is_empty() {
            self.send_message("<b>⚠️ Aborting:</b> No active positions found to liquidate.").await?;
            return Ok(());
        }

        let mints: Vec<String> = active_positions.iter().map(|p| p.token_mint.clone()).collect();
        let symbols: Vec<String> = active_positions.iter().map(|p| p.symbol.clone()).collect();

        self.send_message(&format!("<b>📦 Bundling Targets:</b> <code>{}</code>\n<i>Optimizing routes...</i>", symbols.join(", "))).await?;

        let kp_opt = match load_keypair_from_env("WALLET_PRIVATE_KEY") {
            Ok(kp) => Some(kp),
            Err(e) => {
                self.send_message(&format!("⚠️ <b>Key Vault Error:</b> {}", e)).await?;
                None
            }
        };

        if let Some(kp) = kp_opt {
            match executor.execute_multi_sell(mints.clone(), &kp, 100).await {
                Ok(results) => {
                    let mut total_sol = 0.0;
                    for res in &results { total_sol += res.output_amount; }
                    
                    self.send_message(&format!(
                        "<b>✅ GLOBAL LIQUIDATION COMPLETE</b>\n\
                        <b>━━━━━━━━━━━━━━━━━━━━━━</b>\n\
                        <b>⬢ Items:</b> <code>{}</code>\n\
                        <b>⬢ Total Yield:</b> <code>{:.4} SOL</code>\n\
                        <b>━━━━━━━━━━━━━━━━━━━━━━</b>\n\
                        <i>All tracked positions have been closed.</i>",
                        results.len(), total_sol
                    )).await?;

                    // Marcar como inactivas en DB
                    for mint in mints {
                        let _ = state_manager.close_position(&mint).await;
                    }
                }
                Err(e) => {
                    self.send_message(&format!("❌ <b>CRITICAL BUNDLE FAILURE:</b> {}", e)).await?;
                }
            }
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
            self.send_message("<b>🛡️ STATUS: NO ACTIVE ALLOCATIONS</b>").await?;
            return Ok(());
        }

        let mut response = "<b>📡 LIVE TELEMETRY</b>\n<b>━━━━━━━━━━━━━━━━━━━━━━</b>\n\n".to_string();

        for pos in positions {
            let dd = pos.drawdown_percent();
            let status_emoji = if dd > 0.0 { "🟢" } else if dd > -20.0 { "🟡" } else { "🔴" };
            
            response.push_str(&format!(
                "{} <b>{}</b>\n\
                <b>⋄ Price:</b>   <code>${:.8}</code>\n\
                <b>⋄ Entry:</b>   <code>${:.8}</code>\n\
                <b>⋄ Yield:</b>   <b>{}{:.2}%</b>\n\
                <b>⋄ Value:</b>   <code>{:.3} SOL</code>\n\n",
                status_emoji,
                &pos.token_mint[..8],
                pos.current_price,
                pos.entry_price,
                if dd > 0.0 { "+" } else { "" },
                dd,
                pos.current_value
            ));
        }

        response.push_str("<b>━━━━━━━━━━━━━━━━━━━━━━</b>");
        self.send_message(&response).await?;
        Ok(())
    }

    /// Comando /balance - Muestra el balance de la wallet
    async fn cmd_balance(&self, wallet_monitor: Arc<WalletMonitor>) -> Result<()> {
        match wallet_monitor.get_sol_balance() {
            Ok(balance) => {
                let message = format!(
                    "<b>🏦 VAULT STATUS</b>\n\
                    <b>━━━━━━━━━━━━━━━━━━━━━━</b>\n\n\
                    <b>⋄ Allocation:</b> <code>{:.4} SOL</code>\n\n\
                    <b>━━━━━━━━━━━━━━━━━━━━━━</b>",
                    balance
                );
                self.send_message(&message).await?;
            }
            Err(e) => {
                self.send_message(&format!("❌ <b>Vault Error:</b> {}", e)).await?;
            }
        }
        Ok(())
    }

    /// Comando /targets - Muestra la lista de tokens monitoreados
    async fn cmd_targets(&self, config: Arc<AppConfig>, state_manager: Arc<StateManager>) -> Result<()> {
        let mut response = "<b>🎯 SECURED TARGETS (DB)</b>\n<b>━━━━━━━━━━━━━━━━━━━━━━</b>\n\n".to_string();

        if let Ok(db_positions) = state_manager.get_active_positions().await {
            if db_positions.is_empty() {
                response.push_str("<i>No assets indexed.</i>\n\n");
            } else {
                for target in db_positions {
                    let status = if target.active { "✅ ACTIVE" } else { "⏸ INACTIVE" };
                    response.push_str(&format!(
                        "<b>⬢ {}</b> <code>({})</code>\n\
                        <b>⋄ Limits:</b> <code>{:.0}% / {:.0}%</code>\n\
                        <b>⋄ Entry:</b>  <code>{:.3} SOL</code>\n\
                        <b>⋄ Status:</b> {}\n\n",
                        target.symbol,
                        &target.token_mint[..8],
                        target.stop_loss_percent,
                        target.tp_percent.unwrap_or(100.0),
                        target.amount_sol,
                        status
                    ));
                }
            }
        }

        response.push_str(&format!(
            "<b>━━━━━━━━━━━━━━━━━━━━━━</b>\n\n\
            <b>⚙️ PREFERENCES</b>\n\
            <b>⋄ Execution:</b> {}\n\
            <b>⋄ Heartbeat:</b> <code>{}s</code>",
            if config.global_settings.auto_execute { "🔴 ARMED" } else { "🟡 DRY-RUN" },
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

    /// Envía un mensaje en HTML
    async fn send_message(&self, text: &str) -> Result<()> {
        let url = format!(
            "https://api.telegram.org/bot{}/sendMessage",
            self.bot_token
        );

        let client = reqwest::Client::new();
        let payload = serde_json::json!({
            "chat_id": self.chat_id,
            "text": text,
            "parse_mode": "HTML"
        });

        client.post(&url).json(&payload).send().await?;
        Ok(())
    }

    /// Comando /positions - Muestra posiciones activas desde la DB
    async fn cmd_positions(&self, state_manager: Arc<StateManager>) -> Result<()> {
        match state_manager.get_active_positions().await {
            Ok(positions) => {
                if positions.is_empty() {
                    self.send_message("<b>📋 ACTIVE LEDGER</b>\n<b>━━━━━━━━━━━━━━━━━━━━━━</b>\nNo active allocations detected.").await?;
                    return Ok(());
                }

                let mut response = "<b>📋 ACTIVE LEDGER</b>\n<b>━━━━━━━━━━━━━━━━━━━━━━</b>\n\n".to_string();

                for pos in positions {
                    let dd = ((pos.current_price - pos.entry_price) / pos.entry_price) * 100.0;
                    let status_emoji = if dd > 20.0 { "🟢" } else if dd > 0.0 { "🟡" } else { "🔴" };
                    let tokens_held = pos.amount_sol / pos.entry_price;
                    let current_value_sol = tokens_held * pos.current_price;
                    let pnl = current_value_sol - pos.amount_sol;

                    response.push_str(&format!(
                        "{} <b>{}</b>\n\
                        <b>⋄ Entry:</b>   <code>${:.8}</code>\n\
                        <b>⋄ Price:</b>   <code>${:.8}</code>\n\
                        <b>⋄ PnL:</b>     <b>{}{:.2}%</b> <i>({}{:.3} SOL)</i>\n\
                        <b>⋄ SL / TP:</b> <code>{:.0}% / {:.0}%</code>\n\n",
                        status_emoji,
                        pos.symbol,
                        pos.entry_price,
                        pos.current_price,
                        if dd > 0.0 { "+" } else { "" },
                        dd,
                        if pnl > 0.0 { "+" } else { "" },
                        pnl,
                        pos.stop_loss_percent,
                        pos.tp_percent.unwrap_or(100.0)
                    ));
                }

                response.push_str("<b>━━━━━━━━━━━━━━━━━━━━━━</b>");
                self.send_message(&response).await?;
            }
            Err(e) => {
                self.send_message(&format!("❌ <b>DB Fault:</b> {}", e)).await?;
            }
        }
        Ok(())
    }

    /// Comando /history - Muestra historial de trades (últimos 10)
    async fn cmd_history(&self, state_manager: Arc<StateManager>) -> Result<()> {
        match state_manager.get_trade_history(10).await {
            Ok(trades) => {
                if trades.is_empty() {
                    self.send_message("<b>📜 TRADE LEDGER</b>\n<b>━━━━━━━━━━━━━━━━━━━━━━</b>\nNo operations recorded.").await?;
                    return Ok(());
                }

                let mut response = "<b>📜 RECENT EXECUTIONS</b>\n<b>━━━━━━━━━━━━━━━━━━━━━━</b>\n\n".to_string();

                for trade in trades {
                    let pnl_sol = trade.pnl_sol.unwrap_or(0.0);
                    let pnl_percent = trade.pnl_percent.unwrap_or(0.0);
                    
                    let pnl_emoji = if pnl_sol > 0.0 { "🟢" } else { "🔴" };
                    let timestamp = chrono::DateTime::<chrono::Utc>::from_timestamp(trade.timestamp, 0)
                        .map(|dt| dt.format("%H:%M %m/%d").to_string())
                        .unwrap_or_else(|| "N/A".to_string());

                    response.push_str(&format!(
                        "{} <b>{}</b> <i>({})</i>\n\
                        <b>⋄ Type:</b>   {}\n\
                        <b>⋄ PnL:</b>    <b>{}{:.3} SOL</b> <i>({:+.1}%)</i>\n\n",
                        pnl_emoji,
                        trade.symbol,
                        timestamp,
                        trade.trade_type,
                        if pnl_sol > 0.0 { "+" } else { "" },
                        pnl_sol,
                        pnl_percent,
                    ));
                }

                response.push_str("<b>━━━━━━━━━━━━━━━━━━━━━━</b>");
                self.send_message(&response).await?;
            }
            Err(e) => {
                self.send_message(&format!("❌ <b>DB Fault:</b> {}", e)).await?;
            }
        }
        Ok(())
    }

    /// Comando /stats - Muestra estadísticas completas
    async fn cmd_stats(&self, state_manager: Arc<StateManager>) -> Result<()> {
        match state_manager.get_stats().await {
            Ok(stats) => {
                let avg_pnl = if stats.total_trades > 0 {
                    stats.total_pnl_sol / stats.total_trades as f64
                } else {
                    0.0
                };

                let status_emoji = if stats.total_pnl_sol > 0.0 { "🟢" } else if stats.total_pnl_sol == 0.0 { "🟡" } else { "🔴" };

                let response = format!(
                    "<b>📈 PERFORMANCE METRICS</b>\n\
                    <b>━━━━━━━━━━━━━━━━━━━━━━</b>\n\n\
                    {} <b>Net Yield:</b> <b>{}{:.4} SOL</b>\n\
                    <b>⋄ Scalps:</b>    <code>{}</code>\n\
                    <b>⋄ Active:</b>    <code>{}</code>\n\
                    <b>⋄ Avg/Pos:</b>   <code>{}{:.4} SOL</code>\n\n\
                    <b>━━━━━━━━━━━━━━━━━━━━━━</b>",
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
                self.send_message(&format!("❌ <b>DB Fault:</b> {}", e)).await?;
            }
        }
        Ok(())
    }
}
