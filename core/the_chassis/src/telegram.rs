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

impl TelegramNotifier {
    /// Crea un nuevo notificador de Telegram
    pub fn new() -> Self {
        let bot_token = std::env::var("TELEGRAM_BOT_TOKEN").unwrap_or_default();
        let chat_id = std::env::var("TELEGRAM_CHAT_ID").unwrap_or_default();
        
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
            "🚨 *ALERTA DE STOP-LOSS* 🚨\n\n\
            🪙 Token: *{}*\n\
            📉 Precio Actual: ${:.8}\n\
            📊 Precio Entrada: ${:.8}\n\
            📉 Drawdown: *{:.2}%*\n\
            🛑 Límite SL: {:.1}%\n\n\
            ⚡ *ACCIÓN REQUERIDA*\n\
            👉 [Abrir Jupiter para vender]({})\n\n\
            ⏰ {}",
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
            "✅ *VENTA AUTOMÁTICA EJECUTADA*\n\n\
            🪙 Token: *{}*\n\
            💰 Precio: ${:.8}\n\
            💵 Cantidad: ~{:.4} SOL\n\n\
            ⏰ {}",
            symbol,
            price,
            amount_sol,
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
        );
        
        self.send_message(&message, true).await
    }
    
    /// Envía un mensaje informativo de estado
    pub async fn send_status_update(
        &self,
        message: &str,
    ) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }
        
        let formatted = format!(
            "ℹ️ *Estado del Sistema*\n\n{}\n\n⏰ {}",
            message,
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
        );
        
        self.send_message(&formatted, true).await
    }
    
    /// Envía un alerta de error crítico
    pub async fn send_error_alert(
        &self,
        error: &str,
    ) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }
        
        let message = format!(
            "❌ *ERROR CRÍTICO*\n\n{}\n\n⏰ {}",
            error,
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
        );
        
        self.send_message(&message, true).await
    }
    
    /// Método interno para enviar mensajes
    async fn send_message(&self, text: &str, markdown: bool) -> Result<()> {
        let url = format!(
            "https://api.telegram.org/bot{}/sendMessage",
            self.bot_token
        );
        
        let mut payload = json!({
            "chat_id": self.chat_id,
            "text": text,
        });
        
        if markdown {
            payload["parse_mode"] = json!("Markdown");
        }
        
        let client = reqwest::Client::new();
        let response = client
            .post(&url)
            .json(&payload)
            .send()
            .await?;
        
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
