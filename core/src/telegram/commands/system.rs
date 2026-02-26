use anyhow::Result;
use std::sync::Arc;
use crate::wallet::WalletMonitor;
use solana_client::rpc_client::RpcClient;
use std::time::Instant;

/// Comando /ping - Health Check institucional
    pub async fn cmd_ping(handler: &super::CommandHandler, wallet_monitor: Arc<WalletMonitor>) -> Result<()> {
        let uptime = handler.start_time.elapsed();
        let hours = uptime.as_secs() / 3600;
        let minutes = (uptime.as_secs() % 3600) / 60;
        let secs = uptime.as_secs() % 60;

        // Check RPC
        let rpc_status = if let Ok(api_key) = std::env::var("HELIUS_API_KEY") {
            let rpc_url = format!("https://mainnet.helius-rpc.com/?api-key={}", api_key);
            let start = Instant::now();
            let client = RpcClient::new(rpc_url);
            match client.get_slot() {
                Ok(slot) => {
                    let latency = start.elapsed().as_millis();
                    let quality = if latency < 200 {
                        "🟢"
                    } else if latency < 500 {
                        "🟡"
                    } else {
                        "🔴"
                    };
                    format!("{} Helius RPC: {}ms (Slot: {})", quality, latency, slot)
                }
                Err(e) => format!("🔴 Helius RPC: ERROR ({})", e),
            }
        } else {
            "🔴 Helius RPC: API KEY no configurada".to_string()
        };

        // Check Wallet
        let wallet_status = match wallet_monitor.get_sol_balance() {
            Ok(balance) => {
                let emoji = if balance > 0.05 {
                    "🟢"
                } else if balance > 0.02 {
                    "🟡"
                } else {
                    "🔴"
                };
                format!("{} Wallet: {:.4} SOL", emoji, balance)
            }
            Err(e) => format!("🔴 Wallet: ERROR ({})", e),
        };

        // Hibernation status
        let hibernate_status = if super::CommandHandler::is_hibernating() {
            "🛑 <b>SUSPENDED</b>"
        } else {
            "🟢 <b>ENGAGED</b>"
        };

        let response = format!(
            "<b>🏓 SYSTEM DIAGNOSTICS</b>\n\
            <b>━━━━━━━━━━━━━━━━━━━━━━</b>\n\n\
            <b>⋄ Uptime:</b> <code>{}h {}m {}s</code>\n\
            <b>⋄ {}</b>\n\
            <b>⋄ {}</b>\n\
            <b>⋄ Health:</b> {}\n\
            <b>⋄ Engine:</b> <code>v2.0.0-institutional</code>\n\
            <b>⋄ Marker:</b> <code>DIAG_CODE: b08ad</code>\n\n\
            <b>━━━━━━━━━━━━━━━━━━━━━━</b>",
            hours, minutes, secs, rpc_status, wallet_status, hibernate_status
        );

        handler.send_message(&response).await?;
        Ok(())
    }

