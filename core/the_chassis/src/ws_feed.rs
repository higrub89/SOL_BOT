//! # WebSocket Price Feed — Seguimiento de Precios via Solana WebSocket (GRATIS)
//!
//! Usa `accountSubscribe` nativo de Solana para recibir updates push
//! cuando las vault accounts cambian. Calcula precio desde reserves.
//!
//! ## Ventajas sobre DexScreener
//! - Push (no polling) → ~200-400ms latencia vs ~5000ms
//! - On-chain directo → No depende de terceros  
//! - GRATIS con cualquier RPC (Helius free tier incluido)
//!
//! ## Flow
//! ```text
//!   WebSocket connect → accountSubscribe(coin_vault, pc_vault)
//!     │
//!     ▼ (on account change notification)
//!   getAccountInfo(vault) → parse amount → update reserves → calculate price
//!     │
//!     ▼
//!   PriceUpdate → mpsc channel → Monitor principal
//! ```

use futures_util::{SinkExt, StreamExt};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, RwLock};

use crate::amm_math::{
    new_sol_price_tracker, parse_spl_token_account_amount, SolPriceUsd, VaultPair,
};
use crate::price_feed::{MonitoredToken, PriceCache, PriceSource, PriceUpdate};
use crate::telegram::TelegramNotifier;

