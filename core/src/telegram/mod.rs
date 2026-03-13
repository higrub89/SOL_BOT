//! # Telegram Notifications Module
//!
//! Módulo para enviar alertas críticas vía Telegram cuando:
//! - Se activa un Stop-Loss
//! - Se ejecuta (o intenta ejecutar) una venta de emergencia
//! - Hay errores críticos del sistema

use anyhow::Result;
use reqwest;
use serde_json::json;

pub struct TelegramNotifier {
    bot_token: String,
    chat_id: String,
    enabled: bool,
}

impl Default for TelegramNotifier {
    fn default() -> Self {
        Self::new()
    }
}

impl TelegramNotifier {
    /// Crea un nuevo notificador de Telegram
    pub fn new() -> Self {
        // Usamos el fetcher seguro que cae a Secret Manager
        let bot_token = crate::wallet::get_env_or_secret("TELEGRAM_BOT_TOKEN");
        let chat_id = crate::wallet::get_env_or_secret("TELEGRAM_CHAT_ID");

        let enabled = !bot_token.is_empty() && !chat_id.is_empty();

        if enabled {
            println!("📱 Telegram Notifier: ACTIVADO");
            println!("   • Chat ID: {}", chat_id);
        } else {
            println!("📱 Telegram Notifier: DESACTIVADO (configura TELEGRAM_BOT_TOKEN y TELEGRAM_CHAT_ID)");
        }

        Self {
            bot_token,
            chat_id,
            enabled,
        }
    }

    /// Envía una alerta de Stop-Loss activado
    pub async fn send_stop_loss_alert(
        &self,
        symbol: &str,
        current_price: f64,
        entry_price: f64,
        drawdown: f64,
        stop_loss_limit: f64,
        url: &str,
    ) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }

        let message = format!(
            "<b>🚨 EMERGENCY PROTOCOL ACTIVATED</b>\n\
            <b>━━━━━━━━━━━━━━━━━━━━━━</b>\n\
            <b>⬢ Token:</b> <code>{}</code>\n\
            <b>⬡ Current Price:</b> <code>${:.8}</code>\n\
            <b>⬡ Entry Price:</b> <code>${:.8}</code>\n\
            <b>📉 Drawdown:</b> <b>{:.2}%</b>\n\
            <b>🛑 SL Limit:</b> {:.1}%\n\
            <b>━━━━━━━━━━━━━━━━━━━━━━</b>\n\
            ⚡ <b>MANUAL ACTION REQUIRED</b>\n\
            <a href='{}'>[ 💎 EXECUTE SELL VIA JUPITER ]</a>\n\n\
            <i>🕰 {}</i>",
            symbol,
            current_price,
            entry_price,
            drawdown,
            stop_loss_limit,
            url,
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
        );

        self.send_message(&message, true).await
    }

    /// Envía una alerta cuando se ejecuta una venta automática
    pub async fn send_auto_sell_executed(
        &self,
        symbol: &str,
        price: f64,
        amount_sol: f64,
    ) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }

        let message = format!(
            "<b>⚜️ AUTO-SELL EXECUTED</b>\n\
            <b>━━━━━━━━━━━━━━━━━━━━━━</b>\n\
            <b>⬢ Token:</b> <code>{}</code>\n\
            <b>💎 Transact Price:</b> <code>${:.8}</code>\n\
            <b>💵 Salvaged:</b> <code>~{:.4} SOL</code>\n\
            <b>━━━━━━━━━━━━━━━━━━━━━━</b>\n\
            <i>🕰 {}</i>",
            symbol,
            price,
            amount_sol,
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
        );

        self.send_message(&message, true).await
    }

    /// Envía un mensaje informativo de estado
    pub async fn send_status_update(&self, message: &str) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }

        let formatted = format!(
            "<b>ℹ️ SYSTEM UPDATE</b>\n\
            <b>━━━━━━━━━━━━━━━━━━━━━━</b>\n\
            {}\n\
            <b>━━━━━━━━━━━━━━━━━━━━━━</b>\n\
            <i>🕰 {}</i>",
            message,
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
        );

        self.send_message(&formatted, true).await
    }

    /// Envía una alerta de conectividad (desconexión/reconexión de feeds)
    pub async fn send_connectivity_alert(
        &self,
        feed_name: &str,
        is_connected: bool,
        details: &str,
    ) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }

        let (icon, status) = if is_connected {
            ("✅", "RECONECTADO")
        } else {
            ("⚠️", "DESCONECTADO")
        };

        let message = format!(
            "<b>{} FEED {}</b>\n\
            <b>━━━━━━━━━━━━━━━━━━━━━━</b>\n\
            <b>⬢ Feed:</b> <code>{}</code>\n\
            <b>⬡ Estado:</b> {}\n\
            {}\n\
            <b>━━━━━━━━━━━━━━━━━━━━━━</b>\n\
            <i>🕰 {}</i>",
            icon,
            status,
            feed_name,
            if is_connected { "🟢 Online" } else { "🔴 Offline" },
            details,
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
        );

        self.send_message(&message, true).await
    }

    /// Envía un alerta de error crítico
    pub async fn send_error_alert(&self, error: &str) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }

        let message = format!(
            "<b>❌ CRITICAL SYSTEM FAILURE</b>\n\
            <b>━━━━━━━━━━━━━━━━━━━━━━</b>\n\
            <code>{}</code>\n\
            <b>━━━━━━━━━━━━━━━━━━━━━━</b>\n\
            <i>🕰 {}</i>",
            error,
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
        );

        self.send_message(&message, true).await
    }

    /// Método interno para enviar mensajes
    pub async fn send_message(&self, text: &str, html: bool) -> Result<()> {
        let url = format!("https://api.telegram.org/bot{}/sendMessage", self.bot_token);

        let mut payload = json!({
            "chat_id": self.chat_id,
            "text": text,
        });

        if html {
            payload["parse_mode"] = json!("HTML");
        }

        let client = reqwest::Client::new();
        let response = client.post(&url).json(&payload).send().await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            eprintln!("⚠️  Error enviando mensaje a Telegram: {}", error_text);
            anyhow::bail!("Error de Telegram API: {}", error_text);
        }

        Ok(())
    }

    /// Verifica si el notificador está habilitado
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}
pub mod commands;
