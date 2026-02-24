//! # Emergency Exit System
//! 
//! Sistema de stop-loss y panic sell para protección de capital.
//! Usa Jito Bundles para garantizar ejecución ultra-rápida.


use std::time::{SystemTime, UNIX_EPOCH};

/// Configuración de las condiciones de emergencia
#[derive(Debug, Clone)]
pub struct EmergencyConfig {
    /// % de pérdida máxima antes de activar stop-loss (-30 = -30%)
    pub max_loss_percent: f64,
    
    /// Balance mínimo de SOL en wallet antes de activar alerta
    pub min_sol_balance: f64,
    
    /// Precio mínimo del activo (en USD) antes de panic sell
    pub min_asset_price: f64,
    
    /// Activar modo emergencia
    pub enabled: bool,
}

impl Default for EmergencyConfig {
    fn default() -> Self {
        Self {
            max_loss_percent: -30.0,
            min_sol_balance: 0.01, // 0.01 SOL mínimo para fees
            min_asset_price: 0.0,   // Sin límite por defecto
            enabled: true,
        }
    }
}

/// Estado de una posición activa
#[derive(Debug, Clone)]
pub struct Position {
    pub token_mint: String,
    pub symbol: String,
    pub entry_price: f64,
    pub amount_invested: f64, // En SOL
    pub current_price: f64,
    pub current_value: f64,   // En SOL
}

impl Position {
    /// Calcula el drawdown actual en %
    pub fn drawdown_percent(&self) -> f64 {
        ((self.current_value - self.amount_invested) / self.amount_invested) * 100.0
    }

    /// Verifica si la posición necesita stop-loss
    pub fn needs_stop_loss(&self, config: &EmergencyConfig) -> bool {
        if !config.enabled {
            return false;
        }
        
        let dd = self.drawdown_percent();
        dd <= config.max_loss_percent
    }

    /// Verifica si el precio está en zona de pánico
    pub fn needs_panic_sell(&self, config: &EmergencyConfig) -> bool {
        if !config.enabled || config.min_asset_price == 0.0 {
            return false;
        }
        
        self.current_price < config.min_asset_price
    }
}

/// Sistema de monitoreo de emergencia
pub struct EmergencyMonitor {
    config: EmergencyConfig,
    positions: Vec<Position>,
    last_check: u64,
}

impl EmergencyMonitor {
    pub fn new(config: EmergencyConfig) -> Self {
        Self {
            config,
            positions: Vec::new(),
            last_check: Self::now(),
        }
    }

    /// Añade una posición al monitoreo
    pub fn add_position(&mut self, position: Position) {
        println!("🔍 Monitoreando nueva posición:");
        println!("   • Token: {}", position.token_mint);
        println!("   • Entrada: ${:.8}", position.entry_price);
        println!("   • Inversión: {:.4} SOL", position.amount_invested);
        println!("   • Stop Loss: {:.1}%\n", self.config.max_loss_percent);
        
        self.positions.push(position);
    }

    /// Actualiza el precio de una posición
    pub fn update_position(&mut self, token_mint: &str, current_price: f64, current_value: f64) {
        if let Some(pos) = self.positions.iter_mut().find(|p| p.token_mint == token_mint) {
            pos.current_price = current_price;
            pos.current_value = current_value;
        }
    }

    /// Obtiene una referencia a una posición por nombre
    pub fn get_position(&self, token_mint: &str) -> Option<&Position> {
        self.positions.iter().find(|p| p.token_mint == token_mint)
    }

    /// Obtiene todas las posiciones activas
    pub fn get_all_positions(&self) -> Vec<Position> {
        self.positions.clone()
    }

    /// Verifica todas las posiciones y detecta emergencias
    pub fn check_emergencies(&mut self) -> Vec<EmergencyAlert> {
        let mut alerts = Vec::new();
        let now = Self::now();
        
        // Solo verificar cada 5 segundos para no saturar
        if now - self.last_check < 5 {
            return alerts;
        }
        
        self.last_check = now;
        
        for pos in &self.positions {
            // Check Stop Loss
            if pos.needs_stop_loss(&self.config) {
                alerts.push(EmergencyAlert {
                    alert_type: AlertType::StopLoss,
                    token_mint: pos.token_mint.clone(),
                    drawdown: pos.drawdown_percent(),
                    current_price: pos.current_price,
                    message: format!(
                        "⚠️ STOP LOSS ACTIVADO: {} @ ${:.8} (Drawdown: {:.1}%)",
                        pos.token_mint, pos.current_price, pos.drawdown_percent()
                    ),
                });
            }
            
            // Check Panic Sell
            if pos.needs_panic_sell(&self.config) {
                alerts.push(EmergencyAlert {
                    alert_type: AlertType::PanicSell,
                    token_mint: pos.token_mint.clone(),
                    drawdown: pos.drawdown_percent(),
                    current_price: pos.current_price,
                    message: format!(
                        "🚨 PANIC SELL: {} cayó a ${:.8}",
                        pos.token_mint, pos.current_price
                    ),
                });
            }
        }
        
        alerts
    }

    fn now() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }
}

/// Tipo de alerta de emergencia
#[derive(Debug, Clone, PartialEq)]
pub enum AlertType {
    StopLoss,
    PanicSell,
    LowBalance,
}

/// Alerta de emergencia
#[derive(Debug, Clone)]
pub struct EmergencyAlert {
    pub alert_type: AlertType,
    pub token_mint: String,
    pub drawdown: f64,
    pub current_price: f64,
    pub message: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_drawdown() {
        let pos = Position {
            token_mint: "TEST".to_string(),
            symbol: "TEST".to_string(),
            entry_price: 1.0,
            amount_invested: 0.1,
            current_price: 0.7,
            current_value: 0.07,
        };
        
        assert_eq!(pos.drawdown_percent(), -30.0);
    }

    #[test]
    fn test_stop_loss_trigger() {
        let config = EmergencyConfig {
            max_loss_percent: -30.0,
            ..Default::default()
        };
        
        let pos = Position {
            token_mint: "TEST".to_string(),
            symbol: "TEST".to_string(),
            entry_price: 1.0,
            amount_invested: 0.1,
            current_price: 0.7,
            current_value: 0.07,
        };
        
        assert!(pos.needs_stop_loss(&config));
    }
}
