//! # Polymarket Bot — CLI Entry Point
//!
//! Punto de entrada principal con subcomandos para operar el bot:
//! - `markets`: Listar mercados disponibles
//! - `positions`: Ver posiciones abiertas
//! - `serve`: Arrancar el servidor gRPC

use clap::{Parser, Subcommand};
use tracing_subscriber::EnvFilter;
use anyhow::Result;

#[derive(Parser)]
#[command(
    name = "polymarket-bot",
    version = "0.1.0",
    author = "Ruben",
    about = "🎯 Polymarket Prediction Engine — Bot de trading para mercados de predicción"
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Listar mercados de predicción disponibles
    Markets {
        /// Número máximo de mercados a mostrar
        #[arg(short, long, default_value = "20")]
        limit: usize,
    },
    /// Ver posiciones abiertas
    Positions,
    /// Arrancar el servidor gRPC
    Serve,
    /// Mostrar configuración actual
    Config,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Inicializar sistema de observabilidad
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info"))
        )
        .with_target(true)
        .json()
        .init();

    let cli = Cli::parse();

    // Cargar configuración
    let config = polymarket_bot::PolymarketConfig::from_env()?;

    match cli.command {
        Some(Commands::Markets { limit }) => {
            tracing::info!("📊 Consultando mercados de Polymarket (límite: {})...", limit);
            let client = polymarket_bot::client::PolymarketClient::new(&config.api)?;
            match client.get_markets(limit).await {
                Ok(markets) => {
                    println!("\n🎯 Mercados de Predicción ({} encontrados):", markets.len());
                    println!("{}", "═".repeat(80));
                    for m in &markets {
                        println!(
                            "  {} | YES: {:.1}% | NO: {:.1}% | Vol: ${:.0} | {}",
                            m.question,
                            m.yes_price * 100.0,
                            m.no_price * 100.0,
                            m.volume,
                            m.category
                        );
                    }
                    println!("{}", "═".repeat(80));
                }
                Err(e) => {
                    tracing::error!("❌ Error obteniendo mercados: {}", e);
                }
            }
        }
        Some(Commands::Positions) => {
            tracing::info!("📋 Consultando posiciones abiertas...");
            println!("\n📋 Posiciones abiertas:");
            println!("{}", "═".repeat(80));
            println!("  (Sin posiciones — conectar API para datos en vivo)");
            println!("{}", "═".repeat(80));
        }
        Some(Commands::Serve) => {
            tracing::info!("🚀 Arrancando servidor gRPC en {}...", config.grpc.listen_addr);
            println!("🎯 Polymarket Bot — Servidor gRPC");
            println!("   Dirección: {}", config.grpc.listen_addr);
            println!("   API: {}", config.api.rest_url);
            println!("   Max posiciones: {}", config.risk.max_open_positions);
            println!("   Max por posición: {} USDC", config.risk.max_position_usdc);
            println!("\n⏳ Servidor gRPC pendiente de implementación completa...");
            // TODO: Implementar tonic::Server con PolymarketBot service
        }
        Some(Commands::Config) => {
            println!("\n⚙️  Configuración actual:");
            println!("{}", "═".repeat(50));
            println!("  API REST:       {}", config.api.rest_url);
            println!("  API WebSocket:  {}", config.api.ws_url);
            println!("  gRPC Address:   {}", config.grpc.listen_addr);
            println!("  Max Positions:  {}", config.risk.max_open_positions);
            println!("  Max per Pos:    {} USDC", config.risk.max_position_usdc);
            println!("  Stop-Loss:      {}%", config.risk.default_stop_loss_pct);
            println!("  Min Edge:       {}%", config.risk.min_edge_threshold * 100.0);
            println!("{}", "═".repeat(50));
        }
        None => {
            println!("🎯 Polymarket Prediction Engine v0.1.0");
            println!("   Usa --help para ver comandos disponibles");
        }
    }

    Ok(())
}
