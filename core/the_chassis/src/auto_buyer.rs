//! # Auto Buyer Module
//! 
//! Compra automática de tokens usando Raydium directo con fallback a Jupiter.
//! Este módulo maneja todo el flujo desde la decisión de compra hasta añadir
//! el token al monitoreo automático.

use anyhow::{Context, Result};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    pubkey::Pubkey,
    signature::{Keypair, Signer, Signature},
    transaction::VersionedTransaction,
};
use base64::{engine::general_purpose, Engine as _};
use crate::raydium::RaydiumClient;
use crate::jupiter::JupiterClient;
use crate::config::{AppConfig, TargetConfig};
use std::fs;

const SOL_MINT: &str = "So11111111111111111111111111111111111111112";

const DEFAULT_SLIPPAGE_BPS: u16 = 300; // 3% slippage por defecto

/// Resultado de una compra automática
#[derive(Debug)]
pub struct BuyResult {
    pub signature: String,
    pub token_mint: String,
    pub amount_sol: f64,
    pub tokens_received: f64,
    pub effective_price: f64,
    pub route: String, // "Raydium" o "Jupiter"
}

/// Configuración para compra automática
#[derive(Debug, Clone)]
pub struct AutoBuyConfig {
    pub token_mint: String,
    pub symbol: Option<String>,
    pub amount_sol: f64,
    pub slippage_bps: u16,
    pub add_to_monitoring: bool,
    pub stop_loss_percent: f64,
    pub trailing_enabled: bool,
}

impl Default for AutoBuyConfig {
    fn default() -> Self {
        Self {
            token_mint: String::new(),
            symbol: None,
            amount_sol: 0.025,
            slippage_bps: DEFAULT_SLIPPAGE_BPS,
            add_to_monitoring: true,
            stop_loss_percent: -60.0,
            trailing_enabled: true,
        }
    }
}

/// Auto Buyer - Ejecutor de compras automáticas
pub struct AutoBuyer {
    raydium_client: RaydiumClient,
    jupiter_client: JupiterClient,
    rpc_url: String,
}

impl AutoBuyer {
    /// Crea un nuevo AutoBuyer
    pub fn new(rpc_url: String) -> Result<Self> {
        let raydium_client = RaydiumClient::new(rpc_url.clone())?;
        let jupiter_client = JupiterClient::new();
        
        Ok(Self {
            raydium_client,
            jupiter_client,
            rpc_url,
        })
    }
    
    /// Ejecuta una compra automática
    pub async fn buy(
        &self,
        config: &AutoBuyConfig,
        keypair: &Keypair,
    ) -> Result<BuyResult> {
        println!("\n╔════════════════════════════════════════════════════════════╗");
        println!("║           🤖 AUTO-BUY - Compra Automática                ║");
        println!("╚════════════════════════════════════════════════════════════╝\n");
        
        // 1. Validaciones pre-compra
        self.validate_buy(config, keypair).await?;
        
        // 2. Obtener precio actual
        let price = self.get_current_price(&config.token_mint).await?;
        println!("💰 Precio actual: ${:.8}", price);
        
        // 3. Calcular cantidades
        let amount_in_lamports = (config.amount_sol * 1_000_000_000.0) as u64;
        let estimated_tokens = (config.amount_sol / price) * 0.97; // -3% slippage estimado
        
        println!("📊 Comprando:");
        println!("   • Inversión: {:.4} SOL", config.amount_sol);
        println!("   • Tokens estimados: {:.2}", estimated_tokens);
        println!("   • Slippage máx: {:.2}%", config.slippage_bps as f64 / 100.0);
        
        // 4. Intentar compra vía Raydium (directo)
        println!("\n🚀 Ruta 1: Intentando Raydium directo...");
        match self.buy_via_raydium(&config.token_mint, amount_in_lamports, config.slippage_bps, keypair) {
            Ok(signature) => {
                println!("✅ Compra exitosa vía Raydium!");
                println!("🔗 https://solscan.io/tx/{}", signature);
                
                let result = BuyResult {
                    signature,
                    token_mint: config.token_mint.clone(),
                    amount_sol: config.amount_sol,
                    tokens_received: estimated_tokens, // TODO: obtener cantidad real de la TX
                    effective_price: price,
                    route: "Raydium".to_string(),
                };
                
                // 5. Añadir al monitoreo si está configurado
                if config.add_to_monitoring {
                    self.add_to_monitoring(config, price)?;
                }
                
                return Ok(result);
            },
            Err(e) => {
                eprintln!("⚠️  Raydium falló: {}", e);
                println!("🔄 Ruta 2: Intentando Jupiter fallback...");
            }
        }
        
        // 5. Fallback a Jupiter
        let result = self
            .buy_via_jupiter(
                &config.token_mint,
                amount_in_lamports,
                config.slippage_bps,
                keypair,
                price,
            )
            .await?;
        
        // 6. Añadir al monitoreo
        if config.add_to_monitoring {
            self.add_to_monitoring(config, result.effective_price)?;
        }
        
        Ok(result)
    }
    
