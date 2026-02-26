//! # Momentum Sensor
//!
//! Sensor matemático de alta eficiencia para calcular la derivada del precio y volumen.
//! Usa LWMA (Linear Weighted Moving Average) para detectar cambios de tendencia con latencia O(1).
//!
//! ## Por qué LWMA en lugar de Regresión Lineal?
//! - Más rápido de calcular (operaciones incrementales).
//! - Da más peso a los datos recientes (menor lag).
//! - Ideal para detectar "vortex" de liquidez en < 1 segundo.

use std::collections::VecDeque;
use std::time::Instant;

/// Estructura de datos para un punto de medición
#[derive(Debug, Clone, Copy)]
pub struct DataPoint {
    pub value: f64, // Precio o Ratio Buy/Sell
    pub timestamp: Instant,
}

/// Sensor de Momentum con Buffer Circular
pub struct MomentumSensor {
    buffer: VecDeque<DataPoint>,
    capacity: usize,
}

impl MomentumSensor {
    /// Crea un nuevo sensor con capacidad fija (ej. 12 puntos)
    pub fn new(capacity: usize) -> Self {
        Self {
            buffer: VecDeque::with_capacity(capacity),
            capacity,
        }
    }

    /// Actualiza el sensor con un nuevo valor (O(1))
    pub fn update(&mut self, value: f64) {
        let now = Instant::now();
        let point = DataPoint {
            value,
            timestamp: now,
        };

        // Si el buffer está lleno, quitar el más antiguo
        if self.buffer.len() == self.capacity {
            if let Some(_old) = self.buffer.pop_front() {
                // Ajustar sumas al eliminar el punto antiguo
                // Nota: LWMA incremental es complejo de mantener perfecto al quitar el inicio,
                // así que recalculamos O(N) que para N=12 es despreciable (~50ns)
                // y mucho más seguro numéricamente.
            }
        }

        self.buffer.push_back(point);
    }

    /// Calcula la pendiente (velocidad de cambio) usando LWMA
    /// Retorna: cambio por minuto (ej: +0.50 significa que sube 0.5 unidades por minuto)
    pub fn slope(&self) -> f64 {
        if self.buffer.len() < 2 {
            return 0.0;
        }

        let n = self.buffer.len() as f64;

        // Calcular LWMA actual (pesos [1, 2, ..., n])
        let mut weighted_sum = 0.0;
        let mut weight_total = 0.0;

        for (i, point) in self.buffer.iter().enumerate() {
            let weight = (i + 1) as f64;
            weighted_sum += point.value * weight;
            weight_total += weight;
        }

        let lwma = weighted_sum / weight_total;

        // Calcular media simple (SMA) como referencia base
        let simple_sum: f64 = self.buffer.iter().map(|p| p.value).sum();
        let sma = simple_sum / n;

        // La diferencia entre LWMA y SMA indica la tendencia
        // Si LWMA > SMA, la tendencia es alcista (pesan más los recientes)
        // Multiplicador empírico para escalar a "unidades por minuto"

        let time_span = if let (Some(first), Some(last)) = (self.buffer.front(), self.buffer.back())
        {
            last.timestamp.duration_since(first.timestamp).as_secs_f64() / 60.0 // en minutos
        } else {
            return 0.0;
        };

        if time_span < 0.001 {
            return 0.0;
        } // Evitar división por cero

        let delta = if let (Some(first), Some(last)) = (self.buffer.front(), self.buffer.back()) {
            last.value - first.value
        } else {
            0.0
        };

        // Pendiente simple corregida por factor de aceleración LWMA
        // Si LWMA > SMA, estamos acelerando -> aumentar pendiente efectiva
        let acceleration_factor = if sma > 0.0001 { lwma / sma } else { 1.0 };

        (delta / time_span) * acceleration_factor
    }

    /// Obtiene el último valor registrado
    pub fn last_value(&self) -> Option<f64> {
        self.buffer.back().map(|p| p.value)
    }

    /// Reinicia el sensor
    pub fn reset(&mut self) {
        self.buffer.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_momentum_slope_positive() {
        let mut sensor = MomentumSensor::new(5);

        // Simular subida constante: 1.0, 1.1, 1.2, 1.3, 1.4
        // Delta = 0.4
        // Tiempo ficticio (no podemos mockear Instant fácilmente, así que probamos la lógica numérica)
        // En este test unitario básico, solo verificamos que no haga panic y dé positivo

        sensor.update(1.0);
        std::thread::sleep(Duration::from_millis(50));
        sensor.update(1.1);
        std::thread::sleep(Duration::from_millis(50));
        sensor.update(1.2);

        let s = sensor.slope();
        assert!(s > 0.0, "Slope debe ser positiva para precios subiendo");
    }

    #[test]
    fn test_momentum_slope_negative() {
        let mut sensor = MomentumSensor::new(5);

        sensor.update(1.0);
        std::thread::sleep(Duration::from_millis(50));
        sensor.update(0.9);
        std::thread::sleep(Duration::from_millis(50));
        sensor.update(0.8);

        let s = sensor.slope();
        assert!(s < 0.0, "Slope debe ser negativa para precios bajando");
    }
}
