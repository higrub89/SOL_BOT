//! # Actuadores Inteligentes HFT
//! 
//! Módulos responsables de ejecutar las decisiones del Engine con parámetros dinámicos.
//! Incluye cálculo de propinas Jito (Priority Fee) y Slippage adaptativo.

use crate::engine::types::{MaturityStage};

/// Calculadora de Propinas Jito Dinámica
/// Ajusta el Priority Fee según la urgencia del Momentum
pub struct DynamicTipCalculator {
    base_tip_lamports: u64,
    max_tip_lamports: u64,
    slope_multiplier: f64,
}

impl DynamicTipCalculator {
    pub fn new() -> Self {
        Self {
            base_tip_lamports: 100_000, // 0.0001 SOL base
            max_tip_lamports: 5_000_000, // 0.005 SOL max cap
            slope_multiplier: 1_000_000.0, // Escala la pendiente a lamports
        }
    }

    /// Calcula el tip óptimo basado en momentum y etapa
    pub fn calculate_tip(&self, momentum_slope: f64, stage: MaturityStage) -> u64 {
        let urgency_factor = match stage {
            MaturityStage::EarlyHighRisk => 1.5, // 50% extra en fase temprana
            MaturityStage::MomentumCore => 1.2,  // 20% extra en core
            MaturityStage::LateReversal => 1.0,  // Base en late game
        };

        // Si la pendiente es negativa o muy baja, usar base
        if momentum_slope <= 0.0 {
            return self.base_tip_lamports;
        }

        // Tip = Base + (Slope * Multiplier * Urgency)
        let dynamic_part = momentum_slope * self.slope_multiplier * urgency_factor;
        let total_tip = self.base_tip_lamports as f64 + dynamic_part;

        // Aplicar Cap Máximo de Seguridad
        total_tip.min(self.max_tip_lamports as f64) as u64
    }
}

/// Calculadora de Slippage Adaptativo
/// Abre el margen de error si hay alta volatilidad (momentum fuerte)
pub struct AdaptiveSlippageCalculator {
    base_bps: u16,     // 200 bps = 2%
    max_bps: u16,      // 1500 bps = 15%
    slope_factor: f64, // Factor de escalado
}

impl AdaptiveSlippageCalculator {
    pub fn new() -> Self {
        Self {
            base_bps: 200,      // 2% base
            max_bps: 1500,      // 15% max (seguridad)
            slope_factor: 2000.0, // 0.1 slope -> +200 bps (+2%)
        }
    }

    /// Calcula el slippage en Basis Points (bps)
    pub fn calculate_slippage(&self, momentum_slope: f64, stage: MaturityStage) -> u16 {
        // En Early High Risk, permitimos más slippage para asegurar entrada
        let stage_base = match stage {
            MaturityStage::EarlyHighRisk => 500, // Empezar en 5%
            _ => self.base_bps,
        };

        if momentum_slope <= 0.0 {
            return stage_base;
        }

        // Slippage = Base + (Slope * Factor)
        let extra_bps = (momentum_slope * self.slope_factor) as u16;
        let total = stage_base + extra_bps;

        total.min(self.max_bps)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dynamic_tip() {
        let calc = DynamicTipCalculator::new();
        
        // Caso 1: Sin momentum (Base)
        assert_eq!(calc.calculate_tip(0.0, MaturityStage::MomentumCore), 100_000);

        // Caso 2: High Momentum (Slope 0.5) en Early Stage
        // Base (100k) + (0.5 * 1M * 1.5) = 100k + 750k = 850k
        let tip = calc.calculate_tip(0.5, MaturityStage::EarlyHighRisk);
        assert!(tip >= 850_000);
        assert!(tip <= 5_000_000); // Respetar cap
    }

    #[test]
    fn test_adaptive_slippage() {
        let calc = AdaptiveSlippageCalculator::new();

        // Caso 1: Normal
        assert_eq!(calc.calculate_slippage(0.0, MaturityStage::LateReversal), 200);

        // Caso 2: PUMP violento (Slope 1.0)
        // Base (200) + (1.0 * 2000) = 2200 -> Cap at 1500
        let slippage = calc.calculate_slippage(1.0, MaturityStage::MomentumCore);
        assert_eq!(slippage, 1500);
    }
}
