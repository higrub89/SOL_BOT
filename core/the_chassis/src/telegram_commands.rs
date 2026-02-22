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
                    <i>Select an operation command:</i>\n\n\
                    ⬡ <b>SYSTEM COMMANDS</b>\n\
                    /ping - Diagnostics & Uptime\n\
                    /balance - Vault Balance\n\n\
                    ⬡ <b>TRADING COMMANDS</b>\n\
                    <code>/buy &lt;MINT&gt; &lt;SOL&gt;</code> - Execute Snipe\n\
                    <code>/panic &lt;MINT&gt;</code> - Emergency Liquidation\n\n\
                    ⬡ <b>MONITORING</b>\n\
                    /positions - Live Ledger\n\
                    /targets - Configured Assets\n\
                    /history - Recent Execution Log\n\
                    /stats - Performance Analytics\n\n\
                    ⬡ <b>CONTROL</b>\n\
                    /hibernate - Halt Execution\n\
                    /wake - Resume Operations\n\n\
                    <b>━━━━━━━━━━━━━━━━━━━━━━</b>\n\
                    <i>Aegis Protocol Active — 24/7 Monitoring</i>").await?;
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
                    <b>━━━━━━━━━━━━━━━━━━━━━━</b>\n\
                    ⬡ /ping - System Health & RPC Latency\n\
                    ⬡ /positions - Live Active Positions (DB)\n\
                    ⬡ /history - Ledger of Last 10 Trades\n\
                    ⬡ /stats - Comprehensive PnL Analytics\n\
                    ⬡ /balance - SOL Balance in Hot Wallet\n\
                    ⬡ /targets - Tracked Asset Configuration\n\
                    ⬡ <code>/buy &lt;MINT&gt; &lt;SOL&gt;</code> - Precision Entry\n\
                    ⬡ <code>/track &lt;MINT&gt; &lt;SYMBOL&gt; &lt;SOL&gt; &lt;SL&gt;</code> - Manual Indexing\n\
                    ⬡ <code>/panic &lt;MINT&gt;</code> - Sell 100% Immediately\n\
                    ⬡ /hibernate - Suspend ALL Trading\n\
                    ⬡ /wake - Re-enable Trading\n\
                    ⬡ /reboot - Hot Reload (Pick up new tracks)\n\n\
                    <b>━━━━━━━━━━━━━━━━━━━━━━</b>").await?;
            }

            cmd if cmd.starts_with("/buy ") => {
                if Self::is_hibernating() {
                    self.send_message("🛑 Bot en HIBERNACIÓN. Usa `/wake` primero.").await?;
                } else {
                    self.cmd_buy(cmd, Arc::clone(&executor), Arc::clone(&state_manager)).await?;
                }
            }

            cmd if cmd.starts_with("/track ") => {
                self.cmd_track(cmd, Arc::clone(&state_manager)).await?;
            }

            "/reboot" => {
                self.send_message("<b>🔄 SYSTEM REBOOT INITIATED</b>\nRestarting process...").await?;
                std::process::exit(0);
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
            "🛑 <b>SUSPENDED</b>"
        } else {
            "🟢 <b>ENGAGED</b>"
        };

        let response = format!(
            "<b>🏓 SYSTEM DIAGNOSTICS</b>\n\
            <b>━━━━━━━━━━━━━━━━━━━━━━</b>\n\
            <b>⬢ Uptime:</b> <code>{}h {}m {}s</code>\n\
            {}\n\
            {}\n\
            <b>⬢ Status:</b> {}\n\
            <b>⬢ Engine:</b> <code>v2.0.0-alpha</code>\n\
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
                    active: true,
                    created_at: chrono::Utc::now().timestamp(),
                    updated_at: chrono::Utc::now().timestamp(),
                };

                let _ = state_manager.upsert_position(&pos);
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
                    active: true,
                    created_at: chrono::Utc::now().timestamp(),
                    updated_at: chrono::Utc::now().timestamp(),
                };

                state_manager.upsert_position(&pos)?;

                self.send_message(&format!(
                    "<b>✅ ASSET TRACKED SUCCESSFULLY</b>\n\
                    <b>━━━━━━━━━━━━━━━━━━━━━━</b>\n\
                    <b>⬢ Symbol:</b> <code>{}</code>\n\
                    <b>⬢ Entry:</b> <code>${:.8}</code>\n\
                    <b>⬢ SL / TP:</b> <code>{}% / {}%</code>\n\
                    <b>━━━━━━━━━━━━━━━━━━━━━━</b>\n\
                    <i>🔄 Use /reboot to activate monitoring.</i>",
                    symbol, price_data.price_usd, sl, tp
                )).await?;
            }
            Err(e) => {
                self.send_message(&format!("❌ <b>Tracking Error:</b> {}", e)).await?;
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

    /// Comando /status - Muestra el estado de todos los tokens
    async fn cmd_status(&self, emergency_monitor: Arc<Mutex<EmergencyMonitor>>) -> Result<()> {
        let positions = {
            let monitor = emergency_monitor.lock().unwrap();
            monitor.get_all_positions()
        };

        if positions.is_empty() {
            self.send_message("<b>⚠️ NO ACTIVE POSITIONS (LEGACY)</b>").await?;
            return Ok(());
        }

        let mut response = "<b>📊 LIVE TRACKING (LEGACY)</b>\n<b>━━━━━━━━━━━━━━━━━━━━━━</b>\n".to_string();

        for pos in positions {
            let dd = pos.drawdown_percent();
            let status_emoji = if dd > 0.0 { "🟢" } else if dd > -20.0 { "🟡" } else { "🔴" };
            
            response.push_str(&format!(
                "{} <b>{}</b>\n\
                <b>⬡ Price:</b> <code>${:.8}</code>\n\
                <b>⬡ Entry:</b> <code>${:.8}</code>\n\
                <b>⬡ Drawdown:</b> <b>{}{:.2}%</b>\n\
                <b>⬡ Value:</b> <code>{:.4} SOL</code>\n\n",
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
                    "<b>🏦 VAULT BALANCE</b>\n\
                    <b>━━━━━━━━━━━━━━━━━━━━━━</b>\n\
                    <b>⬡ SOL:</b> <code>{:.4} SOL</code>\n\
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
    async fn cmd_targets(&self, config: Arc<AppConfig>) -> Result<()> {
        let mut response = "<b>🎯 SECURED TARGETS</b>\n<b>━━━━━━━━━━━━━━━━━━━━━━</b>\n\n".to_string();

        for target in &config.targets {
            let status = if target.active { "✅ ACTIVE" } else { "⏸ INACTIVE" };
            response.push_str(&format!(
                "<b>⬢ {}</b> <code>({})</code>\n\
                <b>⬡ Stop-Loss:</b> <code>{:.1}%</code>\n\
                <b>⬡ Allocation:</b> <code>{:.4} SOL</code>\n\
                <b>⬡ Status:</b> {}\n\n",
                target.symbol,
                &target.mint[..8],
                target.stop_loss_percent,
                target.amount_sol,
                status
            ));
        }

        response.push_str(&format!(
            "<b>━━━━━━━━━━━━━━━━━━━━━━</b>\n\
            <b>⚙️ GLOBAL PREFERENCES</b>\n\
            <b>⬡ Auto-Execute:</b> {}\n\
            <b>⬡ Scan Interval:</b> <code>{}s</code>",
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
        match state_manager.get_active_positions() {
            Ok(positions) => {
                if positions.is_empty() {
                    self.send_message("<b>📋 ACTIVE LEDGER</b>\n<b>━━━━━━━━━━━━━━━━━━━━━━</b>\nNo active allocations detected.").await?;
                    return Ok(());
                }

                let mut response = "<b>📋 ACTIVE LEDGER</b>\n<b>━━━━━━━━━━━━━━━━━━━━━━</b>\n".to_string();

                for pos in positions {
                    let dd = ((pos.current_price - pos.entry_price) / pos.entry_price) * 100.0;
                    let status_emoji = if dd > 20.0 { "🟢" } else if dd > 0.0 { "🟡" } else { "🔴" };
                    let tokens_held = pos.amount_sol / pos.entry_price;
                    let current_value_sol = tokens_held * pos.current_price;
                    let pnl = current_value_sol - pos.amount_sol;

                    response.push_str(&format!(
                        "{} <b>{}</b>\n\
                        <b>⬡ Entry:</b> <code>${:.8}</code> <i>({:.4} SOL)</i>\n\
                        <b>⬡ Current:</b> <code>${:.8}</code>\n\
                        <b>⬡ Yield:</b> <code>{:.2} Tokens</code>\n\
                        <b>⬡ Drawdown:</b> <b>{}{:.2}%</b>\n\
                        <b>⬡ PnL:</b> <b>{}{:.4} SOL</b>\n\
                        <b>⬡ SL:</b> <code>{:.1}%{}</code>\n\n",
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
                        if pos.trailing_enabled { " <i>(Trailing)</i>" } else { "" }
                    ));
                }

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
        match state_manager.get_trade_history(10) {
            Ok(trades) => {
                if trades.is_empty() {
                    self.send_message("<b>📜 TRADE LEDGER</b>\n<b>━━━━━━━━━━━━━━━━━━━━━━</b>\nNo operations recorded.").await?;
                    return Ok(());
                }

                let mut response = "<b>📜 RECENT EXECUTIONS (T-10)</b>\n<b>━━━━━━━━━━━━━━━━━━━━━━</b>\n\n".to_string();

                for trade in trades {
                    let pnl_sol = trade.pnl_sol.unwrap_or(0.0);
                    let pnl_percent = trade.pnl_percent.unwrap_or(0.0);
                    
                    let pnl_emoji = if pnl_sol > 0.0 { "🟢" } else { "🔴" };
                    let timestamp = chrono::DateTime::<chrono::Utc>::from_timestamp(trade.timestamp, 0)
                        .map(|dt| dt.format("%m/%d %H:%M").to_string())
                        .unwrap_or_else(|| "N/A".to_string());

                    response.push_str(&format!(
                        "{} <b>{}</b> <i>({})</i>\n\
                        <b>⬡ Type:</b> {}\n\
                        <b>⬡ Price:</b> <code>${:.8}</code>\n\
                        <b>⬡ PnL:</b> <b>{}{:.4} SOL</b> <i>({}{:.2}%)</i>\n\
                        <b>⬡ Tx:</b> <code>{}</code>\n\n",
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
                self.send_message(&format!("❌ <b>DB Fault:</b> {}", e)).await?;
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
                    "<b>📈 PERFORMANCE ANALYTICS</b>\n\
                    <b>━━━━━━━━━━━━━━━━━━━━━━</b>\n\
                    {} <b>Total PnL:</b> <b>{}{:.4} SOL</b>\n\
                    <b>⬡ Executions:</b> <code>{}</code>\n\
                    <b>⬡ Active Positions:</b> <code>{}</code>\n\
                    <b>⬡ Avg Yield/Trade:</b> <code>{}{:.4} SOL</code>\n\
                    <b>━━━━━━━━━━━━━━━━━━━━━━</b>\n\
                    <i>Source: Persistent Ledger</i>",
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
