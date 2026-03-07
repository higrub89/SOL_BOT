use anyhow::Result;
use std::sync::Arc;
use crate::wallet::WalletMonitor;
use solana_client::rpc_client::RpcClient;
use std::time::Instant;

/// Comando /ping — Diagnostic & Latency (Titanium Interface)
pub async fn cmd_ping(handler: &super::CommandHandler, wallet_monitor: Arc<WalletMonitor>) -> Result<()> {
    let uptime = handler.start_time.elapsed();
    let hours   = uptime.as_secs() / 3600;
    let minutes = (uptime.as_secs() % 3600) / 60;
    let secs    = uptime.as_secs() % 60;

    // ── RPC Latency Check ─────────────────────────────
    let (rpc_line, rpc_latency_ms) = if let Ok(api_key) = std::env::var("HELIUS_API_KEY") {
        let rpc_url = format!("https://mainnet.helius-rpc.com/?api-key={}", api_key);
        let t0 = Instant::now();
        let client = RpcClient::new(rpc_url);
        match client.get_slot() {
            Ok(slot) => {
                let ms = t0.elapsed().as_millis();
                (format!("{}ms  ·  slot #{}", ms, slot), ms)
            }
            Err(e) => (format!("FAULT  ·  {}", e), 9999),
        }
    } else {
        ("API KEY MISSING".to_string(), 9999)
    };

    // ── Latency Grade (S-CLASS through DEGRADED) ─────
    let (grade, grade_desc) = if rpc_latency_ms < 100 {
        ("S-CLASS", "Optimal")
    } else if rpc_latency_ms < 250 {
        ("A-CLASS", "High Performance")
    } else if rpc_latency_ms < 500 {
        ("B-CLASS", "Acceptable")
    } else {
        ("DEGRADED", "Network Issues")
    };

    // ── Wallet Balance ─────────────────────────────────
    let wallet_line = match wallet_monitor.get_sol_balance() {
        Ok(bal) => {
            let dot = if bal > 0.05 { "⬢" } else if bal > 0.01 { "⬥" } else { "⏷" };
            format!("{}  {:.6} SOL", dot, bal)
        }
        Err(e) => format!("⏷  VAULT ERROR  ·  {}", e),
    };

    // ── Engine State ───────────────────────────────────
    let engine_state = if super::CommandHandler::is_hibernating() {
        "SUSPENDED"
    } else {
        "ENGAGED"
    };

    let msg = format!(
"<b>THE CHASSIS</b>  <code>DIAGNOSTIC</code>
<code>──────────────────────────</code>

<b>UPTIME</b>
<code>  {h:02}h {m:02}m {s:02}s</code>

<b>NETWORK</b>
<code>  {rpc}</code>
<code>  GRADE  {grade} ({desc})</code>

<b>VAULT</b>
<code>  {wallet}</code>

<b>ENGINE</b>
<code>  {engine}</code>

<code>──────────────────────────</code>
<i>The Chassis · Institutional Execution Layer</i>",
        h = hours, m = minutes, s = secs,
        rpc    = rpc_line,
        grade  = grade,
        desc   = grade_desc,
        wallet = wallet_line,
        engine = engine_state,
    );

    handler.send_message(&msg).await?;
    Ok(())
}