    /// Valida que se puede ejecutar la compra
    async fn validate_buy(&self, config: &AutoBuyConfig, _keypair: &Keypair) -> Result<()> {
        // Validar mint address
        if config.token_mint.is_empty() || config.token_mint.len() < 32 {
            anyhow::bail!("❌ Mint address inválido: {}", config.token_mint);
        }
        
        // Validar cantidad
        if config.amount_sol <= 0.0 {
            anyhow::bail!("❌ Cantidad debe ser mayor a 0: {}", config.amount_sol);
        }
        
        if config.amount_sol < 0.01 {
            anyhow::bail!("❌ Cantidad mínima: 0.01 SOL (recibido: {})", config.amount_sol);
        }
        
        // TODO: Verificar balance de SOL
        // Por ahora asumimos que el usuario tiene balance suficiente
        
        Ok(())
    }
    
    /// Obtiene el precio actual del token
    async fn get_current_price(&self, mint: &str) -> Result<f64> {
        // Intentar obtener precio de DexScreener
        match self.get_price_from_dexscreener(mint).await {
            Ok(price) if price > 0.0 => {
                println!("✅ Precio obtenido de DexScreener");
                return Ok(price);
            },
            _ => {}
        }
        
        // Fallback: Jupiter Price API
        match self.jupiter_client.get_price(mint).await {
            Ok(price) if price > 0.0 => {
                println!("✅ Precio obtenido de Jupiter");
                return Ok(price);
            },
            _ => {}
        }
        
        anyhow::bail!("❌ No se pudo obtener precio del token")
    }
    
    /// Obtiene precio de DexScreener
    async fn get_price_from_dexscreener(&self, mint: &str) -> Result<f64> {
        let url = format!("https://api.dexscreener.com/latest/dex/tokens/{}", mint);
        let response: serde_json::Value = reqwest::get(&url).await?.json().await?;
        
        if let Some(pairs) = response["pairs"].as_array() {
            if let Some(pair) = pairs.first() {
                if let Some(price_str) = pair["priceUsd"].as_str() {
                    if let Ok(price) = price_str.parse::<f64>() {
                        return Ok(price);
                    }
                }
            }
        }
        
        anyhow::bail!("No se encontró precio en DexScreener")
    }
    
    /// Ejecuta compra vía Raydium directo
    fn buy_via_raydium(
        &self,
        token_mint: &str,
        amount_in_lamports: u64,
        _slippage_bps: u16,
        keypair: &Keypair,
    ) -> Result<String> {
        // Calcular min_amount_out (por ahora usamos 0 para permitir cualquier cantidad)
        // TODO: Calcular basado en quote real del pool
        let min_amount_out = 0;
        
        // Ejecutar swap
        self.raydium_client.execute_swap(
            SOL_MINT,
            token_mint,
            amount_in_lamports,
            min_amount_out,
            keypair,
        )
    }
    