/// Ejecuta el loop de WebSocket que monitorea vault accounts
/// y calcula precios on-chain en tiempo real.
pub async fn ws_price_loop(
    tx: mpsc::Sender<PriceUpdate>,
    tokens: Vec<MonitoredToken>,
    rpc_ws_url: String,
    cache: PriceCache,
) {
    let mut reconnect_delay = Duration::from_secs(2);
    let max_reconnect_delay = Duration::from_secs(60);
    let mut reconnection_count: u32 = 0;
    let staleness_timeout = Duration::from_secs(60); // WS es más lento que gRPC
    let notifier = TelegramNotifier::new();

    // ── Construir vault tracking ──
    let vault_pairs: Vec<VaultPair> = tokens
        .iter()
        .filter(|t| {
            t.coin_vault.as_ref().is_some_and(|v| v.len() >= 8)
                && t.pc_vault.as_ref().is_some_and(|v| v.len() >= 8)
        })
        .map(|t| VaultPair {
            token_mint: t.mint.clone(),
            symbol: t.symbol.clone(),
            coin_vault: t.coin_vault.clone().unwrap(),
            pc_vault: t.pc_vault.clone().unwrap(),
            base_decimals: t.token_decimals,
            quote_decimals: 9, // SOL
            last_coin_reserve: None,
            last_pc_reserve: None,
        })
        .collect();

    if vault_pairs.is_empty() {
        eprintln!("⚠️  [WebSocket] No hay vault accounts configuradas.");
        eprintln!("   💡 Usa 'cargo run --bin find_vaults -- <MINT>' para encontrarlas");
        return;
    }

    // Mapa: vault_address → token_mint
    let mut vault_to_mint: HashMap<String, String> = HashMap::new();
    // Mapa: vault_address → is_coin (true = coin_vault, false = pc_vault)
    let mut vault_is_coin: HashMap<String, bool> = HashMap::new();

    for pair in &vault_pairs {
        vault_to_mint.insert(pair.coin_vault.clone(), pair.token_mint.clone());
        vault_to_mint.insert(pair.pc_vault.clone(), pair.token_mint.clone());
        vault_is_coin.insert(pair.coin_vault.clone(), true);
        vault_is_coin.insert(pair.pc_vault.clone(), false);
    }

    let all_vault_addresses: Vec<String> = vault_to_mint.keys().cloned().collect();

    println!(
        "   🔌 WebSocket vault accounts: {}",
        all_vault_addresses.len()
    );
    for pair in &vault_pairs {
        let coin_display = if pair.coin_vault.len() >= 8 {
            &pair.coin_vault[..8]
        } else {
            &pair.coin_vault
        };
        let pc_display = if pair.pc_vault.len() >= 8 {
            &pair.pc_vault[..8]
        } else {
            &pair.pc_vault
        };
        println!(
            "      └─ {} | coin: {}... | pc: {}...",
            pair.symbol, coin_display, pc_display
        );
    }

    // Vault tracker thread-safe
    let vault_tracker: Arc<RwLock<HashMap<String, VaultPair>>> = {
        let mut map = HashMap::new();
        for pair in vault_pairs {
            map.insert(pair.token_mint.clone(), pair);
        }
        Arc::new(RwLock::new(map))
    };

    // SOL price tracker
    let sol_price: SolPriceUsd = new_sol_price_tracker();

    // Mapa para trackear subscription_id → vault_address
    let sub_to_vault: Arc<RwLock<HashMap<u64, String>>> = Arc::new(RwLock::new(HashMap::new()));

    // ── Loop de conexión con reconexión automática ──
    loop {
        println!(
            "🔌 [WebSocket] Conectando a {}...",
            &rpc_ws_url[..50.min(rpc_ws_url.len())]
        );

        match tokio_tungstenite::connect_async(&rpc_ws_url).await {
            Ok((ws_stream, _response)) => {
                println!("✅ [WebSocket] Conexión establecida");
                reconnect_delay = Duration::from_secs(2);

                // Notificar reconexión por Telegram (solo si es una reconexión, no la primera)
                if reconnection_count > 0 {
                    let _ = notifier.send_connectivity_alert(
                        "WebSocket RPC",
                        true,
                        &format!("<b>Reconexión #{}</b> exitosa.", reconnection_count),
                    ).await;
                }

                let (mut write, mut read) = ws_stream.split();

                // ── Suscribirse a cada vault account ──
                // Guardamos request_id → vault_address para mapear respuestas
                let mut request_id_to_vault: HashMap<u64, String> = HashMap::new();

                for (i, vault_addr) in all_vault_addresses.iter().enumerate() {
                    let request_id = (i + 1) as u64;

                    let subscribe_msg = json!({
                        "jsonrpc": "2.0",
                        "id": request_id,
                        "method": "accountSubscribe",
                        "params": [
                            vault_addr,
                            {
                                "encoding": "base64",
                                "commitment": "processed"  // Máxima velocidad
                            }
                        ]
                    });

                    if let Err(e) = write
                        .send(tokio_tungstenite::tungstenite::Message::Text(
                            subscribe_msg.to_string(),
                        ))
                        .await
                    {
                        eprintln!("❌ [WebSocket] Error enviando suscripción: {}", e);
                        break;
                    }

                    request_id_to_vault.insert(request_id, vault_addr.clone());
                }

                println!(
                    "📡 [WebSocket] Suscrito a {} vault accounts (commitment: processed)",
                    all_vault_addresses.len()
                );

                let mut update_count: u64 = 0;
                let start_time = Instant::now();
                let mut last_data_at = Instant::now();

                // ── Procesar mensajes entrantes ──
                loop {
                    // Watchdog: detectar stream zombie
                    let msg_result = tokio::select! {
                        msg = read.next() => msg,
                        _ = tokio::time::sleep(staleness_timeout) => {
                            let stale_secs = last_data_at.elapsed().as_secs();
                            eprintln!(
                                "🧊 [WebSocket] Stream ZOMBIE detectado — sin datos hace {}s",
                                stale_secs
                            );
                            let _ = notifier.send_connectivity_alert(
                                "WebSocket RPC",
                                false,
                                &format!(
                                    "🧊 <b>Stream zombie</b> — sin datos hace {}s\n\
                                     Updates antes del corte: {}",
                                    stale_secs, update_count
                                ),
                            ).await;
                            break;
                        }
                    };

                    let msg_result = match msg_result {
                        Some(r) => r,
                        None => break, // Stream cerrado
                    };

                    match msg_result {
                        Ok(msg) => {
                            last_data_at = Instant::now(); // Reset watchdog
                            if let tokio_tungstenite::tungstenite::Message::Text(text) = msg {
                                if let Ok(json_msg) = serde_json::from_str::<Value>(&text) {
                                    // Caso 1: Respuesta a suscripción (contiene "id" y "result")
                                    if let Some(id) = json_msg.get("id").and_then(|v| v.as_u64()) {
                                        if let Some(sub_id) =
                                            json_msg.get("result").and_then(|v| v.as_u64())
                                        {
                                            if let Some(vault_addr) = request_id_to_vault.get(&id) {
                                                let mut subs = sub_to_vault.write().await;
                                                subs.insert(sub_id, vault_addr.clone());
                                            }
                                        }
                                        continue;
                                    }

                                    // Caso 2: Notificación de cambio de cuenta
                                    if json_msg.get("method").and_then(|v| v.as_str())
                                        == Some("accountNotification")
                                    {
                                        if let Some(params) = json_msg.get("params") {
                                            let sub_id = params
                                                .get("subscription")
                                                .and_then(|v| v.as_u64())
                                                .unwrap_or(0);

                                            // Buscar qué vault corresponde a esta suscripción
                                            let vault_addr = {
                                                let subs = sub_to_vault.read().await;
                                                subs.get(&sub_id).cloned()
                                            };

                                            if let Some(vault_addr) = vault_addr {
                                                // Extraer account data
                                                if let Some(data) = extract_account_data(params) {
                                                    if let Some(amount) =
                                                        parse_spl_token_account_amount(&data)
                                                    {
                                                        // Encontrar el token al que pertenece esta vault
                                                        if let Some(token_mint) =
                                                            vault_to_mint.get(&vault_addr)
                                                        {
                                                            update_count += 1;

                                                            // Actualizar reserve y calcular precio
                                                            let price_result = {
                                                                let mut tracker =
                                                                    vault_tracker.write().await;
                                                                if let Some(pair) =
                                                                    tracker.get_mut(token_mint)
                                                                {
                                                                    pair.update_reserve(
                                                                        &vault_addr,
                                                                        amount,
                                                                    );

                                                                    if pair.is_ready() {
                                                                        pair.calculate_price_in_quote().map(|price_sol| {
                                                                            (
                                                                                pair.symbol.clone(),
                                                                                pair.token_mint.clone(),
                                                                                price_sol,
                                                                                pair.calculate_liquidity_in_quote().unwrap_or(0.0),
                                                                            )
                                                                        })
                                                                    } else {
                                                                        None
                                                                    }
                                                                } else {
                                                                    None
                                                                }
                                                            };

                                                            if let Some((
                                                                symbol,
                                                                mint,
                                                                price_sol,
                                                                liq_sol,
                                                            )) = price_result
                                                            {
                                                                // Obtener SOL price desde caché
                                                                let current_sol =
                                                                    *sol_price.read().await;
                                                                let sol_usd = if current_sol == 0.0
                                                                {
                                                                    let c = cache.read().await;
                                                                    c.values()
                                                                        .find(|p| {
                                                                            p.price_native > 0.0
                                                                                && p.price_usd > 0.0
                                                                        })
                                                                        .map(|p| {
                                                                            p.price_usd
                                                                                / p.price_native
                                                                        })
                                                                        .unwrap_or(0.0)
                                                                } else {
                                                                    current_sol
                                                                };

                                                                if sol_usd > 0.0 {
                                                                    *sol_price.write().await =
                                                                        sol_usd;
                                                                }

                                                                let price_usd = price_sol * sol_usd;
                                                                let liquidity_usd =
                                                                    liq_sol * sol_usd;

                                                                // Datos adicionales del caché
                                                                let (volume_24h, price_change_24h) = {
                                                                    let c = cache.read().await;
                                                                    c.get(&mint)
                                                                        .map(|p| {
                                                                            (
                                                                                p.volume_24h,
                                                                                p.price_change_24h,
                                                                            )
                                                                        })
                                                                        .unwrap_or((0.0, 0.0))
                                                                };

                                                                let ws_update = PriceUpdate {
                                                                    token_mint: mint.clone(),
                                                                    symbol: symbol.clone(),
                                                                    price_usd,
                                                                    price_native: price_sol,
                                                                    liquidity_usd,
                                                                    volume_24h,
                                                                    price_change_24h,
                                                                    source: PriceSource::WebSocket,
                                                                    received_at: Instant::now(),
                                                                };

                                                                // Actualizar caché
                                                                {
                                                                    let mut c = cache.write().await;
                                                                    c.insert(
                                                                        mint.clone(),
                                                                        ws_update.clone(),
                                                                    );
                                                                }

                                                                let _ = tx.try_send(ws_update);

                                                                // Log periódico
                                                                if update_count.is_multiple_of(25) {
                                                                    let elapsed =
                                                                        start_time.elapsed();
                                                                    let rate = update_count as f64
                                                                        / elapsed.as_secs_f64();
                                                                    println!(
                                                                        "⚡ [WS] #{} {} = {:.10} SOL (${:.8}) | Liq: {:.1} SOL | {:.1} upd/s",
                                                                        update_count, symbol, price_sol, price_usd, liq_sol, rate
                                                                    );
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("❌ [WebSocket] Error en stream: {}", e);
                            break;
                        }
                    }
                }

                println!(
                    "⚠️  [WebSocket] Conexión cerrada (recibidos {} updates)",
                    update_count
                );
                let _ = notifier.send_connectivity_alert(
                    "WebSocket RPC",
                    false,
                    &format!(
                        "Conexión cerrada.\n\
                         <b>Updates recibidos:</b> {}\n\
                         Reconectando con backoff...",
                        update_count
                    ),
                ).await;
                // Limpiar suscripciones
                sub_to_vault.write().await.clear();
            }
            Err(e) => {
                eprintln!("❌ [WebSocket] Error de conexión: {}", e);
                let _ = notifier.send_connectivity_alert(
                    "WebSocket RPC",
                    false,
                    &format!("Error de conexión: <code>{}</code>", e),
                ).await;
            }
        }

        // Exponential backoff
        reconnection_count += 1;
        eprintln!(
            "🔄 [WebSocket] Reconexión #{} en {:?}...",
            reconnection_count, reconnect_delay
        );
        tokio::time::sleep(reconnect_delay).await;
        reconnect_delay = (reconnect_delay * 2).min(max_reconnect_delay);
    }
}

/// Extrae los bytes del account data desde la notificación JSON
fn extract_account_data(params: &Value) -> Option<Vec<u8>> {
    let result = params.get("result")?;
    let value = result.get("value")?;
    let data_arr = value.get("data")?.as_array()?;

    // El formato es ["<base64_data>", "base64"]
    if data_arr.len() >= 2 {
        let b64_str = data_arr[0].as_str()?;
        base64::Engine::decode(&base64::engine::general_purpose::STANDARD, b64_str).ok()
    } else {
        None
    }
}
