use anyhow::Result;
use std::sync::Arc;
use crate::state_manager::StateManager;
use crate::wallet::WalletMonitor;
use crate::config::AppConfig;

/// Comando /status - Muestra el estado de todos los tokens
    pub async fn cmd_status(handler: &super::CommandHandler, state_manager: Arc<StateManager>) -> Result<()> {
        let positions = match state_manager.get_active_positions().await {
            Ok(pos) => pos,
            Err(e) => {
                handler.send_message(&format!("❌ <b>DB Fault:</b> {}", e))
                    .await?;
                return Ok(());
            }
        };

        if positions.is_empty() {
            handler.send_message("<b>🛡️ STATUS: NO ACTIVE ALLOCATIONS</b>")
                .await?;
            return Ok(());
        }

        let mut response =
            "<b>📡 LIVE TELEMETRY</b>\n<b>━━━━━━━━━━━━━━━━━━━━━━</b>\n\n".to_string();

        for pos in positions {
            let dd = if pos.entry_price > 0.0 {
                ((pos.current_price - pos.entry_price) / pos.entry_price) * 100.0
            } else {
                0.0
            };

            let current_value = if pos.entry_price > 0.0 {
                (pos.amount_sol / pos.entry_price) * pos.current_price
            } else {
                0.0
            };

            let status_emoji = if dd > 0.0 {
                "🟢"
            } else if dd > -20.0 {
                "🟡"
            } else {
                "🔴"
            };

            let mint_display = if pos.token_mint.len() > 8 {
                &pos.token_mint[..8]
            } else {
                &pos.token_mint
            };

            response.push_str(&format!(
                "{} <b>{}</b> (<code>{}...</code>)\n\
                <b>⋄ Price:</b>   <code>{:.8} SOL</code>\n\
                <b>⋄ Entry:</b>   <code>{:.8} SOL</code>\n\
                <b>⋄ Yield:</b>   <b>{}{:.2}%</b>\n\
                <b>⋄ Value:</b>   <code>{:.6} SOL</code>\n\n",
                status_emoji,
                pos.symbol,
                mint_display,
                pos.current_price,
                pos.entry_price,
                if dd > 0.0 { "+" } else { "" },
                dd,
                current_value
            ));
        }

        response.push_str("<b>━━━━━━━━━━━━━━━━━━━━━━</b>");
        handler.send_message(&response).await?;
        Ok(())
    }

/// Comando /balance - Muestra el balance de la wallet
    pub async fn cmd_balance(handler: &super::CommandHandler, wallet_monitor: Arc<WalletMonitor>) -> Result<()> {
        match wallet_monitor.get_sol_balance() {
            Ok(balance) => {
                let message = format!(
                    "<b>🏦 VAULT STATUS</b>\n\
                    <b>━━━━━━━━━━━━━━━━━━━━━━</b>\n\n\
                    <b>⋄ Allocation:</b> <code>{:.4} SOL</code>\n\n\
                    <b>━━━━━━━━━━━━━━━━━━━━━━</b>",
                    balance
                );
                handler.send_message(&message).await?;
            }
            Err(e) => {
                handler.send_message(&format!("❌ <b>Vault Error:</b> {}", e))
                    .await?;
            }
        }
        Ok(())
    }

