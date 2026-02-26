use anyhow::Result;
use std::sync::Arc;
use crate::state_manager::StateManager;

/// Comando /track - Añade un token manualmente al DB para monitoreo
    pub async fn cmd_track(handler: &super::CommandHandler, command: &str, state_manager: Arc<StateManager>) -> Result<()> {
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.len() < 5 {
            handler.send_message("❌ <b>Syntax Error:</b> <code>/track &lt;MINT&gt; &lt;SYMBOL&gt; &lt;SOL&gt; &lt;SL&gt; [TP]</code>\nExample: <code>/track 3GEz... SCRAT 0.1 -50 100</code>").await?;
            return Ok(());
        }

        let mint = parts[1];

        // ✅ CRITICAL: Validar mint antes de procesar
        let valid_mint =
            match crate::validation::FinancialValidator::validate_mint(mint, "/track command") {
                Ok(m) => m,
                Err(e) => {
                    handler.send_message(&format!("❌ <b>MINT VALIDATION ERROR:</b> {}", e))
                        .await?;
                    return Ok(());
                }
            };

        let symbol = parts[2];
        let sol: f64 = parts[3].parse().unwrap_or(0.0);
        let sl: f64 = parts[4].parse().unwrap_or(-50.0);
        let tp: f64 = if parts.len() > 5 {
            parts[5].parse().unwrap_or(100.0)
        } else {
            100.0
        };

        handler.send_message(&format!("🔍 <b>Indexing Asset: {}...</b>", symbol))
            .await?;

        let scanner = crate::scanner::PriceScanner::new();
        match scanner.get_token_price(&valid_mint).await {
            Ok(price_data) => {
                let pos = crate::state_manager::PositionState {
                    id: None,
                    token_mint: valid_mint.to_string(),
                    symbol: symbol.to_string(),
                    entry_price: price_data.price_native,
                    current_price: price_data.price_native,
                    amount_sol: sol,
                    stop_loss_percent: sl,
                    trailing_enabled: true,
                    trailing_distance_percent: 25.0,
                    trailing_activation_threshold: 20.0,
                    trailing_highest_price: Some(price_data.price_native),
                    trailing_current_sl: Some(sl),
                    tp_percent: Some(tp),
                    tp_amount_percent: Some(50.0), // Default sell 50%
                    tp_triggered: false,
                    tp2_percent: Some(200.0),        // TP2 Moonbag default
                    tp2_amount_percent: Some(100.0), // Sell remaining
                    tp2_triggered: false,
                    active: true,
                    created_at: chrono::Utc::now().timestamp(),
                    updated_at: chrono::Utc::now().timestamp(),
                };

                state_manager.upsert_position(pos).await?;

                handler.send_message(&format!(
                    "<b>✅ ASSET TRACKED SUCCESSFULLY</b>\n\
                    <b>━━━━━━━━━━━━━━━━━━━━━━</b>\n\
                    <b>⬢ Symbol:</b> <code>{}</code>\n\
                    <b>⬢ Entry:</b> <code>{:.8} SOL</code>\n\
                    <b>⬢ SL / TP:</b> <code>{}% / {}%</code>\n\
                    <b>━━━━━━━━━━━━━━━━━━━━━━</b>\n\
                    <i>🔄 Note: Restart tracking active once PriceFeed refreshes.</i>",
                    symbol, price_data.price_native, sl, tp
                ))
                .await?;
            }
            Err(e) => {
                handler.send_message(&format!("❌ <b>Tracking Error:</b> {}", e))
                    .await?;
            }
        }

        Ok(())
    }

/// Comando /untrack - Elimina instantáneamente de SQLite y silencia al monitor
    pub async fn cmd_untrack(handler: &super::CommandHandler, command: &str, state_manager: Arc<StateManager>) -> Result<()> {
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.len() < 2 {
            handler.send_message("❌ <b>Syntax Error:</b> <code>/untrack &lt;MINT&gt;</code>")
                .await?;
            return Ok(());
        }

        let mint = parts[1];
        match state_manager.close_position(mint).await {
            Ok(_) => {
                handler.send_message(&format!("🔴 <b>ASSET UNTRACKED</b>\n<code>{}</code> will no longer trigger trading events.", mint)).await?;
            }
            Err(e) => {
                handler.send_message(&format!("❌ <b>DB Fault:</b> {}", e))
                    .await?;
            }
        }
        Ok(())
    }

/// Comando /update - Hot swap parameters in DB
    pub async fn cmd_update(handler: &super::CommandHandler, command: &str, state_manager: Arc<StateManager>) -> Result<()> {
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.len() < 3 {
            handler.send_message(
                "❌ <b>Syntax Error:</b> <code>/update &lt;MINT&gt; sl=-X tp=Y</code>",
            )
            .await?;
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
                    handler.send_message(&format!("❌ <b>DB Fault:</b> {}", e))
                        .await?;
                    return Ok(());
                }

                let msg = format!(
                    "⚙️ <b>HOT-SWAP COMPLETE</b> for {}\n\
                     <b>━━━━━━━━━━━━━━━━━━━━━━</b>\n\
                     ⬡ <b>SL:</b> {}\n\
                     ⬡ <b>TP:</b> {}\n\
                     <i>Execution engine updated without reboot.</i>",
                    pos.symbol,
                    if updated_sl {
                        format!("<code>{:.1}%</code> ✅", pos.stop_loss_percent)
                    } else {
                        "Unchanged".to_string()
                    },
                    if updated_tp {
                        format!("<code>{:.1}%</code> ✅", pos.tp_percent.unwrap_or(0.0))
                    } else {
                        "Unchanged".to_string()
                    }
                );
                handler.send_message(&msg).await?;
            }
            Ok(None) => {
                handler.send_message(&format!(
                    "⚠️ <b>Position Not Found:</b> <code>{}</code>",
                    mint
                ))
                .await?;
            }
            Err(e) => {
                handler.send_message(&format!("❌ <b>DB Fault:</b> {}", e))
                    .await?;
            }
        }

        Ok(())
    }

