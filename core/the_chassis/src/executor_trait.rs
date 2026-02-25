//! # Executor Trait - The Swiss Standard
//!
//! Abstracción polimórfica para sistemas de ejecución de swaps.
//! Permite cambiar entre DEXs (Jupiter, Raydium, Orca) sin modificar la lógica de negocio.
//!
//! ## Filosofía
//! "El que controla la abstracción, controla el sistema."
//!
//! Este trait define el estándar de calidad suiza para ejecutores.
//! Cualquier implementación debe cumplir con este contrato.

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use solana_sdk::{pubkey::Pubkey, signature::Keypair};

/// Quote de un swap (cotización)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quote {
    /// Mint del token de entrada
    pub input_mint: String,
    /// Mint del token de salida
    pub output_mint: String,
    /// Cantidad de entrada (en unidades base)
    pub in_amount: u64,
    /// Cantidad estimada de salida (en unidades base)
    pub out_amount: u64,
    /// Impacto en el precio (%)
    pub price_impact_pct: f64,
    /// Ruta del swap (e.g., "Raydium → Orca")
    pub route: String,
    /// Slippage tolerable (basis points)
    pub slippage_bps: u16,
    /// Datos adicionales específicos del DEX (opaco)
    pub raw_data: Option<String>,
}

/// Resultado de una ejecución de swap
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwapExecution {
    /// Firma de la transacción
    pub signature: String,
    /// Cantidad real de entrada consumida
    pub actual_in: u64,
    /// Cantidad real de salida recibida
    pub actual_out: u64,
    /// Slippage real experimentado (%)
    pub actual_slippage_pct: f64,
    /// Latencia total desde quote hasta confirmación (ms)
    pub latency_ms: u64,
    /// DEX utilizado
    pub executor_name: String,
}

impl SwapExecution {
    /// Imprime un resumen ejecutivo del swap
    pub fn print_summary(&self) {
        println!("╔════════════════════════════════════════════════════════════╗");
        println!("║              ✅ SWAP EXECUTION CONFIRMED                 ║");
        println!("╚════════════════════════════════════════════════════════════╝");
        println!();
        println!("🔗 Signature:     {}", self.signature);
        println!("⚡ Executor:      {}", self.executor_name);
        println!("📊 Input:         {} units", self.actual_in);
        println!("💎 Output:        {} units", self.actual_out);
        println!("📉 Slippage:      {:.2}%", self.actual_slippage_pct);
        println!("⏱️  Latency:       {}ms", self.latency_ms);
        println!();
        println!("🔗 Solscan:       https://solscan.io/tx/{}", self.signature);
        println!();
    }
}

/// Trait principal para ejecutores de swaps
///
/// Cualquier DEX (Jupiter, Raydium, Orca) debe implementar este trait
/// para ser compatible con The Chassis.
#[async_trait]
pub trait Executor: Send + Sync {
    /// Nombre del executor (e.g., "Jupiter", "Raydium")
    fn name(&self) -> &str;

    /// Obtener una cotización para un swap
    ///
    /// # Argumentos
    /// * `input_mint` - Pubkey del token de entrada
    /// * `output_mint` - Pubkey del token de salida
    /// * `amount` - Cantidad a swappear (en unidades base)
    /// * `slippage_bps` - Slippage tolerado en basis points (100 = 1%)
    ///
    /// # Returns
    /// Cotización con la mejor ruta encontrada
    async fn get_quote(
        &self,
        input_mint: &Pubkey,
        output_mint: &Pubkey,
        amount: u64,
        slippage_bps: u16,
    ) -> Result<Quote>;

    /// Ejecutar un swap basado en una cotización
    ///
    /// # Argumentos
    /// * `quote` - Cotización obtenida previamente
    /// * `wallet` - Keypair de la wallet que ejecutará el swap
    /// * `auto_unwrap` - Si es true, convierte WSOL → SOL nativo automáticamente
    ///
    /// # Returns
    /// Resultado de la ejecución con signature y métricas
    async fn execute_swap(
        &self,
        quote: &Quote,
        wallet: &Keypair,
        auto_unwrap: bool,
    ) -> Result<SwapExecution>;

