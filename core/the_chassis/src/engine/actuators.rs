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
            // Free Tier / Bootstrapping adjustments: 
            // We use higher base tips to compensate for WS latency vs gRPC bots
            base_tip_lamports: 200_000, // 0.0002 SOL base (increased from 0.0001)
            max_tip_lamports: 10_000_000, // 0.01 SOL max cap (increased to secure block execution)
            slope_multiplier: 2_000_000.0, // Escala más agresiva
        }
    }

    /// Calcula el tip óptimo basado en momentum y etapa
    pub fn calculate_tip(&self, momentum_slope: f64, stage: MaturityStage) -> u64 {
        let urgency_factor = match stage {
            MaturityStage::EarlyHighRisk => 2.0, // 100% extra en fase temprana
            MaturityStage::MomentumCore => 1.5,  // 50% extra en core
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
    base_bps: u16,     // 300 bps = 3%
    max_bps: u16,      // 2000 bps = 20%
    slope_factor: f64, // Factor de escalado
}

impl AdaptiveSlippageCalculator {
    pub fn new() -> Self {
        Self {
            // Ajustado para Free Tier: más slippage asume datos menos frescos (latencia WS)
            base_bps: 300,      // 3% base (incrementado desde 2%)
            max_bps: 2000,      // 20% max (seguridad incrementada para no fallar tx)
            slope_factor: 2500.0, // 0.1 slope -> +250 bps (+2.5%)
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
        assert_eq!(calc.calculate_tip(0.0, MaturityStage::MomentumCore), 200_000);

        // Caso 2: High Momentum (Slope 0.5) en Early Stage
        // Base (200k) + (0.5 * 2M * 2.0) = 200k + 2000k = 2.2M
        let tip = calc.calculate_tip(0.5, MaturityStage::EarlyHighRisk);
        assert!(tip >= 2_200_000);
        assert!(tip <= 10_000_000); // Respetar cap
    }

    #[test]
    fn test_adaptive_slippage() {
        let calc = AdaptiveSlippageCalculator::new();

        // Caso 1: Normal
        assert_eq!(calc.calculate_slippage(0.0, MaturityStage::LateReversal), 300);

        // Caso 2: PUMP violento (Slope 1.0)
        // Base (300) + (1.0 * 2500) = 2800 -> Cap at 2000
        let slippage = calc.calculate_slippage(1.0, MaturityStage::MomentumCore);
        assert_eq!(slippage, 2000);
    }
}
