//! # Configuration Manager
//! 
//! Carga y gestiona la configuración dinámica desde targets.json.

use serde::{Deserialize, Serialize};
use std::fs;
use anyhow::{Result, Context};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AppConfig {
    pub targets: Vec<TargetConfig>,
    pub global_settings: GlobalSettings,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TargetConfig {
    pub symbol: String,
    pub mint: String,
    pub entry_price: f64,
    pub amount_sol: f64,
    pub stop_loss_percent: f64,
    pub panic_sell_price: f64,
    pub active: bool,
    
    // Pool account para Geyser streaming (opcional)
    #[serde(default)]
    pub pool_account: Option<String>,
    
    // Vault accounts para cálculo de precio on-chain (Raydium V4)
    #[serde(default)]
    pub coin_vault: Option<String>,
    #[serde(default)]
    pub pc_vault: Option<String>,
    #[serde(default = "default_token_decimals")]
    pub token_decimals: u8,
    
    // Trailing Stop-Loss (opcional)
    #[serde(default)]
    pub trailing_enabled: bool,
    #[serde(default = "default_trailing_distance")]
    pub trailing_distance_percent: f64,
    #[serde(default = "default_trailing_activation")]
    pub trailing_activation_threshold: f64,
}

fn default_trailing_distance() -> f64 { 30.0 }
fn default_trailing_activation() -> f64 { 50.0 }
fn default_token_decimals() -> u8 { 6 }

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GlobalSettings {
    pub min_sol_balance: f64,
    pub jito_tip_lamports: u64,
    pub auto_execute: bool,
    pub monitor_interval_sec: u64,
}

impl AppConfig {
    /// Carga la configuración desde targets.json
    pub fn load() -> Result<Self> {
        let content = fs::read_to_string("targets.json")
            .context("No se pudo leer targets.json")?;
        
        let config: AppConfig = serde_json::from_str(&content)
            .context("Error parseando targets.json")?;
            
        Ok(config)
    }

    /// Guarda la configuración actual a targets.json (si se actualiza en memoria)
    pub fn save(&self) -> Result<()> {
        let content = serde_json::to_string_pretty(self)?;
        fs::write("targets.json", content)?;
        Ok(())
    }
}
