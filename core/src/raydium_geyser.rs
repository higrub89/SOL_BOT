//! # Raydium Geyser Subsystem
//!
//! Conexión directa O(1) vía WebSocket al programa AMM V4 de Raydium.
//! Este módulo recibe el firehose de eventos y decodifica el estado en memoria
//! sin realizar heap allocations continuos (bypass del overhead).

use crate::raydium_hft::{RaydiumAmmV4State, RAYDIUM_V4_PROGRAM_ID};
use crate::price_feed::PriceCache;
use anyhow::{Context, Result};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::time::Instant;
use tokio_tungstenite::{connect_async, tungstenite::Message};

#[derive(Debug, Deserialize, Serialize)]
struct ProgramNotification {
    params: Option<ProgramParams>,
}

#[derive(Debug, Deserialize, Serialize)]
struct ProgramParams {
    result: Option<ProgramResult>,
}

#[derive(Debug, Deserialize, Serialize)]
struct ProgramResult {
    value: Option<ProgramValue>,
}

#[derive(Debug, Deserialize, Serialize)]
struct ProgramValue {
    pubkey: String,
    account: Option<AccountInfo>,
}

#[derive(Debug, Deserialize, Serialize)]
struct AccountInfo {
    data: Vec<String>,
}

/// Escucha el firehose del AMM V4
pub struct RaydiumGeyser {
    rpc_url: String,
    cache: Option<PriceCache>,
}

impl RaydiumGeyser {
    pub fn new(rpc_url: String, cache: Option<PriceCache>) -> Self {
        Self { rpc_url, cache }
    }

    pub async fn listen(&self) -> Result<()> {
        println!("🔌 Conectando al Firehose de Raydium V4...");

        let (ws_stream, _) = connect_async(&self.rpc_url)
            .await
            .context("Error conectando a WebSocket Raydium Geyser")?;

        let (mut write, mut read) = ws_stream.split();

        // Suscripción al programa Raydium V4
        let subscribe_msg = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "programSubscribe",
            "params": [
                RAYDIUM_V4_PROGRAM_ID,
                {
                    "encoding": "base64",
                    "commitment": "processed"
                }
            ]
        });

        write.send(Message::Text(subscribe_msg.to_string())).await?;

        println!("📡 Raydium Firehose Activo. Filtro: Cero-Alloc Mapped States");

        // Pre-allocate buffer continuo para la decodificación en caliente.
        // Evita re-alocar memoria cada vez que llega un bloque.
        // RaydiumAmmV4State ocupa exactamente 752 bytes, 1024 es margen suficiente.
        let mut decode_buffer = vec![0u8; 1024];
        let mut update_count: u64 = 0;

        while let Some(msg) = read.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    let start = Instant::now();
                    
                    if let Ok(notification) = serde_json::from_str::<ProgramNotification>(&text) {
                        if let Some(params) = notification.params {
                            if let Some(result) = params.result {
                                if let Some(value) = result.value {
                                    if let Some(account) = value.account {
                                        if !account.data.is_empty() {
                                            let b64_str = &account.data[0];
                                            
                                            // Zero-alloc base64 decode en nuestra arena reservada
                                            if let Ok(decoded_len) = BASE64.decode_slice(b64_str.as_bytes(), &mut decode_buffer) {
                                                // HFT Memory map instantáneo (O(1))
                                                if let Ok(state) = RaydiumAmmV4State::parse(&decode_buffer[..decoded_len]) {
                                                    let elapsed = start.elapsed();
                                                    
                                                    // TODO: Para calcular precio real, necesitamos leer los vault accounts
                                                    // con getAccountInfo(coin_vault_pubkey) y getAccountInfo(pc_vault_pubkey).
                                                    // El state account NO contiene los balances, solo los pubkeys de los vaults.
                                                    // Por ahora solo logueamos métricas de PnL disponibles en el state.
                                                    
                                                    // HFT Bridge: Cache update deshabilitado hasta implementar vault lookup
                                                    // Los balances reales están en las cuentas vault, no en el state.
                                                    if self.cache.is_some() {
                                                        let _coin_vault = state.coin_vault_pubkey();
                                                        let _pc_vault = state.pc_vault_pubkey();
                                                        // TODO: Implementar getMultipleAccounts para leer vaults en paralelo
                                                    }

                                                    if update_count % 100 == 0 {
                                                        println!(
                                                            "⚡ [Pool {}] Parse O(1) en {:?}. Base PnL: {}, Quote PnL: {}",
                                                            &value.pubkey[..8],
                                                            elapsed,
                                                            state.base_need_take_pnl(),
                                                            state.quote_need_take_pnl()
                                                        );
                                                    }
                                                    update_count += 1;
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                Ok(Message::Ping(_)) => {
                    let _ = write.send(Message::Pong(vec![])).await;
                }
                Ok(Message::Close(_)) => {
                    println!("🔴 Servidor de Geyser cerró la conexión");
                    break;
                }
                _ => {}
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_geyser_notification_parsing() {
        let sample_json = r#"{
            "jsonrpc": "2.0",
            "method": "programNotification",
            "params": {
                "result": {
                    "context": { "slot": 5208469 },
                    "value": {
                        "pubkey": "HkH...",
                        "account": {
                            "executable": false,
                            "lamports": 12345,
                            "owner": "675kPX9MHTjS2tw1y8qyxokq1tKho2FpT1GEnbUX245R",
                            "rentEpoch": 0,
                            "data": [
                                "base64_string_here",
                                "base64"
                            ]
                        }
                    }
                },
                "subscription": 24040
            }
        }"#;

        let parsed: Result<ProgramNotification, _> = serde_json::from_str(sample_json);
        assert!(parsed.is_ok());
        let notif = parsed.unwrap();
        assert_eq!(
            notif.params.unwrap().result.unwrap().value.unwrap().account.unwrap().data[0],
            "base64_string_here"
        );
    }
}
