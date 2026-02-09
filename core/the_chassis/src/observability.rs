//! # Observability - Telemetría de Hiperlujo
//! 
//! Sistema de logging estructurado con niveles de detalle institucionales.
//! 
//! ## Niveles de Log
//! - **TRACE:** Debugging extremo (solo en desarrollo)
//! - **DEBUG:** Información de diagnóstico
//! - **INFO:** Eventos importantes del sistema
//! - **WARN:** Situaciones anómalas pero recuperables
//! - **ERROR:** Errores que requieren atención
//! 
//! ## Formato de Log Premium
//! ```
//! [2026-02-09 22:15:01.423][INFO][EXECUTOR-RAYDIUM] Swap Success | TX: 5ghZ... | Latency: 420ms | Slippage: 0.5%
//! ```

use tracing::{info, warn, error, debug, Level};
use tracing_subscriber::{
    fmt::{self, format::FmtSpan},
    EnvFilter,
    layer::SubscriberExt,
    util::SubscriberInitExt,
};
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use std::path::Path;

/// Configuración del sistema de observabilidad
pub struct ObservabilityConfig {
    /// Nivel de log mínimo (trace, debug, info, warn, error)
    pub log_level: Level,
    /// Directorio donde se guardarán los logs
    pub log_dir: String,
    /// Si se debe imprimir también en stdout
    pub stdout_enabled: bool,
    /// Si se debe usar formato JSON (para parsing automático)
    pub json_format: bool,
}

impl Default for ObservabilityConfig {
    fn default() -> Self {
        Self {
            log_level: Level::INFO,
            log_dir: "../../operational/logs".to_string(),
            stdout_enabled: true,
            json_format: false,
        }
    }
}

impl ObservabilityConfig {
    /// Configuración para producción (logs JSON rotativos)
    pub fn production() -> Self {
        Self {
            log_level: Level::INFO,
            log_dir: "../../operational/logs".to_string(),
            stdout_enabled: false,
            json_format: true,
        }
    }

    /// Configuración para desarrollo (logs verbosos en stdout)
    pub fn development() -> Self {
        Self {
            log_level: Level::DEBUG,
            log_dir: "../../operational/logs".to_string(),
            stdout_enabled: true,
            json_format: false,
        }
    }
}

/// Inicializa el sistema de observabilidad
pub fn init_observability(config: ObservabilityConfig) -> anyhow::Result<()> {
    // Crear el directorio de logs si no existe
    std::fs::create_dir_all(&config.log_dir)?;

    // Rolling file appender (rota diariamente)
    let file_appender = RollingFileAppender::new(
        Rotation::DAILY,
        &config.log_dir,
        "chassis.log",
    );

    // Filtro de niveles
    let env_filter = EnvFilter::from_default_env()
        .add_directive(config.log_level.into());

    // Layer de archivo
    let file_layer = fmt::layer()
        .with_writer(file_appender)
        .with_ansi(false)
        .with_target(true)
        .with_thread_ids(true)
        .with_line_number(true)
        .with_span_events(FmtSpan::CLOSE);

    // Construir subscriber
    if config.stdout_enabled {
        // Con stdout
        let stdout_layer = fmt::layer()
            .with_target(true)
            .with_thread_ids(false)
            .with_line_number(false);

        tracing_subscriber::registry()
            .with(env_filter)
            .with(file_layer)
            .with(stdout_layer)
            .init();
    } else {
        // Solo archivo
        tracing_subscriber::registry()
            .with(env_filter)
            .with(file_layer)
            .init();
    }

    info!("✅ Observability system initialized");
    info!("📁 Log directory: {}", config.log_dir);
    info!("📊 Log level: {:?}", config.log_level);

    Ok(())
}

/// Macros de conveniencia para logs de "hiperlujo"

/// Log de ejecución de swap con métricas completas
#[macro_export]
macro_rules! log_swap {
    ($executor:expr, $signature:expr, $latency_ms:expr, $slippage:expr) => {
        tracing::info!(
            executor = $executor,
            tx = $signature,
            latency_ms = $latency_ms,
            slippage_pct = $slippage,
            "Swap executed successfully"
        );
    };
}

/// Log de quote obtenido
#[macro_export]
macro_rules! log_quote {
    ($dex:expr, $in_amount:expr, $out_amount:expr, $price_impact:expr) => {
        tracing::debug!(
            dex = $dex,
            in_amount = $in_amount,
            out_amount = $out_amount,
            price_impact_pct = $price_impact,
            "Quote obtained"
        );
    };
}

/// Log de auditoría de token
#[macro_export]
macro_rules! log_audit {
    ($token:expr, $score:expr, $verdict:expr) => {
        tracing::info!(
            token_mint = $token,
            score = $score,
            verdict = $verdict,
            "Token audit completed"
        );
    };
}

/// Log de error con contexto
#[macro_export]
macro_rules! log_error {
    ($module:expr, $error:expr, $context:expr) => {
        tracing::error!(
            module = $module,
            error = %$error,
            context = $context,
            "Error occurred"
        );
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_observability_config() {
        let dev_config = ObservabilityConfig::development();
        assert_eq!(dev_config.log_level, Level::DEBUG);
        assert!(dev_config.stdout_enabled);

        let prod_config = ObservabilityConfig::production();
        assert_eq!(prod_config.log_level, Level::INFO);
        assert!(prod_config.json_format);
    }
}
