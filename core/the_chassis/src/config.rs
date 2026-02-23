//! # Configuration Manager
//! 
//! Carga y gestiona la configuración dinámica desde settings.json.

use serde::{Deserialize, Serialize};
use std::fs;
use anyhow::{Result, Context};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AppConfig {
    pub global_settings: GlobalSettings,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GlobalSettings {
    pub min_sol_balance: f64,
    pub jito_tip_lamports: u64,
    pub auto_execute: bool,
    pub monitor_interval_sec: u64,
}

impl AppConfig {
    /// Carga la configuración desde settings.json
    pub fn load() -> Result<Self> {
        let content = fs::read_to_string("settings.json")
            .context("No se pudo leer settings.json")?;
        
        let config: AppConfig = serde_json::from_str(&content)
            .context("Error parseando settings.json")?;
            
        Ok(config)
    }

    /// Guarda la configuración actual a settings.json (si se actualiza en memoria)
    pub fn save(&self) -> Result<()> {
        let content = serde_json::to_string_pretty(self)?;
        fs::write("settings.json", content)?;
        Ok(())
    }
}
