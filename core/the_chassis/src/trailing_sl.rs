//! # Trailing Stop-Loss System
//! 
//! Sistema inteligente que ajusta el stop-loss dinámicamente cuando el precio sube

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrailingStopLoss {
    /// Precio de entrada original
    pub entry_price: f64,
    
    /// Stop-loss inicial (porcentaje negativo, ej: -50.0)
    pub initial_sl_percent: f64,
    
    /// Precio máximo alcanzado desde la entrada
    pub peak_price: f64,
    
    /// Stop-loss actual (se ajusta automáticamente)
    pub current_sl_percent: f64,
    
    /// Porcentaje de retroceso permitido desde el pico (ej: 30.0 = permite caer 30% desde el pico)
    pub trailing_distance_percent: f64,
    
    /// Si el trailing está activado
    pub enabled: bool,
    
    /// Ganancia mínima para activar trailing (ej: 50.0 = activar cuando suba 50%)
    pub activation_threshold_percent: f64,
}

impl TrailingStopLoss {
    /// Crea un nuevo trailing stop-loss
    pub fn new(
        entry_price: f64,
        initial_sl_percent: f64,
        trailing_distance_percent: f64,
        activation_threshold_percent: f64,
    ) -> Self {
        Self {
            entry_price,
            initial_sl_percent,
            peak_price: entry_price,
            current_sl_percent: initial_sl_percent,
            trailing_distance_percent,
            enabled: false,
            activation_threshold_percent,
        }
    }

    /// Actualiza el trailing SL con un nuevo precio
    /// Retorna true si el SL cambió
    pub fn update(&mut self, current_price: f64) -> bool {
        let mut changed = false;

        // Actualizar precio pico
        if current_price > self.peak_price {
            self.peak_price = current_price;
            changed = true;
        }

        // Calcular ganancia desde entrada
        let gain_percent = ((current_price - self.entry_price) / self.entry_price) * 100.0;

        // Activar trailing si alcanzamos el threshold
        if !self.enabled && gain_percent >= self.activation_threshold_percent {
            self.enabled = true;
            println!("🎯 Trailing Stop-Loss ACTIVADO en +{:.2}%", gain_percent);
            changed = true;
        }

        // Si está activado, ajustar el SL
        if self.enabled {
            // Calcular nuevo SL basado en el pico
            let new_sl_price = self.peak_price * (1.0 - (self.trailing_distance_percent / 100.0));
            let new_sl_percent = ((new_sl_price - self.entry_price) / self.entry_price) * 100.0;

            // Actualizar solo si el nuevo SL es más alto que el actual
            if new_sl_percent > self.current_sl_percent {
                let old_sl = self.current_sl_percent;
                self.current_sl_percent = new_sl_percent;
                println!("📈 Stop-Loss ajustado: {:.2}% → {:.2}%", old_sl, new_sl_percent);
                changed = true;
            }
        }

        changed
    }

    /// Verifica si el precio actual activó el stop-loss
    pub fn is_triggered(&self, current_price: f64) -> bool {
        let current_percent = ((current_price - self.entry_price) / self.entry_price) * 100.0;
        current_percent <= self.current_sl_percent
    }

    /// Obtiene el precio de activación del SL actual
    pub fn get_sl_price(&self) -> f64 {
        self.entry_price * (1.0 + (self.current_sl_percent / 100.0))
    }

    /// Obtiene el estado actual como string legible
    pub fn status_string(&self) -> String {
        if self.enabled {
            format!(
                "🟢 ACTIVO | SL: {:.2}% | Pico: ${:.8} | Ganancia: +{:.2}%",
                self.current_sl_percent,
                self.peak_price,
                ((self.peak_price - self.entry_price) / self.entry_price) * 100.0
            )
        } else {
            format!(
                "⚪ INACTIVO | SL: {:.2}% | Activación: +{:.1}%",
                self.current_sl_percent,
                self.activation_threshold_percent
            )
        }
    }

    /// Reinicia el trailing (útil si querés volver a empezar)
    pub fn reset(&mut self) {
        self.peak_price = self.entry_price;
        self.current_sl_percent = self.initial_sl_percent;
        self.enabled = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trailing_activation() {
        let mut tsl = TrailingStopLoss::new(
            1.0,      // entry
            -50.0,    // initial SL
            30.0,     // trailing distance
            100.0,    // activation threshold (+100%)
        );

        // Subimos a +100%, debería activarse
        assert!(tsl.update(2.0));
        assert!(tsl.enabled);
    }

    #[test]
    fn test_trailing_adjustment() {
        let mut tsl = TrailingStopLoss::new(1.0, -50.0, 30.0, 50.0);
        
        // Activar
        tsl.update(1.5); // +50%
        assert!(tsl.enabled);

        // Subir más
        tsl.update(2.0); // +100%
        
        // El SL debería haberse ajustado hacia arriba
        assert!(tsl.current_sl_percent > -50.0);
    }

    #[test]
    fn test_sl_trigger() {
        let mut tsl = TrailingStopLoss::new(1.0, -50.0, 30.0, 50.0);
        tsl.update(1.5); // Activar
        tsl.update(2.0); // Pico a $2

        // El SL ahora está en: $2 * (1 - 0.30) = $1.40
        // Si el precio cae a $1.30, debería disparar
        assert!(tsl.is_triggered(1.30));
    }
}
