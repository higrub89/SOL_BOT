use anyhow::Result;
use std::sync::Arc;
use crate::state_manager::StateManager;
use crate::wallet::WalletMonitor;
use crate::config::AppConfig;

// ─────────────────────────────────────────────────────────────────────────────
// /status — Live Telemetry por posición
// ─────────────────────────────────────────────────────────────────────────────
pub async fn cmd_status(
    handler: &super::CommandHandler,
    state_manager: Arc<StateManager>,
    price_cache: crate::price_feed::PriceCache,
) -> Result<()> {
    let positions = match state_manager.get_active_positions().await {
        Ok(p) => p,
        Err(e) => {
            handler.send_message(&format!("<b>SYSTEM FAULT</b>\n<code>{}</code>", e)).await?;
            return Ok(());
        }
    };

    if positions.is_empty() {
        handler.send_message(
            "<b>THE CHASSIS</b>  <code>LIVE TELEMETRY</code>\n\
            <code>━━━━━━━━━━━━━━━━━━━━━━━━━━</code>\n\n\
            <code>  NO ACTIVE ALLOCATIONS</code>\n\n\
            <code>━━━━━━━━━━━━━━━━━━━━━━━━━━</code>"
        ).await?;
        return Ok(());
    }

    let mut msg = "<b>THE CHASSIS</b>  <code>LIVE TELEMETRY</code>\n\
        <code>━━━━━━━━━━━━━━━━━━━━━━━━━━</code>\n\n".to_string();

    for mut pos in positions {
        {
            let cache = price_cache.read().await;
            if let Some(pd) = cache.get(&pos.token_mint) {
                if pd.price_native > 0.0 { pos.current_price = pd.price_native; }
            }
        }

        let dd = if pos.entry_price > 0.0 {
            ((pos.current_price - pos.entry_price) / pos.entry_price) * 100.0
        } else { 0.0 };

        let current_value = if pos.entry_price > 0.0 {
            (pos.amount_sol / pos.entry_price) * pos.current_price
        } else { 0.0 };

        let pnl_sol = current_value - pos.amount_sol;

        let signal = pnl_indicator(dd);
        let sign   = if dd >= 0.0 { "+" } else { "" };
        let psign  = if pnl_sol >= 0.0 { "+" } else { "" };

        msg.push_str(&format!(
"<b>{sym}</b>  <code>{signal}</code>
<code>  ENTRY   {entry:.9} SOL</code>
<code>  PRICE   {price:.9} SOL</code>
<code>  YIELD   {sign}{dd:.2}%   ({psign}{pnl:.4} SOL)</code>
<code>  SL      {sl:.0}%         TP  {tp:.0}%</code>

",
            sym   = pos.symbol,
            signal = signal,
            entry = pos.entry_price,
            price = pos.current_price,
            sign  = sign,
            dd    = dd,
            psign = psign,
            pnl   = pnl_sol,
            sl    = pos.stop_loss_percent,
            tp    = pos.tp_percent.unwrap_or(100.0),
        ));

        if msg.len() > 3200 {
            handler.send_message(&msg).await?;
            msg = "<b>THE CHASSIS</b>  <code>TELEMETRY · CONT.</code>\n\
                <code>━━━━━━━━━━━━━━━━━━━━━━━━━━</code>\n\n".to_string();
        }
    }

    msg.push_str("<code>━━━━━━━━━━━━━━━━━━━━━━━━━━</code>");
    handler.send_message(&msg).await?;
    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
// /balance — Vault Status
// ─────────────────────────────────────────────────────────────────────────────
pub async fn cmd_balance(
    handler: &super::CommandHandler,
    wallet_monitor: Arc<WalletMonitor>,
) -> Result<()> {
    match wallet_monitor.get_sol_balance() {
        Ok(balance) => {
            let tier = if balance > 1.0 {
                "TIER I   · OPERATIONAL"
            } else if balance > 0.1 {
                "TIER II  · REDUCED CAPACITY"
            } else if balance > 0.01 {
                "TIER III · CRITICAL RESERVE"
            } else {
                "TIER IV  · HIBERNATION ADVISED"
            };

            let bar = sol_balance_bar(balance);

            let msg = format!(
"<b>THE CHASSIS</b>  <code>VAULT STATUS</code>
<code>━━━━━━━━━━━━━━━━━━━━━━━━━━</code>

<code>  {balance:.6} SOL</code>
<code>  {bar}</code>
<code>  {tier}</code>

<code>━━━━━━━━━━━━━━━━━━━━━━━━━━</code>",
                balance = balance,
                bar     = bar,
                tier    = tier,
            );
            handler.send_message(&msg).await?;
        }
        Err(e) => {
            handler.send_message(&format!(
                "<b>VAULT FAULT</b>\n<code>{}</code>", e
            )).await?;
        }
    }
    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
// /targets — Configuración de targets activos
// ─────────────────────────────────────────────────────────────────────────────
pub async fn cmd_targets(
    handler: &super::CommandHandler,
    config: Arc<AppConfig>,
    state_manager: Arc<StateManager>,
) -> Result<()> {
    let mut msg = "<b>THE CHASSIS</b>  <code>TARGET REGISTRY</code>\n\
        <code>━━━━━━━━━━━━━━━━━━━━━━━━━━</code>\n\n".to_string();

    if let Ok(db_positions) = state_manager.get_active_positions().await {
        if db_positions.is_empty() {
            msg.push_str("<code>  NO INDEXED ASSETS</code>\n\n");
        } else {
            for (i, t) in db_positions.iter().enumerate() {
                let state = if t.active { "LIVE" } else { "PAUSED" };
                let tp_pct = t.tp_percent.unwrap_or(100.0);

                msg.push_str(&format!(
"<code>  [{i:02}] {sym:<12} {state}</code>
<code>       MINT  {mint}...</code>
<code>       SL    {sl:.0}%     TP   {tp:.0}%</code>
<code>       SIZE  {size:.4} SOL</code>

",
                    i    = i + 1,
                    sym  = t.symbol,
                    state = state,
                    mint = &t.token_mint[..8],
                    sl   = t.stop_loss_percent,
                    tp   = tp_pct,
                    size = t.amount_sol,
                ));

                if msg.len() > 3200 {
                    handler.send_message(&msg).await?;
                    msg = "<b>TARGET REGISTRY · CONT.</b>\n\
                        <code>━━━━━━━━━━━━━━━━━━━━━━━━━━</code>\n\n".to_string();
                }
            }
        }
    }

    let exec_mode = if config.global_settings.auto_execute {
        "ARMED   · LIVE EXECUTION"
    } else {
        "DRY-RUN · SIMULATION ONLY"
    };

    msg.push_str(&format!(
"<code>━━━━━━━━━━━━━━━━━━━━━━━━━━</code>
<code>  ENGINE  {mode}</code>
<code>  TICK    {tick}s interval</code>",
        mode = exec_mode,
        tick = config.global_settings.monitor_interval_sec,
    ));

    handler.send_message(&msg).await?;
    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
// /fees — Fee Burn Analysis
// ─────────────────────────────────────────────────────────────────────────────
pub async fn cmd_fees(
    handler: &super::CommandHandler,
    state_manager: Arc<StateManager>,
) -> Result<()> {
    let all_time = match state_manager.get_fee_stats(None).await {
        Ok(s) => s,
        Err(e) => {
            handler.send_message(&format!("<b>DB FAULT</b>\n<code>{}</code>", e)).await?;
            return Ok(());
        }
    };

    let since_24h = chrono::Utc::now().timestamp() - 86400;
    let last_24h = state_manager.get_fee_stats(Some(since_24h)).await
        .unwrap_or(crate::state_manager::FeeStats {
            total_fee_sol: 0.0, total_trades: 0,
            avg_fee_sol: 0.0, total_pnl_gross: 0.0, net_pnl_sol: 0.0,
        });

    let net_sign   = if all_time.net_pnl_sol  >= 0.0 { "+" } else { "" };
    let gross_sign = if all_time.total_pnl_gross >= 0.0 { "+" } else { "" };
    let net_ind    = if all_time.net_pnl_sol  >= 0.0 { "◆" } else { "▽" };

    let msg = format!(
"<b>THE CHASSIS</b>  <code>FEE ANALYTICS</code>
<code>━━━━━━━━━━━━━━━━━━━━━━━━━━</code>

<b>LAST 24H</b>
<code>  TRADES    {t24}</code>
<code>  FEE BURN  -{f24:.6} SOL</code>
<code>  AVG/TRADE  {a24:.6} SOL</code>

<b>ALL TIME</b>
<code>  TRADES    {tall}</code>
<code>  FEE BURN  -{fall:.6} SOL</code>
<code>  AVG/TRADE  {aall:.6} SOL</code>

<b>P&amp;L BREAKDOWN</b>
<code>  GROSS PNL  {gs}{gross:.6} SOL</code>
<code>  FEE DRAG  -{fall_r:.6} SOL</code>
<code>  ───────────────────────</code>
<code>  {ni} NET PNL   {ns}{net:.6} SOL</code>

<code>━━━━━━━━━━━━━━━━━━━━━━━━━━</code>
<i>Fee capture active since v2.1+</i>",
        t24   = last_24h.total_trades,
        f24   = last_24h.total_fee_sol,
        a24   = last_24h.avg_fee_sol,
        tall  = all_time.total_trades,
        fall  = all_time.total_fee_sol,
        aall  = all_time.avg_fee_sol,
        gs    = gross_sign,
        gross = all_time.total_pnl_gross,
        fall_r = all_time.total_fee_sol,
        ni    = net_ind,
        ns    = net_sign,
        net   = all_time.net_pnl_sol,
    );

    handler.send_message(&msg).await?;
    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
// /history — Últimas 10 ejecuciones
// ─────────────────────────────────────────────────────────────────────────────
pub async fn cmd_history(
    handler: &super::CommandHandler,
    state_manager: Arc<StateManager>,
) -> Result<()> {
    let trades = match state_manager.get_trade_history(10).await {
        Ok(t) => t,
        Err(e) => {
            handler.send_message(&format!("<b>DB FAULT</b>\n<code>{}</code>", e)).await?;
            return Ok(());
        }
    };

    if trades.is_empty() {
        handler.send_message(
            "<b>THE CHASSIS</b>  <code>EXECUTION LOG</code>\n\
            <code>━━━━━━━━━━━━━━━━━━━━━━━━━━</code>\n\n\
            <code>  NO OPERATIONS RECORDED</code>\n\n\
            <code>━━━━━━━━━━━━━━━━━━━━━━━━━━</code>"
        ).await?;
        return Ok(());
    }

    let mut msg = "<b>THE CHASSIS</b>  <code>EXECUTION LOG</code>\n\
        <code>━━━━━━━━━━━━━━━━━━━━━━━━━━</code>\n\n".to_string();

    for trade in trades {
        let pnl_sol = trade.pnl_sol.unwrap_or(0.0);
        let pnl_pct = trade.pnl_percent.unwrap_or(0.0);
        let sign    = if pnl_sol >= 0.0 { "+" } else { "" };
        let ind     = if pnl_sol >= 0.0 { "◆" } else { "▽" };

        let ts = chrono::DateTime::<chrono::Utc>::from_timestamp(trade.timestamp, 0)
            .map(|dt| dt.format("%m/%d %H:%M UTC").to_string())
            .unwrap_or_else(|| "—".to_string());

        msg.push_str(&format!(
"<code>  {ind} {sym:<10}  {ts}</code>
<code>    {ttype:<16}  {sign}{pnl:.4} SOL  ({pct:+.1}%)</code>

",
            ind   = ind,
            sym   = trade.symbol,
            ts    = ts,
            ttype = trade.trade_type,
            sign  = sign,
            pnl   = pnl_sol,
            pct   = pnl_pct,
        ));
    }

    msg.push_str("<code>━━━━━━━━━━━━━━━━━━━━━━━━━━</code>");
    handler.send_message(&msg).await?;
    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
// /stats — Performance Analytics
// ─────────────────────────────────────────────────────────────────────────────
pub async fn cmd_stats(
    handler: &super::CommandHandler,
    state_manager: Arc<StateManager>,
) -> Result<()> {
    match state_manager.get_stats().await {
        Ok(stats) => {
            let avg = if stats.total_trades > 0 {
                stats.total_pnl_sol / stats.total_trades as f64
            } else { 0.0 };

            let net_sign = if stats.total_pnl_sol >= 0.0 { "+" } else { "" };
            let avg_sign = if avg >= 0.0 { "+" } else { "" };
            let ind      = if stats.total_pnl_sol >= 0.0 { "◆" } else { "▽" };

            let msg = format!(
"<b>THE CHASSIS</b>  <code>PERFORMANCE</code>
<code>━━━━━━━━━━━━━━━━━━━━━━━━━━</code>

<code>  {ind} NET PNL       {ns}{pnl:.6} SOL</code>
<code>  ─────────────────────────</code>
<code>    TOTAL SCALPS   {trades}</code>
<code>    OPEN POSITIONS {active}</code>
<code>    AVG / POSITION {avgs}{avg:.6} SOL</code>

<code>━━━━━━━━━━━━━━━━━━━━━━━━━━</code>",
                ind    = ind,
                ns     = net_sign,
                pnl    = stats.total_pnl_sol,
                trades = stats.total_trades,
                active = stats.active_positions,
                avgs   = avg_sign,
                avg    = avg,
            );
            handler.send_message(&msg).await?;
        }
        Err(e) => {
            handler.send_message(&format!("<b>DB FAULT</b>\n<code>{}</code>", e)).await?;
        }
    }
    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
// Utilidades de diseño
// ─────────────────────────────────────────────────────────────────────────────

/// Indicador de rendimiento: usa solo caracteres monoespaciados limpios
fn pnl_indicator(dd: f64) -> &'static str {
    if dd > 50.0      { "◆◆◆◆◆" }
    else if dd > 20.0 { "◆◆◆◆◇" }
    else if dd > 5.0  { "◆◆◆◇◇" }
    else if dd > 0.0  { "◆◆◇◇◇" }
    else if dd > -10.0{ "◈◇◇◇◇" }
    else if dd > -25.0{ "▽▽◇◇◇" }
    else              { "▽▽▽▽▽" }
}

/// Barra de balance de SOL (10 segmentos)
fn sol_balance_bar(sol: f64) -> String {
    let max = 2.0_f64;
    let ratio = (sol / max).min(1.0);
    let filled = (ratio * 20.0).round() as usize;
    let empty  = 20 - filled;
    format!("[{}{}]", "█".repeat(filled), "░".repeat(empty))
}
