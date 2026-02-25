//! # Liquidity Monitor - Detector de Ballenas y Movimientos Sospechosos
//!
//! Monitorea cambios dramáticos en liquidez y volumen para detectar señales de peligro

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiquiditySnapshot {
    pub timestamp: i64,
    pub liquidity_usd: f64,
    pub volume_24h: f64,
    pub price_usd: f64,
    pub holders_count: Option<u64>,
}

pub struct LiquidityMonitor {
    /// Historial de snapshots (últimos 10)
    history: Vec<LiquiditySnapshot>,

    /// Umbral de caída de liquidez para alertar (porcentaje)
    liquidity_drop_threshold: f64,

    /// Umbral de spike de volumen sospechoso (múltiplo del promedio)
    volume_spike_multiplier: f64,
}

#[derive(Debug)]
pub enum LiquidityAlert {
    /// Caída dramática de liquidez
    LiquidityDrop {
        from_usd: f64,
        to_usd: f64,
        percent: f64,
    },

    /// Spike sospechoso de volumen
    VolumeSuspicious {
        current: f64,
        average: f64,
        multiplier: f64,
    },

    /// Caída de precio + caída de liquidez = señal de rug pull potencial
    RugPullWarning {
        price_drop: f64,
        liquidity_drop: f64,
    },
}

impl LiquidityMonitor {
    pub fn new(liquidity_drop_threshold: f64, volume_spike_multiplier: f64) -> Self {
        Self {
            history: Vec::new(),
            liquidity_drop_threshold,
            volume_spike_multiplier,
        }
    }

    /// Añade un nuevo snapshot y analiza
    pub fn add_snapshot(&mut self, snapshot: LiquiditySnapshot) -> Vec<LiquidityAlert> {
        let mut alerts = Vec::new();

        // Si tenemos historial, comparar
        if let Some(prev) = self.history.last() {
            // Detectar caída de liquidez
            if snapshot.liquidity_usd < prev.liquidity_usd {
                let drop_percent =
                    ((prev.liquidity_usd - snapshot.liquidity_usd) / prev.liquidity_usd) * 100.0;

                if drop_percent >= self.liquidity_drop_threshold {
                    alerts.push(LiquidityAlert::LiquidityDrop {
                        from_usd: prev.liquidity_usd,
                        to_usd: snapshot.liquidity_usd,
                        percent: drop_percent,
                    });
                }

                // Detectar rug pull potencial: caída de precio + caída de liquidez
                if snapshot.price_usd < prev.price_usd {
                    let price_drop =
                        ((prev.price_usd - snapshot.price_usd) / prev.price_usd) * 100.0;

                    if drop_percent > 20.0 && price_drop > 30.0 {
                        alerts.push(LiquidityAlert::RugPullWarning {
                            price_drop,
                            liquidity_drop: drop_percent,
                        });
                    }
                }
            }

            // Detectar spike de volumen sospechoso
            if self.history.len() >= 3 {
                let avg_volume: f64 = self.history.iter().map(|s| s.volume_24h).sum::<f64>()
                    / self.history.len() as f64;

                if snapshot.volume_24h > avg_volume * self.volume_spike_multiplier {
                    alerts.push(LiquidityAlert::VolumeSuspicious {
                        current: snapshot.volume_24h,
                        average: avg_volume,
                        multiplier: snapshot.volume_24h / avg_volume,
                    });
                }
            }
        }

        // Guardar snapshot (máximo 10)
        self.history.push(snapshot);
        if self.history.len() > 10 {
            self.history.remove(0);
        }

        alerts
    }

    /// Obtiene el snapshot más reciente
    pub fn latest_snapshot(&self) -> Option<&LiquiditySnapshot> {
        self.history.last()
    }

    /// Calcula la tendencia de liquidez (positiva/negativa)
    pub fn liquidity_trend(&self) -> Option<f64> {
        if self.history.len() < 2 {
            return None;
        }

        let first = self.history.first().unwrap().liquidity_usd;
        let last = self.history.last().unwrap().liquidity_usd;

        Some(((last - first) / first) * 100.0)
    }
}

