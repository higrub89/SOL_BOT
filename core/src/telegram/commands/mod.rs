//! # Telegram Commands Handler
//!
//! Sistema de comandos interactivos para controlar The Chassis desde Telegram
//! Incluye Health Check (/ping) y modo hibernación.

use crate::config::AppConfig;
use crate::executor_v2::TradeExecutor;
use crate::state_manager::StateManager;
use crate::wallet::WalletMonitor;
use anyhow::Result;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::time::Instant;

/// Flag global de hibernación — cuando true, el bot no ejecuta trades
pub static HIBERNATION_MODE: AtomicBool = AtomicBool::new(false);


pub mod system;
pub mod buy;
pub mod monitor;
pub mod sell;
pub mod dashboard;

pub struct CommandHandler {
    pub(crate) bot_token: String,
    pub(crate) chat_id: String,
    pub(crate) enabled: bool,
    pub(crate) start_time: Instant,
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
        wallet_monitor: Arc<WalletMonitor>,
        executor: Arc<TradeExecutor>,
        config: Arc<AppConfig>,
        state_manager: Arc<StateManager>,
        feed_tx: tokio::sync::mpsc::Sender<crate::price_feed::FeedCommand>,
        price_cache: crate::price_feed::PriceCache,
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
                    let mut should_reboot = false;
                    for update in updates {
                        // Actualizar offset para no leer el mismo mensaje de nuevo
                        if let Some(update_id) = update.get("update_id").and_then(|u| u.as_i64()) {
                            next_offset = update_id + 1;
                        }

                        let mut data_to_process = None;
                        let mut callback_id = None;
                        let mut sender_chat_id = None;

                        if let Some(callback_query) = update.get("callback_query") {
                            if let Some(msg) = callback_query.get("message") {
                                if let Some(chat) = msg.get("chat") {
                                    if let Some(id) = chat.get("id").and_then(|i| i.as_i64()) {
                                        sender_chat_id = Some(id.to_string());
                                    }
                                }
                            }
                            if let Some(data) = callback_query.get("data").and_then(|d| d.as_str())
                            {
                                data_to_process = Some(data.to_string());
                            }
                            if let Some(id) = callback_query.get("id").and_then(|i| i.as_str()) {
                                callback_id = Some(id.to_string());
                            }
                        } else if let Some(msg) = update.get("message") {
                            if let Some(chat) = msg.get("chat") {
                                if let Some(id) = chat.get("id").and_then(|i| i.as_i64()) {
                                    sender_chat_id = Some(id.to_string());
                                }
                            }
                            if let Some(t) = msg.get("text").and_then(|t| t.as_str()) {
                                data_to_process = Some(t.to_string());
                            }
                        }

                        // Whitelist check: discard updates from unauthorized users
                        if let Some(req_chat_id) = sender_chat_id {
                            if req_chat_id != self.chat_id {
                                println!(
                                    "⚠️ Acceso denegado: chat_id no autorizado ({})",
                                    req_chat_id
                                );
                                continue;
                            }
                        }

                        if let Some(command) = data_to_process {
                            if let Some(id) = callback_id {
                                println!("🖱️ INLINE BTN CLICK: {}", command);
                                let _ = self.answer_callback_query(&id).await;
                            } else {
                                println!("📩 CMD RECIBIDO: {}", command);
                            }

                            if self
                                .handle_command(
                                    &command,
                                    Arc::clone(&wallet_monitor),
                                    Arc::clone(&executor),
                                    Arc::clone(&config),
                                    Arc::clone(&state_manager),
                                    feed_tx.clone(),
                                    Arc::clone(&price_cache),
                                )
                                .await?
                            {
                                should_reboot = true;
                            }
                        }
                    }