/// Comando /targets - Muestra la lista de tokens monitoreados
    pub async fn cmd_targets(
        handler: &super::CommandHandler,
        config: Arc<AppConfig>,
        state_manager: Arc<StateManager>,
    ) -> Result<()> {
        let mut response =
            "<b>🎯 SECURED TARGETS (DB)</b>\n<b>━━━━━━━━━━━━━━━━━━━━━━</b>\n\n".to_string();

        if let Ok(db_positions) = state_manager.get_active_positions().await {
            if db_positions.is_empty() {
                response.push_str("<i>No assets indexed.</i>\n\n");
            } else {
                for target in db_positions {
                    let status = if target.active {
                        "✅ ACTIVE"
                    } else {
                        "⏸ INACTIVE"
                    };
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
            if config.global_settings.auto_execute {
                "🔴 ARMED"
            } else {
                "🟡 DRY-RUN"
            },
            config.global_settings.monitor_interval_sec
        ));

        handler.send_message(&response).await?;
        Ok(())
    }

/// Comando /fees - Dashboard de costes de gas (Priority Fee + Jito Tip)
    pub async fn cmd_fees(handler: &super::CommandHandler, state_manager: Arc<StateManager>) -> Result<()> {
        let all_time = match state_manager.get_fee_stats(None).await {
            Ok(s) => s,
            Err(e) => {
                handler.send_message(&format!("❌ <b>DB Fault:</b> {}", e)).await?;
                return Ok(());
            }
        };

        let since_24h = chrono::Utc::now().timestamp() - 86400;
        let last_24h = state_manager.get_fee_stats(Some(since_24h)).await.unwrap_or(
            crate::state_manager::FeeStats {
                total_fee_sol: 0.0,
                total_trades: 0,
                avg_fee_sol: 0.0,
                total_pnl_gross: 0.0,
                net_pnl_sol: 0.0,
            }
        );

        let net_emoji = if all_time.net_pnl_sol > 0.0 { "🟢" } else { "🔴" };
        let net_sign = if all_time.net_pnl_sol > 0.0 { "+" } else { "" };
        let gross_sign = if all_time.total_pnl_gross > 0.0 { "+" } else { "" };

        let sep = "━━━━━━━━━━━━━━━━━━━━━━";
        let response = format!(
            "<b>⛽ FEE BURN DASHBOARD</b>
<b>{sep}</b>

            <b>◈ LAST 24H</b>
            ◉ <b>Trades:</b> <code>{t24}</code>
            ◉ <b>Fee Burn:</b> <code>-{f24:.6} SOL</code>
            ◉ <b>Avg/Trade:</b> <code>{a24:.6} SOL</code>

            <b>◈ ALL TIME</b>
            ◉ <b>Trades:</b> <code>{tall}</code>
            ◉ <b>Total Burn:</b> <code>-{fall:.6} SOL</code>
            ◉ <b>Avg/Trade:</b> <code>{aall:.6} SOL</code>

            <b>◈ P&amp;L ANALYSIS</b>
            ◉ <b>Gross PnL:</b> <code>{gsign}{gross:.4} SOL</code>
            ◉ <b>Fee Burn:</b>  <code>-{fall:.4} SOL</code>
            ◉ {nem} <b>Net PnL:</b>   <code>{nsign}{net:.4} SOL</code>

            <b>{sep}</b>
            <i>fee_sol capturado desde v2.1 en adelante.</i>",
            sep = sep,
            t24 = last_24h.total_trades, f24 = last_24h.total_fee_sol, a24 = last_24h.avg_fee_sol,
            tall = all_time.total_trades, fall = all_time.total_fee_sol, aall = all_time.avg_fee_sol,
            gsign = gross_sign, gross = all_time.total_pnl_gross,
            nem = net_emoji, nsign = net_sign, net = all_time.net_pnl_sol
        );

        handler.send_message(&response).await?;
        Ok(())
    }

/// Comando /history - Muestra historial de trades (últimos 10)
    pub async fn cmd_history(handler: &super::CommandHandler, state_manager: Arc<StateManager>) -> Result<()> {
        match state_manager.get_trade_history(10).await {
            Ok(trades) => {
                if trades.is_empty() {
                    handler.send_message("<b>📜 TRADE LEDGER</b>\n<b>━━━━━━━━━━━━━━━━━━━━━━</b>\nNo operations recorded.").await?;
                    return Ok(());
                }

                let mut response =
                    "<b>📜 RECENT EXECUTIONS</b>\n<b>━━━━━━━━━━━━━━━━━━━━━━</b>\n\n".to_string();

                for trade in trades {
                    let pnl_sol = trade.pnl_sol.unwrap_or(0.0);
                    let pnl_percent = trade.pnl_percent.unwrap_or(0.0);

                    let pnl_emoji = if pnl_sol > 0.0 { "🟢" } else { "🔴" };
                    let timestamp =
                        chrono::DateTime::<chrono::Utc>::from_timestamp(trade.timestamp, 0)
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
                handler.send_message(&response).await?;
            }
            Err(e) => {
                handler.send_message(&format!("❌ <b>DB Fault:</b> {}", e))
                    .await?;
            }
        }
        Ok(())
    }

/// Comando /stats - Muestra estadísticas completas
    pub async fn cmd_stats(handler: &super::CommandHandler, state_manager: Arc<StateManager>) -> Result<()> {
        match state_manager.get_stats().await {
            Ok(stats) => {
                let avg_pnl = if stats.total_trades > 0 {
                    stats.total_pnl_sol / stats.total_trades as f64
                } else {
                    0.0
                };

                let status_emoji = if stats.total_pnl_sol > 0.0 {
                    "🟢"
                } else if stats.total_pnl_sol == 0.0 {
                    "🟡"
                } else {
                    "🔴"
                };

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

                handler.send_message(&response).await?;
            }
            Err(e) => {
                handler.send_message(&format!("❌ <b>DB Fault:</b> {}", e))
                    .await?;
            }
        }
        Ok(())
    }

