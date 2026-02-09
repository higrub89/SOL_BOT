//! # Telegram Commands Handler
//! 
//! Sistema de comandos interactivos para controlar The Chassis desde Telegram

use anyhow::Result;
use std::sync::{Arc, Mutex};
use crate::emergency::EmergencyMonitor;
use crate::wallet::WalletMonitor;
use crate::config::AppConfig;
use crate::executor_v2::{TradeExecutor, ExecutorConfig};
use solana_sdk::signature::Keypair;

pub struct CommandHandler {
    bot_token: String,
    chat_id: String,
    enabled: bool,
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
        }
    }

    /// Procesa comandos recibidos del usuario
    pub async fn process_commands(
        &self,
        emergency_monitor: Arc<Mutex<EmergencyMonitor>>,
        wallet_monitor: Arc<WalletMonitor>,
        config: Arc<AppConfig>,
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
                                Arc::clone(&config),
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
        config: Arc<AppConfig>,
    ) -> Result<()> {
        match command.trim() {
            "/start" => {
                self.send_message("🏎️ **The Chassis Bot v1.1.0**\n\n\
                    ⚡ *Comandos disponibles:*\n\n\
                    💰 `/buy <MINT> <SOL>` - Comprar token\n\
                    📊 `/status` - Estado de posiciones\n\
                    💵 `/balance` - Balance de wallet\n\
                    🎯 `/targets` - Tokens monitoreados\n\
                    ❓ `/help` - Ver ayuda completa\n\n\
                    _El bot protege tus posiciones 24/7 con Trailing Stop-Loss._").await?;
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

            "/help" => {
                self.send_message("📚 **Ayuda de The Chassis**\n\n\
                    • `/status` - Muestra precio actual, drawdown y distancia al SL de cada token\n\
                    • `/balance` - Balance de SOL en tu wallet\n\
                    • `/targets` - Lista de tokens monitoreados\n\
                    • `/buy <MINT> <SOL>` - Compra un token (ej: /buy ABC123... 0.05)\n\
                    • `/pause` - Pausa las alertas (el monitoreo continúa)\n\
                    • `/resume` - Reactiva las alertas\n\n\
                    El bot monitorea automáticamente tus tokens 24/7.").await?;
            }

            cmd if cmd.starts_with("/buy ") => {
                self.cmd_buy(cmd).await?;
            }

            _ => {
                // Comando no reconocido, ignorar silenciosamente
            }
        }

        Ok(())
    }

    /// Comando /buy - Ejecuta una compra de token
    async fn cmd_buy(&self, command: &str) -> Result<()> {
        // Parsear: /buy <MINT> <AMOUNT>
        let parts: Vec<&str> = command.split_whitespace().collect();
        
        if parts.len() < 3 {
            self.send_message("❌ **Uso:** `/buy <MINT> <SOL>`\n\nEjemplo: `/buy 7SYuU1Z6EKfp... 0.05`").await?;
            return Ok(());
        }

        let mint = parts[1];
        let amount: f64 = match parts[2].parse() {
            Ok(a) => a,
            Err(_) => {
                self.send_message("❌ Cantidad inválida. Usa un número (ej: 0.05)").await?;
                return Ok(());
            }
        };

        // Validar cantidad mínima
        if amount < 0.01 {
            self.send_message("❌ Cantidad mínima: 0.01 SOL").await?;
            return Ok(());
        }

        self.send_message(&format!("🔍 Preparando compra...\n\n💰 {:.4} SOL → {}", amount, &mint[..12])).await?;

        // Configurar executor
        let api_key = std::env::var("HELIUS_API_KEY").unwrap_or_default();
        let rpc_url = format!("https://mainnet.helius-rpc.com/?api-key={}", api_key);
        
        let config = ExecutorConfig {
            rpc_url,
            slippage_bps: 100, // 1%
            priority_fee: 50_000,
            dry_run: false,
        };

        let executor = TradeExecutor::new(config);

        // Cargar keypair
        let priv_key = match std::env::var("WALLET_PRIVATE_KEY") {
            Ok(k) => k,
            Err(_) => {
                self.send_message("❌ WALLET_PRIVATE_KEY no configurada en .env").await?;
                return Ok(());
            }
        };
        let keypair = Keypair::from_base58_string(&priv_key);

        // Ejecutar compra
        self.send_message("🚀 Ejecutando swap en Jupiter...").await?;
        
        match executor.execute_buy(mint, Some(&keypair), amount).await {
            Ok(result) => {
                let msg = format!(
                    "✅ **COMPRA EXITOSA**\n\n\
                    💰 SOL gastado: {:.4}\n\
                    💎 Tokens: {:.0}\n\
                    📊 Precio: ${:.10}\n\
                    🔗 [Ver en Solscan](https://solscan.io/tx/{})",
                    result.sol_spent,
                    result.tokens_received,
                    result.price_per_token,
                    result.signature
                );
                self.send_message(&msg).await?;
            }
            Err(e) => {
                self.send_message(&format!("❌ Error en la compra: {}", e)).await?;
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
}
