use anyhow::Result;
use std::sync::Arc;
use crate::executor_v2::TradeExecutor;
use crate::state_manager::StateManager;
use crate::wallet::load_keypair_secure;

/// Comando /panic - Vende TODO inmediatamente
    pub async fn cmd_panic(
        handler: &super::CommandHandler,
        command: &str,
        executor: Arc<TradeExecutor>,
        state_manager: Arc<StateManager>,
    ) -> Result<()> {
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.len() < 2 {
            handler.send_message("❌ <b>Syntax Error:</b> <code>/panic &lt;MINT&gt;</code>")
                .await?;
            return Ok(());
        }

        let mint = parts[1];
        handler.send_message(&format!(
            "<b>🚨 EMERGENCY LIQUIDATION</b>\nLiquidating 100% of <code>{}</code> via Jito...",
            mint
        ))
        .await?;

        let kp_opt = match load_keypair_secure("WALLET_PRIVATE_KEY") {
            Ok(kp) => Some(kp),
            Err(e) => {
                handler.send_message(&format!("⚠️ <b>Key Vault Error:</b> {}", e))
                    .await?;
                None
            }
        };

        match executor
            .execute_emergency_sell(mint, kp_opt.as_ref(), 100)
            .await
        {
            Ok(res) => {
                if let Err(e) = state_manager.close_position(mint).await {
                    handler.send_message(&format!("⚠️ <b>DB ERROR cerrando posición:</b> {}\nLa posición puede seguir activa en DB.", e)).await?;
                }

                let trade = crate::state_manager::TradeRecord {
                    id: None,
                    signature: res.signature.clone(),
                    token_mint: mint.to_string(),
                    symbol: "MANUAL_SELL".to_string(),
                    trade_type: "MANUAL_SELL".to_string(),
                    amount_sol: res.output_amount,
                    tokens_amount: 0.0,
                    price: 0.0,
                    pnl_sol: None,
                    pnl_percent: None,
                    route: "Telegram Override".to_string(),
                    price_impact_pct: res.price_impact_pct,
                    fee_sol: res.fee_sol,
                    timestamp: chrono::Utc::now().timestamp(),
                };
                if let Err(e) = state_manager.record_trade(trade).await {
                    handler.send_message(&format!("⚠️ <b>DB ERROR registrando trade:</b> {}\nTx ejecutada pero NO registrada en historial.", e)).await?;
                }

                handler.send_message(&format!("<b>✅ LIQUIDATION COMPLETE</b>\n<b>⬢ Tx:</b> <code>{}</code>\n🛑 Posición cerrada en base de datos. Monitoreo apagado.", res.signature)).await?
            }
            Err(e) => {
                handler.send_message(&format!("❌ <b>CRITICAL FAILURE:</b> {}", e))
                    .await?
            }
        }

        Ok(())
    }

/// Comando /panic_all - Liquida TODAS las posiciones activas en un bundle
    pub async fn cmd_panic_all(
        handler: &super::CommandHandler,
        executor: Arc<TradeExecutor>,
        state_manager: Arc<StateManager>,
    ) -> Result<()> {
        handler.send_message("<b>🚨 GLOBAL LIQUIDATION INITIATED</b>\nGathering all active positions for Jito Bundling...").await?;

        let active_positions = state_manager.get_active_positions().await?;
        if active_positions.is_empty() {
            handler.send_message("<b>⚠️ Aborting:</b> No active positions found to liquidate.")
                .await?;
            return Ok(());
        }

        let mints: Vec<String> = active_positions
            .iter()
            .map(|p| p.token_mint.clone())
            .collect();
        let symbols: Vec<String> = active_positions.iter().map(|p| p.symbol.clone()).collect();

        handler.send_message(&format!(
            "<b>📦 Bundling Targets:</b> <code>{}</code>\n<i>Optimizing routes...</i>",
            symbols.join(", ")
        ))
        .await?;

        let kp_opt = match load_keypair_secure("WALLET_PRIVATE_KEY") {
            Ok(kp) => Some(kp),
            Err(e) => {
                handler.send_message(&format!("⚠️ <b>Key Vault Error:</b> {}", e))
                    .await?;
                None
            }
        };

        if let Some(kp) = kp_opt {
            match executor.execute_multi_sell(mints.clone(), &kp, 100).await {
                Ok(results) => {
                    let mut total_sol = 0.0;
                    for res in &results {
                        total_sol += res.output_amount;
                    }

                    handler.send_message(&format!(
                        "<b>✅ GLOBAL LIQUIDATION COMPLETE</b>\n\
                        <b>━━━━━━━━━━━━━━━━━━━━━━</b>\n\
                        <b>⬢ Items:</b> <code>{}</code>\n\
                        <b>⬢ Total Yield:</b> <code>{:.4} SOL</code>\n\
                        <b>━━━━━━━━━━━━━━━━━━━━━━</b>\n\
                        <i>All tracked positions have been closed.</i>",
                        results.len(),
                        total_sol
                    ))
                    .await?;

                    // Marcar como inactivas en DB
                    for (mint, res) in mints.into_iter().zip(results) {
                        if let Err(e) = state_manager.close_position(&mint).await {
                            eprintln!("❌ DB ERROR cerrando posición {} en panic_all: {}", mint, e);
                        }

                        let trade = crate::state_manager::TradeRecord {
                            id: None,
                            signature: res.signature.clone(),
                            token_mint: mint.clone(),
                            symbol: "MANUAL_SELL_ALL".to_string(),
                            trade_type: "MANUAL_SELL".to_string(),
                            amount_sol: res.output_amount,
                            tokens_amount: 0.0,
                            price: 0.0,
                            pnl_sol: None,
                            pnl_percent: None,
                            route: "Telegram Override Bundle".to_string(),
                            price_impact_pct: res.price_impact_pct,
                            fee_sol: res.fee_sol,
                            timestamp: chrono::Utc::now().timestamp(),
                        };
                        if let Err(e) = state_manager.record_trade(trade).await {
                            eprintln!("❌ DB ERROR registrando trade {} en panic_all: {}", mint, e);
                        }
                    }
                }
                Err(e) => {
                    handler.send_message(&format!("❌ <b>CRITICAL BUNDLE FAILURE:</b> {}", e))
                        .await?;
                }
            }
        }

        Ok(())
    }