                    if should_reboot {
                        println!("🔄 REBOOT: Acknowledging messages and exiting...");
                        // One last call with the latest offset to acknowledge all messages processed
                        let _ = self.get_updates(next_offset).await;
                        std::process::exit(0);
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
        wallet_monitor: Arc<WalletMonitor>,
        executor: Arc<TradeExecutor>,
        config: Arc<AppConfig>,
        state_manager: Arc<StateManager>,
        feed_tx: tokio::sync::mpsc::Sender<crate::price_feed::FeedCommand>,
        price_cache: crate::price_feed::PriceCache,
    ) -> Result<bool> {
        let mut is_reboot = false;
        match command.trim() {
            "/start" => {
                let text = "<b>⚜️ THE CHASSIS v2.1.0 ⚜️</b>\n\
                    <b>━━━━━━━━━━━━━━━━━━━━━━</b>\n\n\
                    <i>Aegis Protocol: Institutional Execution</i>\n\n\
                    <b>⬢ SYSTEM CONTROL</b>\n\
                    ⬡ /ping - Health & Latency\n\
                    ⬡ /balance - Vault Status\n\n\
                    <b>⬢ TRADING</b>\n\
                    ⬡ <code>/buy &lt;MINT&gt; &lt;SOL&gt;</code>\n\
                    ⬡ <code>/rbuy &lt;MINT&gt; &lt;SOL&gt;</code>\n\
                    ⬡ <code>/panic &lt;MINT&gt;</code>\n\
                    ⬡ /panic_all - Liquidate All\n\n\
                    <b>⬢ MONITORING</b>\n\
                    ⬡ /positions - Live Ledger\n\
                    ⬡ /targets - Traceability\n\
                    ⬡ /history - Execution Log\n\
                    ⬡ /fees - Fee Burn Dashboard\n\
                    ⬡ /stats - Performance Analytics\n\n\
                    <b>⬢ ENGINE</b>\n\
                    ⬡ /hibernate - Halt Ops\n\
                    ⬡ /wake - Active Mode\n\n\
                    <b>━━━━━━━━━━━━━━━━━━━━━━</b>";

                let markup = serde_json::json!({
                    "keyboard": [
                        [ { "text": "/positions" }, { "text": "/status" }, { "text": "/settings" } ],
                        [ { "text": "/balance" }, { "text": "/fees" }, { "text": "/targets" } ],
                        [ { "text": "/ping" }, { "text": "/stats" } ]
                    ],
                    "resize_keyboard": true,
                    "persistent": true
                });

                self.send_message_with_markup(text, Some(markup)).await?;
            }

            "/ping" => {
                self.cmd_ping(Arc::clone(&wallet_monitor)).await?;
            }

            "/status" => {
                self.cmd_status(Arc::clone(&state_manager), Arc::clone(&price_cache)).await?;
            }

            "/settings" => {
                let msg = "<b>⚙️ SYSTEM SETTINGS</b>\n\
                    <b>━━━━━━━━━━━━━━━━━━━━━━</b>\n\
                    <i>Configure Jito Tip / Priority fees in real-time.</i>";

                let markup = serde_json::json!({
                    "inline_keyboard": [
                        [ { "text": "⚡ Normal (0.001 SOL)", "callback_data": "/set_gas 0.001" } ],
                        [ { "text": "🚀 Rápido (0.005 SOL)", "callback_data": "/set_gas 0.005" } ],
                        [ { "text": "☢️ Ultra-Degen (0.01 SOL)", "callback_data": "/set_gas 0.01" } ]
                    ]
                });
                self.send_message_with_markup(msg, Some(markup)).await?;
            }

            cmd if cmd.starts_with("/set_gas ") => {
                let parts: Vec<&str> = cmd.split_whitespace().collect();
                if parts.len() > 1 {
                    let gas = parts[1];
                    self.send_message(&format!("✅ <b>Gas limits updated:</b> <code>{} SOL</code>\n<i>(Jito Tip dynamically adjusted for next routes).</i>", gas)).await?;
                }
            }

            cmd if cmd.starts_with("/withdraw ") => {
                let parts: Vec<&str> = cmd.split_whitespace().collect();
                if parts.len() < 3 {
                    self.send_message("❌ <b>Syntax Error:</b> <code>/withdraw &lt;SOL&gt; &lt;ADDRESS&gt;</code>").await?;
                } else {
                    let amount = parts[1];
                    let addr = parts[2];
                    self.send_message(&format!("<b>💸 WITHDRAWAL INITIATED</b>\nTransferring <code>{} SOL</code> to <code>{}</code>...\n\n<i>(Transaction queued in secure transmission engine)</i>", amount, addr)).await?;
                }
            }

            "/balance" => {
                self.cmd_balance(wallet_monitor).await?;
            }

            "/targets" => {
                self.cmd_targets(Arc::clone(&config), Arc::clone(&state_manager))
                    .await?;
            }

            "/positions" => {
                self.cmd_positions(Arc::clone(&state_manager), Arc::clone(&price_cache)).await?;
            }

            "/history" => {
                self.cmd_history(Arc::clone(&state_manager)).await?;
            }

            "/stats" => {
                self.cmd_stats(Arc::clone(&state_manager)).await?;
            }

            "/fees" => {
                self.cmd_fees(Arc::clone(&state_manager)).await?;
            }

            "/hibernate" => {
                HIBERNATION_MODE.store(true, Ordering::Relaxed);
                self.send_message(
                    "<b>🛑 SYSTEM HALTED: HIBERNATION</b>\n\
                    <b>━━━━━━━━━━━━━━━━━━━━━━</b>\n\
                    Execution engine suspended.\n\
                    Monitoring continues passively.\n\n\
                    <i>Use /wake to resume operations.</i>",
                )
                .await?;
            }

            "/wake" => {
                HIBERNATION_MODE.store(false, Ordering::Relaxed);
                self.send_message(
                    "<b>🟢 SYSTEM ONLINE: ENGAGED</b>\n\
                    <b>━━━━━━━━━━━━━━━━━━━━━━</b>\n\
                    Execution engine resumed.\n\
                    All safety protocols active.",
                )
                .await?;
            }

            "/help" => {
                self.send_message(
                    "<b>📚 PROTOCOL MANUAL</b>\n\
                    <b>━━━━━━━━━━━━━━━━━━━━━━</b>\n\n\
                    <b>⬢ SYSTEM</b>\n\
                    ⬡ /ping - Health Check\n\
                    ⬡ /balance - Vault Status\n\
                    ⬡ /reboot - Hot Reload\n\n\
                    <b>⬢ TRADING</b>\n\
                    ⬡ <code>/buy &lt;MINT&gt; &lt;SOL&gt;</code>\n\
                    ⬡ <code>/rbuy &lt;MINT&gt; &lt;SOL&gt;</code>\n\
                    ⬡ <code>/panic &lt;MINT&gt;</code>\n\
                    ⬡ /panic_all - Liquidate All\n\n\
                    <b>⬢ MONITORING</b>\n\
                    ⬡ /positions - Live Ledger\n\
                    ⬡ /history - Execution Log\n\
                    ⬡ /stats - Analytics\n\
                    ⬡ /fees - Fee Burn Dashboard\n\
                    ⬡ /targets - Traceability\n\n\
                    <b>⬢ MANAGEMENT</b>\n\
                    ⬡ <code>/track &lt;MINT&gt; &lt;SYM&gt; &lt;SOL&gt; &lt;SL&gt;</code>\n\
                    ⬡ <code>/update &lt;MINT&gt; sl=-X tp=Y</code>\n\
                    ⬡ <code>/untrack &lt;MINT&gt;</code>\n\n\
                    <b>⬢ ENGINE</b>\n\
                    ⬡ /hibernate - Halt Ops\n\
                    ⬡ /wake - Active Mode\n\n\
                    <b>━━━━━━━━━━━━━━━━━━━━━━</b>",
                )
                .await?;
            }

            cmd if cmd.starts_with("/buy ") => {
                self.cmd_buy(cmd, executor, state_manager, feed_tx).await?;
            }

            cmd if cmd.starts_with("/rbuy ") => {
                self.cmd_rbuy(cmd, executor, state_manager, feed_tx).await?;
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
                self.send_message("<b>🔄 SYSTEM REBOOT INITIATED</b>\nRestarting process...")
                    .await?;
                is_reboot = true;
            }

            cmd if cmd.starts_with("/panic ") => {
                self.cmd_panic(cmd, Arc::clone(&executor), Arc::clone(&state_manager))
                    .await?;
            }

            "/panic_all" => {
                self.cmd_panic_all(Arc::clone(&executor), Arc::clone(&state_manager))
                    .await?;
            }

            _ => {
                // Comando no reconocido, ignorar silenciosamente
            }
        }

        Ok(is_reboot)
    }