    /// Ejecuta compra vía Jupiter (fallback)
    async fn buy_via_jupiter(
        &self,
        token_mint: &str,
        amount_in_lamports: u64,
        slippage_bps: u16,
        keypair: &Keypair,
        reference_price_usd: f64,
    ) -> Result<BuyResult> {
        println!("🔄 Ejecutando compra vía Jupiter Aggregator...");

        const SOL_MINT: &str = "So11111111111111111111111111111111111111112";

        // 1. Inicializar RPC client
        let rpc_client = RpcClient::new_with_commitment(
            self.rpc_url.clone(),
            CommitmentConfig::confirmed(),
        );

        let user_pubkey = keypair.pubkey();

        // 2. Obtener quote de Jupiter
        println!("🔍 Consultando Jupiter para mejor ruta (fallback)...");
        let quote = self
            .jupiter_client
            .get_quote(
                SOL_MINT,
                token_mint,
                amount_in_lamports,
                slippage_bps,
            )
            .await?;

        self.jupiter_client.print_quote_summary(&quote);

        // Cantidad de tokens a recibir (en unidades de token, según Jupiter)
        let tokens_to_receive: f64 = quote
            .out_amount
            .parse::<f64>()
            .context("Error parseando out_amount de Jupiter")?;

        if tokens_to_receive <= 0.0 {
            anyhow::bail!("Jupiter quote inválido: 0 tokens a recibir");
        }

        // Cantidad de SOL gastado (lamports -> SOL)
        let amount_sol = amount_in_lamports as f64 / 1_000_000_000.0;

        // 3. Obtener transacción de swap
        println!("\n🔧 Generando transacción de swap (Jupiter)...");
        let swap_response = self
            .jupiter_client
            .get_swap_transaction(
                &quote,
                &user_pubkey.to_string(),
                true, // wrap/unwrap SOL
            )
            .await?;

        // 4. Decodificar y deserializar transacción
        println!("🔐 Decodificando transacción...");
        let tx_bytes = general_purpose::STANDARD
            .decode(&swap_response.swap_transaction)
            .context("Error decodificando transacción base64 de Jupiter")?;

        let mut transaction: VersionedTransaction = bincode::deserialize(&tx_bytes)
            .context("Error deserializando transacción de Jupiter")?;

        // 4.1 Firmar transacción con el Keypair del usuario.
        // La transacción que devuelve Jupiter suele venir con las firmas de los
        // programas, pero no con la firma del usuario. Añadimos nuestra firma
        // sobre el mensaje versionado.
        let message_bytes = transaction.message.serialize();
        let user_signature: Signature = keypair.sign_message(&message_bytes);

        // Buscar el índice de la cuenta del usuario en las account keys estáticas
        let user_pubkey: Pubkey = keypair.pubkey();
        let static_keys = transaction.message.static_account_keys();

        if let Some(index) = static_keys.iter().position(|k| *k == user_pubkey) {
            if index < transaction.signatures.len() {
                transaction.signatures[index] = user_signature;
            } else {
                transaction.signatures.resize(index + 1, Signature::default());
                transaction.signatures[index] = user_signature;
            }
        } else {
            // Fallback defensivo: si por alguna razón no encontramos la key,
            // colocamos nuestra firma en la primera posición.
            if transaction.signatures.is_empty() {
                transaction.signatures.push(user_signature);
            } else {
                transaction.signatures[0] = user_signature;
            }
        }

        // 5. Enviar transacción a la red
        println!("📡 Enviando transacción a Solana (Jupiter)...");
        let signature = rpc_client
            .send_and_confirm_transaction(&transaction)
            .context("Error enviando transacción de Jupiter")?;

        println!("✅ Compra vía Jupiter confirmada!");
        println!("🔗 https://solscan.io/tx/{}", signature);

        Ok(BuyResult {
            signature: signature.to_string(),
            token_mint: token_mint.to_string(),
            amount_sol,
            tokens_received: tokens_to_receive,
            // Usamos el precio de referencia en USD obtenido al inicio del flujo.
            effective_price: reference_price_usd,
            route: "Jupiter".to_string(),
        })
    }
    
    /// Añade el token comprado al sistema de monitoreo
    fn add_to_monitoring(&self, config: &AutoBuyConfig, entry_price: f64) -> Result<()> {
        println!("\n📊 Añadiendo al monitoreo automático...");
        
        // Leer targets.json actual
        let content = fs::read_to_string("targets.json")
            .context("No se pudo leer targets.json")?;
        
        let mut app_config: AppConfig = serde_json::from_str(&content)
            .context("Error parseando targets.json")?;
        
        // Verificar si ya existe
        if app_config.targets.iter().any(|t| t.mint == config.token_mint) {
            println!("⚠️  Token ya existe en targets.json, actualizando...");
            
            // Actualizar el target existente
            if let Some(target) = app_config.targets.iter_mut().find(|t| t.mint == config.token_mint) {
                target.entry_price = entry_price;
                target.amount_sol = config.amount_sol;
                target.active = true;
            }
        } else {
            // Crear nuevo target
            let symbol = config.symbol.clone().unwrap_or_else(|| {
                format!("TOKEN_{}", &config.token_mint[..6])
            });
            
            let new_target = TargetConfig {
                symbol,
                mint: config.token_mint.clone(),
                entry_price,
                amount_sol: config.amount_sol,
                stop_loss_percent: config.stop_loss_percent,
                panic_sell_price: entry_price * (1.0 + config.stop_loss_percent / 100.0),
                active: true,
                trailing_enabled: config.trailing_enabled,
                trailing_distance_percent: 25.0,
                trailing_activation_threshold: 100.0,
            };
            
            app_config.targets.push(new_target);
            println!("✅ Nuevo target añadido");
        }
        
        // Guardar
        let json = serde_json::to_string_pretty(&app_config)?;
        fs::write("targets.json", json)?;
        
        println!("✅ targets.json actualizado");
        println!("⚠️  Reinicia el bot para activar el monitoreo:");
        println!("   docker-compose restart");
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_auto_buy_config_default() {
        let config = AutoBuyConfig::default();
        assert_eq!(config.amount_sol, 0.025);
        assert_eq!(config.slippage_bps, 300);
        assert!(config.add_to_monitoring);
    }
}
