use anyhow::Result;
use std::sync::Arc;
use crate::executor_v2::TradeExecutor;
use crate::state_manager::StateManager;



    pub async fn cmd_rbuy(
        handler: &super::CommandHandler,
        command: &str,
        executor: Arc<TradeExecutor>,
        state_manager: Arc<StateManager>,
        feed_tx: tokio::sync::mpsc::Sender<crate::price_feed::FeedCommand>,
    ) -> Result<()> {
        if super::CommandHandler::is_hibernating() {
            handler.send_message("🛑 Bot en HIBERNACIÓN. Usa `/wake` primero.")
                .await?;
            return Ok(());
        }
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.len() < 3 {
            handler.send_message(
                "❌ <b>Syntax:</b> <code>/rbuy &lt;MINT&gt; &lt;SOL&gt; [SLIPPAGE_BPS]</code>",
            )
            .await?;
        } else {
            let mint = parts[1];
            let amount: f64 = parts[2].parse().unwrap_or(0.0);
            let slippage: u16 = if parts.len() > 3 {
                parts[3].parse().unwrap_or(9999)
            } else {
                9999
            };

            // ✅ CRITICAL: Validar mint antes de ejecutar
            match crate::validation::FinancialValidator::validate_mint(mint, "/rbuy command") {
                Ok(valid_mint) => {
                    let slippage_text = if slippage >= 9000 {
                        "INFINITE".to_string()
                    } else {
                        format!("{}%", slippage as f64 / 100.0)
                    };
                    handler.send_message(&format!("<b>☢️ DEGENERATE RAYDIUM ENTRY</b>\n<b>Asset:</b> <code>{}</code>\n<b>Amount:</b> <code>{} SOL</code>\n<b>Slippage:</b> <code>{}</code>\n<i>Bypassing all guards...</i>", valid_mint, amount, slippage_text)).await?;

                    let kp_opt = crate::wallet::load_keypair_from_env("WALLET_PRIVATE_KEY").ok();
                    // Para Raydium, si el slippage es < 9000, calculamos min_out (TODO), por ahora el executor raydium usa 1
                    match executor
                        .execute_raydium_buy(&valid_mint, kp_opt.as_ref(), amount)
                        .await
                    {
                        Ok(res) => {
                            handler.send_message(&format!("<b>✅ DEGEN SUCCESS</b>\nTx: <a href='https://solscan.io/tx/{}'>VIEW</a>", res.signature)).await?;

                            // ARM MONITORING
                            let pos = crate::state_manager::PositionState {
                                id: None,
                                token_mint: valid_mint.to_string(),
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
                            if let Err(e) = state_manager.upsert_position(pos).await {
                                handler.send_message(&format!("⚠️ <b>DB Error guardando posición:</b> {}", e)).await?;
                            }

                            // REGISTRAR TRADE
                            let trade = crate::state_manager::TradeRecord {
                                id: None,
                                signature: res.signature.clone(),
                                token_mint: valid_mint.to_string(),
                                symbol: "DEGEN".to_string(),
                                trade_type: "MANUAL_BUY".to_string(),
                                amount_sol: res.sol_spent,
                                tokens_amount: res.tokens_received,
                                price: res.price_per_token,
                                pnl_sol: None,
                                pnl_percent: None,
                                route: "Telegram Direct Raydium".to_string(),
                                price_impact_pct: res.price_impact_pct,
                                fee_sol: res.fee_sol,
                                timestamp: chrono::Utc::now().timestamp(),
                            };
                            if let Err(e) = state_manager.record_trade(trade).await {
                                handler.send_message(&format!("⚠️ <b>DB Error registrando trade:</b> {}", e)).await?;
                            }

                            // 🔥 SUSCRIPCIÓN DINÁMICA
                            let _ = feed_tx
                                .send(crate::price_feed::FeedCommand::Subscribe(
                                    crate::price_feed::MonitoredToken {
                                        mint: valid_mint.to_string(),
                                        symbol: "DEGEN".to_string(),
                                        pool_account: None,
                                        coin_vault: None,
                                        pc_vault: None,
                                        token_decimals: 6,
                                    },
                                ))
                                .await;

                            handler.send_message("<b>🛡️ MONITORING ARMED</b>\nPosition saved to ledger (Dynamic subscription active).").await?;
                        }
                        Err(e) => {
                            handler.send_message(&format!("❌ <b>DEGEN RAYDIUM FAIL:</b> {}", e))
                                .await?;
                        }
                    }
                }
                Err(e) => {
                    handler.send_message(&format!("❌ <b>MINT VALIDATION ERROR:</b> {}", e))
                        .await?;
                }
            }
        }
        Ok(())
    }



    pub async fn cmd_buy(
        handler: &super::CommandHandler,
        command: &str,
        executor: Arc<TradeExecutor>,
        state_manager: Arc<StateManager>,
        feed_tx: tokio::sync::mpsc::Sender<crate::price_feed::FeedCommand>,
    ) -> Result<()> {
        if super::CommandHandler::is_hibernating() {
            handler.send_message("🛑 Bot en HIBERNACIÓN. Usa `/wake` primero.")
                .await?;
            return Ok(());
        }
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.len() < 3 {
            handler.send_message(
                "❌ <b>Syntax:</b> <code>/buy &lt;MINT&gt; &lt;SOL&gt; [SLIPPAGE_BPS]</code>",
            )
            .await?;
            return Ok(());
        }
        let mint = parts[1];
        let amount: f64 = parts[2].parse().unwrap_or(0.0);
        let slippage: u16 = if parts.len() > 3 {
            parts[3].parse().unwrap_or(100)
        } else {
            100
        };

        handler.cmd_buy_with_params(mint, amount, slippage, executor, state_manager, feed_tx)
            .await
    }

/// Comando /buy con parámetros personalizados
    pub async fn cmd_buy_with_params(
        handler: &super::CommandHandler,
        mint: &str,
        amount: f64,
        slippage_bps: u16,
        executor: Arc<TradeExecutor>,
        state_manager: Arc<StateManager>,
        feed_tx: tokio::sync::mpsc::Sender<crate::price_feed::FeedCommand>,
    ) -> Result<()> {
        // ✅ CRITICAL: Validar mint antes de ejecutar
        let valid_mint =
            match crate::validation::FinancialValidator::validate_mint(mint, "/buy command") {
                Ok(m) => m,
                Err(e) => {
                    handler.send_message(&format!("❌ <b>MINT VALIDATION ERROR:</b> {}", e))
                        .await?;
                    return Ok(());
                }
            };

        handler.send_message(&format!("<b>🛒 INITIATING BUY</b>\n<b>Asset:</b> <code>{}</code>\n<b>Amount:</b> <code>{} SOL</code>\n<b>Slippage:</b> <code>{}%</code>", valid_mint, amount, slippage_bps as f64 / 100.0)).await?;

        let kp_opt = crate::wallet::load_keypair_from_env("WALLET_PRIVATE_KEY").ok();

        // Ejecutar con parámetros custom
        match executor
            .execute_buy_with_custom_params(
                &valid_mint,
                kp_opt.as_ref(),
                amount,
                100_000,
                slippage_bps,
            )
            .await
        {
            Ok(res) => {
                // REGISTRO OBLIGATORIO: Ignoramos la hibernación para el registro si la compra fue exitosa
                if res.output_amount > 0.0 {
                    let price = amount / res.output_amount;

                    // Intentar obtener el símbolo real vía PriceScanner
                    let scanner = crate::scanner::PriceScanner::new();
                    let symbol = if let Ok(data) = scanner.get_token_price(&valid_mint).await {
                        data.symbol
                    } else {
                        "TOKEN".to_string()
                    };

                    let pos = crate::state_manager::PositionState {
                        id: None,
                        token_mint: valid_mint.to_string(),
                        symbol: symbol.clone(),
                        entry_price: price,
                        current_price: price,
                        amount_sol: amount,
                        stop_loss_percent: -20.0,
                        trailing_enabled: true,
                        trailing_distance_percent: 15.0,
                        trailing_activation_threshold: 10.0,
                        trailing_highest_price: Some(price),
                        trailing_current_sl: Some(-20.0),
                        tp_percent: Some(50.0),
                        tp_amount_percent: Some(100.0),
                        tp_triggered: false,
                        tp2_percent: None,
                        tp2_amount_percent: None,
                        tp2_triggered: false,
                        active: true,
                        created_at: chrono::Utc::now().timestamp(),
                        updated_at: chrono::Utc::now().timestamp(),
                    };

                    if let Err(e) = state_manager.upsert_position(pos).await {
                        handler.send_message(&format!(
                            "⚠️ <b>DB Error:</b> {}\nTx: {}",
                            e, res.signature
                        ))
                        .await?;
                    } else {
                        // REGISTRAR TRADE
                        let trade = crate::state_manager::TradeRecord {
                            id: None,
                            signature: res.signature.clone(),
                            token_mint: valid_mint.to_string(),
                            symbol: symbol.clone(),
                            trade_type: "MANUAL_BUY".to_string(),
                            amount_sol: amount,
                            tokens_amount: res.output_amount,
                            price,
                            pnl_sol: None,
                            pnl_percent: None,
                            route: "Telegram Base".to_string(),
                            price_impact_pct: res.price_impact_pct,
                            fee_sol: res.fee_sol,
                            timestamp: chrono::Utc::now().timestamp(),
                        };
                        if let Err(e) = state_manager.record_trade(trade).await {
                            handler.send_message(&format!("⚠️ <b>DB Error registrando trade:</b> {}\nTx: {}", e, res.signature)).await?;
                        }

                        // 🔥 SUSCRIPCIÓN DINÁMICA
                        let _ = feed_tx
                            .send(crate::price_feed::FeedCommand::Subscribe(
                                crate::price_feed::MonitoredToken {
                                    mint: valid_mint.to_string(),
                                    symbol: symbol.clone(),
                                    pool_account: None,
                                    coin_vault: None,
                                    pc_vault: None,
                                    token_decimals: 6,
                                },
                            ))
                            .await;

                        handler.send_message(&format!(
                            "<b>✅ BUY SUCCESS</b>\n\
                            <b>━━━━━━━━━━━━━━━━━━━━━━</b>\n\
                            <b>⬢ Asset:</b>   <code>{}</code>\n\
                            <b>⬢ Tokens:</b>  <code>{:.2}</code>\n\
                            <b>⬢ Entry:</b>   <code>{:.8} SOL</code>\n\
                            <b>⬢ Tx:</b> <a href='https://solscan.io/tx/{}'>VIEW</a>\n\
                            <b>━━━━━━━━━━━━━━━━━━━━━━</b>\n\
                            <i>🛡️ MONITORING ARMED (Dynamic subscription active).</i>",
                            symbol, res.output_amount, price, res.signature
                        ))
                        .await?;
                    }
                } else {
                    handler.send_message(&format!("<b>✅ BUY SUCCESS</b> (Wait for confirm)\nTx: <a href='https://solscan.io/tx/{}'>VIEW</a>", res.signature)).await?;
                }
            }
            Err(e) => {
                handler.send_message(&format!("❌ <b>BUY FAILURE:</b> {}", e))
                    .await?;
            }
        }
        Ok(())
    }