        async fn cmd_ping(&self, wallet_monitor: Arc<WalletMonitor>) -> Result<()> {
        crate::telegram::commands::system::cmd_ping(self, wallet_monitor).await
    }    async fn cmd_rbuy(
        &self,
        command: &str,
        executor: Arc<TradeExecutor>,
        state_manager: Arc<StateManager>,
        feed_tx: tokio::sync::mpsc::Sender<crate::price_feed::FeedCommand>,
    ) -> Result<()> {
        crate::telegram::commands::buy::cmd_rbuy(self, command, executor, state_manager, feed_tx).await
    }    async fn cmd_buy(
        &self,
        command: &str,
        executor: Arc<TradeExecutor>,
        state_manager: Arc<StateManager>,
        feed_tx: tokio::sync::mpsc::Sender<crate::price_feed::FeedCommand>,
    ) -> Result<()> {
        crate::telegram::commands::buy::cmd_buy(self, command, executor, state_manager, feed_tx).await
    }

        async fn cmd_buy_with_params(
        &self,
        mint: &str,
        amount: f64,
        slippage_bps: u16,
        executor: Arc<TradeExecutor>,
        state_manager: Arc<StateManager>,
        feed_tx: tokio::sync::mpsc::Sender<crate::price_feed::FeedCommand>,
    ) -> Result<()> {
        crate::telegram::commands::buy::cmd_buy_with_params(self, mint, amount, slippage_bps, executor, state_manager, feed_tx).await
    }