impl LiquidityAlert {
    /// Convierte la alerta a un mensaje formateado para Telegram
    pub fn to_telegram_message(&self, token_symbol: &str) -> String {
        match self {
            LiquidityAlert::LiquidityDrop {
                from_usd,
                to_usd,
                percent,
            } => {
                format!(
                    "<b>⚠️ ALERTA DE LIQUIDEZ — {}</b>\n\
                    <b>━━━━━━━━━━━━━━━━━━━━━━</b>\n\
                    <b>⬡ Caída de liquidez:</b> <b>-{:.2}%</b>\n\
                    <b>⬡ Antes:</b> <code>${:.0}</code>\n\
                    <b>⬡ Ahora:</b> <code>${:.0}</code>\n\n\
                    <i>🔍 Esto puede indicar ventas grandes o retiro de LP.</i>",
                    token_symbol, percent, from_usd, to_usd
                )
            }

            LiquidityAlert::VolumeSuspicious {
                current,
                average,
                multiplier,
            } => {
                format!(
                    "<b>📊 VOLUMEN ANORMAL — {}</b>\n\
                    <b>━━━━━━━━━━━━━━━━━━━━━━</b>\n\
                    <b>⬡ Spike de volumen:</b> <b>{:.1}x</b> del promedio\n\
                    <b>⬡ Actual 24h:</b> <code>${:.0}</code>\n\
                    <b>⬡ Promedio:</b> <code>${:.0}</code>\n\n\
                    <i>⚠️ Puede indicar actividad de ballenas o dump inminente.</i>",
                    token_symbol, multiplier, current, average
                )
            }

            LiquidityAlert::RugPullWarning {
                price_drop,
                liquidity_drop,
            } => {
                format!(
                    "<b>🚨🚨 ADVERTENCIA DE RUG PULL — {} 🚨🚨</b>\n\
                    <b>━━━━━━━━━━━━━━━━━━━━━━</b>\n\
                    <b>❌ Precio:</b> <b>-{:.1}%</b>\n\
                    <b>❌ Liquidez:</b> <b>-{:.1}%</b>\n\n\
                    <b>⚡ ACCIÓN INMEDIATA RECOMENDADA</b>\n\
                    <i>Considera salir de la posición ahora.</i>",
                    token_symbol, price_drop, liquidity_drop
                )
            }
        }
    }

    /// Obtiene el nivel de severidad (1-3)
    pub fn severity(&self) -> u8 {
        match self {
            LiquidityAlert::LiquidityDrop { percent, .. } => {
                if *percent > 50.0 {
                    3
                } else if *percent > 30.0 {
                    2
                } else {
                    1
                }
            }
            LiquidityAlert::VolumeSuspicious { multiplier, .. } => {
                if *multiplier > 10.0 {
                    3
                } else if *multiplier > 5.0 {
                    2
                } else {
                    1
                }
            }
            LiquidityAlert::RugPullWarning { .. } => 3, // Máxima severidad
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_liquidity_drop_detection() {
        let mut monitor = LiquidityMonitor::new(20.0, 5.0);

        // Snapshot inicial
        monitor.add_snapshot(LiquiditySnapshot {
            timestamp: 1000,
            liquidity_usd: 100_000.0,
            volume_24h: 50_000.0,
            price_usd: 1.0,
            holders_count: None,
        });

        // Caída del 30%
        let alerts = monitor.add_snapshot(LiquiditySnapshot {
            timestamp: 2000,
            liquidity_usd: 70_000.0,
            volume_24h: 50_000.0,
            price_usd: 1.0,
            holders_count: None,
        });

        assert_eq!(alerts.len(), 1);
        assert!(matches!(alerts[0], LiquidityAlert::LiquidityDrop { .. }));
    }

    #[test]
    fn test_rug_pull_warning() {
        let mut monitor = LiquidityMonitor::new(20.0, 5.0);

        monitor.add_snapshot(LiquiditySnapshot {
            timestamp: 1000,
            liquidity_usd: 100_000.0,
            volume_24h: 50_000.0,
            price_usd: 1.0,
            holders_count: None,
        });

        // Caída dramática de ambos
        let alerts = monitor.add_snapshot(LiquiditySnapshot {
            timestamp: 2000,
            liquidity_usd: 50_000.0, // -50%
            volume_24h: 50_000.0,
            price_usd: 0.6, // -40%
            holders_count: None,
        });

        // Debería detectar rug pull warning
        assert!(alerts
            .iter()
            .any(|a| matches!(a, LiquidityAlert::RugPullWarning { .. })));
    }
}