    /// Verificar si el executor está disponible (health check)
    ///
    /// # Returns
    /// true si el executor puede procesar swaps, false si está caído
    async fn is_healthy(&self) -> bool;

    /// Obtener la latencia promedio del executor (ms)
    ///
    /// Útil para elegir el executor más rápido en caso de múltiples opciones.
    async fn avg_latency_ms(&self) -> u64;
}

/// Executor compuesto que puede hacer fallback entre múltiples DEXs
///
/// Ejemplo: Si Raydium falla, automáticamente intenta con Jupiter.
pub struct FallbackExecutor {
    primary: Box<dyn Executor>,
    fallback: Box<dyn Executor>,
}

impl FallbackExecutor {
    pub fn new(primary: Box<dyn Executor>, fallback: Box<dyn Executor>) -> Self {
        Self { primary, fallback }
    }
}

#[async_trait]
impl Executor for FallbackExecutor {
    fn name(&self) -> &str {
        "FallbackExecutor"
    }

    async fn get_quote(
        &self,
        input_mint: &Pubkey,
        output_mint: &Pubkey,
        amount: u64,
        slippage_bps: u16,
    ) -> Result<Quote> {
        match self
            .primary
            .get_quote(input_mint, output_mint, amount, slippage_bps)
            .await
        {
            Ok(quote) => Ok(quote),
            Err(e) => {
                eprintln!("⚠️  Primary executor failed, trying fallback: {}", e);
                self.fallback
                    .get_quote(input_mint, output_mint, amount, slippage_bps)
                    .await
            }
        }
    }

    async fn execute_swap(
        &self,
        quote: &Quote,
        wallet: &Keypair,
        auto_unwrap: bool,
    ) -> Result<SwapExecution> {
        match self.primary.execute_swap(quote, wallet, auto_unwrap).await {
            Ok(result) => Ok(result),
            Err(e) => {
                eprintln!("⚠️  Primary executor failed, trying fallback: {}", e);
                self.fallback.execute_swap(quote, wallet, auto_unwrap).await
            }
        }
    }

    async fn is_healthy(&self) -> bool {
        self.primary.is_healthy().await || self.fallback.is_healthy().await
    }

    async fn avg_latency_ms(&self) -> u64 {
        // Devolver la latencia del más rápido
        std::cmp::min(
            self.primary.avg_latency_ms().await,
            self.fallback.avg_latency_ms().await,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mock executor para testing
    struct MockExecutor {
        name: String,
        healthy: bool,
        latency: u64,
    }

    #[async_trait]
    impl Executor for MockExecutor {
        fn name(&self) -> &str {
            &self.name
        }

        async fn get_quote(
            &self,
            _input_mint: &Pubkey,
            _output_mint: &Pubkey,
            _amount: u64,
            _slippage_bps: u16,
        ) -> Result<Quote> {
            if self.healthy {
                Ok(Quote {
                    input_mint: "SOL".to_string(),
                    output_mint: "USDC".to_string(),
                    in_amount: 1000000000,
                    out_amount: 100000000,
                    price_impact_pct: 0.5,
                    route: "Mock".to_string(),
                    slippage_bps: 100,
                    raw_data: None,
                })
            } else {
                anyhow::bail!("Executor unhealthy")
            }
        }

        async fn execute_swap(
            &self,
            _quote: &Quote,
            _wallet: &Keypair,
            _auto_unwrap: bool,
        ) -> Result<SwapExecution> {
            anyhow::bail!("Not implemented in mock")
        }

        async fn is_healthy(&self) -> bool {
            self.healthy
        }

        async fn avg_latency_ms(&self) -> u64 {
            self.latency
        }
    }

    #[tokio::test]
    async fn test_fallback_executor() {
        let primary = MockExecutor {
            name: "Primary".to_string(),
            healthy: false,
            latency: 100,
        };

        let fallback = MockExecutor {
            name: "Fallback".to_string(),
            healthy: true,
            latency: 200,
        };

        let executor = FallbackExecutor::new(Box::new(primary), Box::new(fallback));

        let sol_mint = Pubkey::new_unique();
        let usdc_mint = Pubkey::new_unique();

        let quote = executor
            .get_quote(&sol_mint, &usdc_mint, 1000000000, 100)
            .await;
        assert!(quote.is_ok());
    }
}