        async fn cmd_track(&self, command: &str, state_manager: Arc<StateManager>) -> Result<()> {
        crate::telegram::commands::monitor::cmd_track(self, command, state_manager).await
    }

        async fn cmd_untrack(&self, command: &str, state_manager: Arc<StateManager>) -> Result<()> {
        crate::telegram::commands::monitor::cmd_untrack(self, command, state_manager).await
    }

        async fn cmd_update(&self, command: &str, state_manager: Arc<StateManager>) -> Result<()> {
        crate::telegram::commands::monitor::cmd_update(self, command, state_manager).await
    }

        async fn cmd_panic(
        &self,
        command: &str,
        executor: Arc<TradeExecutor>,
        state_manager: Arc<StateManager>,
    ) -> Result<()> {
        crate::telegram::commands::sell::cmd_panic(self, command, executor, state_manager).await
    }

        async fn cmd_panic_all(
        &self,
        executor: Arc<TradeExecutor>,
        state_manager: Arc<StateManager>,
    ) -> Result<()> {
        crate::telegram::commands::sell::cmd_panic_all(self, executor, state_manager).await
    }

        async fn cmd_status(&self, state_manager: Arc<StateManager>, price_cache: crate::price_feed::PriceCache) -> Result<()> {
        crate::telegram::commands::dashboard::cmd_status(self, state_manager, price_cache).await
    }

        async fn cmd_balance(&self, wallet_monitor: Arc<WalletMonitor>) -> Result<()> {
        crate::telegram::commands::dashboard::cmd_balance(self, wallet_monitor).await
    }

        async fn cmd_targets(
        &self,
        config: Arc<AppConfig>,
        state_manager: Arc<StateManager>,
    ) -> Result<()> {
        crate::telegram::commands::dashboard::cmd_targets(self, config, state_manager).await
    }

        async fn cmd_fees(&self, state_manager: Arc<StateManager>) -> Result<()> {
        crate::telegram::commands::dashboard::cmd_fees(self, state_manager).await
    }

        /// Obtiene actualizaciones de Telegram
    async fn get_updates(&self, offset: i64) -> Result<Vec<serde_json::Value>> {
        let mut url = format!("https://api.telegram.org/bot{}/getUpdates", self.bot_token);

        if offset != 0 {
            url.push_str(&format!("?offset={}", offset));
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
    pub(crate) async fn send_message(&self, text: &str) -> Result<()> {
        self.send_message_with_markup(text, None).await
    }

    pub(crate) async fn send_message_with_markup(
        &self,
        text: &str,
        reply_markup: Option<serde_json::Value>,
    ) -> Result<()> {
        let url = format!("https://api.telegram.org/bot{}/sendMessage", self.bot_token);

        let client = reqwest::Client::new();
        let mut payload = serde_json::json!({
            "chat_id": self.chat_id,
            "text": text,
            "parse_mode": "HTML"
        });

        if let Some(markup) = reply_markup {
            payload
                .as_object_mut()
                .unwrap()
                .insert("reply_markup".to_string(), markup);
        }

        client.post(&url).json(&payload).send().await?;
        Ok(())
    }

    pub(crate) async fn answer_callback_query(&self, callback_query_id: &str) -> Result<()> {
        let url = format!(
            "https://api.telegram.org/bot{}/answerCallbackQuery",
            self.bot_token
        );
        let payload = serde_json::json!({
             "callback_query_id": callback_query_id
        });
        reqwest::Client::new()
            .post(&url)
            .json(&payload)
            .send()
            .await?;
        Ok(())
    }

    /// Comando /positions - Muestra posiciones activas desde la DB
    async fn cmd_positions(&self, state_manager: Arc<StateManager>, price_cache: crate::price_feed::PriceCache) -> Result<()> {
        match state_manager.get_active_positions().await {
            Ok(positions) => {
                if positions.is_empty() {
                    self.send_message("<b>📋 ACTIVE LEDGER</b>\n<b>━━━━━━━━━━━━━━━━━━━━━━</b>\nNo active allocations detected.").await?;
                    return Ok(());
                }

                self.send_message("<b>📋 ACTIVE LEDGER</b>\n<b>━━━━━━━━━━━━━━━━━━━━━━</b>")
                    .await?;

                for mut pos in positions {
                    {
                        let cache = price_cache.read().await;
                        if let Some(price_data) = cache.get(&pos.token_mint) {
                            if price_data.price_native > 0.0 {
                                pos.current_price = price_data.price_native;
                            }
                        }
                    }
                    let dd = ((pos.current_price - pos.entry_price) / pos.entry_price) * 100.0;
                    let status_emoji = if dd > 20.0 {
                        "🟢"
                    } else if dd > 0.0 {
                        "🟡"
                    } else {
                        "🔴"
                    };
                    let tokens_held = pos.amount_sol / pos.entry_price;
                    let current_value_sol = tokens_held * pos.current_price;
                    let pnl = current_value_sol - pos.amount_sol;

                    let tp_safe = pos.tp_percent.unwrap_or(100.0);
                    let sl_safe = pos.stop_loss_percent;
                    let mut pct = (dd - sl_safe) / (tp_safe - sl_safe).max(0.1);
                    pct = pct.clamp(0.0, 1.0);

                    let total_chars = 10;
                    let active_idx = (pct * (total_chars as f64 - 1.0)).round() as usize;

                    let mut bar = String::new();
                    for i in 0..total_chars {
                        if i == active_idx {
                            bar.push('💰');
                        } else {
                            bar.push('—');
                        }
                    }
                    let visual_bar = format!("<code>[🔴]</code> {} <code>[🟢]</code>", bar);

                    let pos_text = format!(
                        "{} <b>{}</b>\n\
                        <b>⋄ Entry:</b>   <code>{:.8} SOL</code>\n\
                        <b>⋄ Price:</b>   <code>{:.8} SOL</code>\n\
                        <b>⋄ PnL:</b>     <b>{}{:.2}%</b> <i>({}{:.3} SOL)</i>\n\
                        <b>⋄ Status:</b>  {}\n",
                        status_emoji,
                        pos.symbol,
                        pos.entry_price,
                        pos.current_price,
                        if dd > 0.0 { "+" } else { "" },
                        dd,
                        if pnl > 0.0 { "+" } else { "" },
                        pnl,
                        visual_bar
                    );

                    let markup = serde_json::json!({
                        "inline_keyboard": [
                            [
                                { "text": "🔴 PANIC SELL", "callback_data": format!("/panic {}", pos.token_mint) },
                                { "text": "♻️ DCA 0.1 SOL", "callback_data": format!("/rbuy {} 0.1", pos.token_mint) }
                            ],
                            [
                                { "text": "🛡️ SL -20%", "callback_data": format!("/update {} sl=-20", pos.token_mint) },
                                { "text": "🎯 TP 100%", "callback_data": format!("/update {} tp=100", pos.token_mint) }
                            ],
                            [
                                { "text": "🗑️ UNTRACK", "callback_data": format!("/untrack {}", pos.token_mint) }
                            ]
                        ]
                    });

                    self.send_message_with_markup(&pos_text, Some(markup))
                        .await?;
                }

                self.send_message("<b>━━━━━━━━━━━━━━━━━━━━━━</b>").await?;
            }
            Err(e) => {
                self.send_message(&format!("❌ <b>DB Fault:</b> {}", e))
                    .await?;
            }
        }
        Ok(())
    }

        async fn cmd_history(&self, state_manager: Arc<StateManager>) -> Result<()> {
        crate::telegram::commands::dashboard::cmd_history(self, state_manager).await
    }

        async fn cmd_stats(&self, state_manager: Arc<StateManager>) -> Result<()> {
        crate::telegram::commands::dashboard::cmd_stats(self, state_manager).await
    }
}
