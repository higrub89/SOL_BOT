//! # The Chassis - Solana Trading Engine
//!
//! v2.0.0-HFT - Asynchronous Execution & Dynamic Configuration

use anyhow::Result;

use clap::{Parser, Subcommand};
use solana_sdk::program_pack::Pack;
use solana_sdk::signature::Keypair;
use spl_token::state::Account as TokenAccount;
use std::str::FromStr;
use std::sync::{Arc, Mutex};

// ----------------------------------------------------------------------------
// EXPORTED MODULES (API PÚBLICA)
// ----------------------------------------------------------------------------

pub mod amm_math;
pub mod auto_buyer;
pub mod config;
pub mod emergency;
pub mod executor_v2;
pub mod geyser;
pub mod jito;
pub mod jupiter;
pub mod liquidity_monitor;
pub mod price_feed;
pub mod raydium;
pub mod scanner;
pub mod state_manager;
pub mod telegram; // El módulo telegram ahora incluye commands internamente
pub mod telemetry_server;
pub mod trailing_sl;
pub mod validation;
pub mod wallet;
pub mod websocket;
pub mod ws_feed;

// 🏎️ Módulos del Framework Institucional (v2.0)
pub mod engine;
pub mod generated;
pub mod observability;
pub mod sensors;

// ----------------------------------------------------------------------------
// IMPORTS INTERNOS
// ----------------------------------------------------------------------------

use config::AppConfig;
use emergency::{EmergencyConfig, EmergencyMonitor, Position};
use executor_v2::{ExecutorConfig, TradeExecutor};

use price_feed::{MonitoredToken, PriceFeed, PriceFeedConfig};
use state_manager::StateManager;
use telegram::TelegramNotifier;
use telegram::commands::CommandHandler;

use wallet::{load_keypair_from_env, WalletMonitor};

/// Argumentos de línea de comandos para The Chassis
#[derive(Parser)]
#[command(name = "the_chassis")]
#[command(about = "Solana Trading Engine - Speed and Performance", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Ejecuta una compra inmediata de un token (via Jupiter - legacy)
    Buy {
        /// Mint address del token
        #[arg(short, long)]
        mint: String,

        /// Cantidad de SOL a invertir
        #[arg(short, long)]
        sol: f64,

        /// Slippage en bps (100 = 1.0%)
        #[arg(long, default_value_t = 100)]
        slippage: u16,
    },
    /// Compra automática inteligente (Raydium directo + fallback Jupiter)
    AutoBuy {
        /// Mint address del token
        #[arg(short, long)]
        mint: String,

        /// Cantidad de SOL a invertir
        #[arg(short, long, default_value_t = 0.025)]
        sol: f64,

        /// Símbolo del token (opcional)
        #[arg(long)]
        symbol: Option<String>,

        /// Añadir automáticamente al monitoreo
        #[arg(long, default_value_t = true)]
        monitor: bool,
    },
    /// Escanea la red en tiempo real (Sensor de Pump.fun)
    Scan,
    /// Inicia el monitor dinámico de posiciones (por defecto)
    Monitor,
}

/// Configuración del motor
const HELIUS_RPC: &str = "https://mainnet.helius-rpc.com/?api-key=";

/// Entry point de la librería
pub async fn run() -> Result<()> {
    dotenv::dotenv().ok();
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Buy {
            mint,
            sol,
            slippage,
        }) => handle_buy_mode(mint, sol, slippage).await?,
        Some(Commands::AutoBuy {
            mint,
            sol,
            symbol,
            monitor,
        }) => handle_auto_buy_mode(mint, sol, symbol, monitor).await?,
        Some(Commands::Scan) => handle_scan_mode().await?,
        _ => run_monitor_mode().await?,
    }

    Ok(())
}

async fn handle_buy_mode(mint: String, sol: f64, slippage: u16) -> Result<()> {
    println!("🚀 INICIANDO MODO COMPRA DIRECTA...");
    let api_key = std::env::var("HELIUS_API_KEY").expect("HELIUS_API_KEY missing");
    let rpc_url = format!("{}{}", HELIUS_RPC, api_key);

    let config = ExecutorConfig {
        rpc_url: rpc_url.clone(),
        slippage_bps: slippage,
        priority_fee: 50_000,
        dry_run: false,
    };
    let executor = TradeExecutor::new(config);
    let keypair = load_keypair_from_env("WALLET_PRIVATE_KEY")?;
    executor.execute_buy(&mint, Some(&keypair), sol).await?;
    Ok(())
}

async fn handle_auto_buy_mode(
    mint: String,
    sol: f64,
    symbol: Option<String>,
    add_to_monitor: bool,
) -> Result<()> {
    use auto_buyer::{AutoBuyConfig, AutoBuyer};

    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║      🤖 AUTO-BUY INTELIGENTE - Raydium Directo           ║");
    println!("╚════════════════════════════════════════════════════════════╝\n");

    let api_key = std::env::var("HELIUS_API_KEY").expect("HELIUS_API_KEY missing");
    let rpc_url = format!("{}{}", HELIUS_RPC, api_key);
    let keypair = load_keypair_from_env("WALLET_PRIVATE_KEY")?;
    let buyer = AutoBuyer::new(rpc_url)?;

    let config = AutoBuyConfig {
        token_mint: mint.clone(),
        symbol,
        amount_sol: sol,
        slippage_bps: 300,
        add_to_monitoring: add_to_monitor,
        stop_loss_percent: -60.0,
        trailing_enabled: true,
        fast_mode: false,
    };

    match buyer.buy(&config, &keypair).await {
        Ok(result) => {
            println!(
                "\n✅ COMPRA EXITOSA | Token: {} | Tx: {}",
                result.token_mint, result.signature
            );
            if add_to_monitor {
                println!("✅ Token añadido al monitoreo. Reinicia el bot.");
            }
        }
        Err(e) => eprintln!("\n❌ Error en compra: {}", e),
    }
    Ok(())
}

async fn handle_scan_mode() -> Result<()> {
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║         📡 NETWORK SCANNER - Pump.fun Telemetry          ║");
    println!("╚════════════════════════════════════════════════════════════╝\n");
    use websocket::{SolanaWebSocket, WebSocketConfig};
    let config = WebSocketConfig::from_env();
    let scanner = SolanaWebSocket::new(config);
    scanner.listen_to_pump_events().await?;
    Ok(())
}

async fn run_monitor_mode() -> Result<()> {
    let obs_config = if std::env::var("RUST_LOG").is_ok() {
        observability::ObservabilityConfig::production()
    } else {
        observability::ObservabilityConfig::development()
    };
    let _ = observability::init_observability(obs_config);

    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║         🏎️  THE CHASSIS - Solana Trading Engine          ║");
    println!("║           v2.0.0-HFT - Institutional Grade                 ║");
    println!("╚════════════════════════════════════════════════════════════╝\n");

    let app_config = AppConfig::load()
        .unwrap_or_else(|_| panic!("❌ Error crítico: No se pudo cargar settings.json"));
    println!(
        "✅ Configuración HFT cargada. Auto-Execute: {}",
        app_config.global_settings.auto_execute
    );

    let api_key = std::env::var("HELIUS_API_KEY").expect("HELIUS_API_KEY must be set");
    let rpc_url = format!("{}{}", HELIUS_RPC, api_key);
    let wallet_addr = std::env::var("WALLET_ADDRESS").expect("WALLET_ADDRESS must be set");

    // 1. Wallet Monitor
    let wallet_monitor = Arc::new(WalletMonitor::new(rpc_url.clone(), &wallet_addr)?);
    let sol_balance = wallet_monitor.get_sol_balance()?;
    println!("🏦 Balance Inicial: {:.4} SOL", sol_balance);

    // 2. DB Asíncrona (Connection Pool)
    let state_manager = Arc::new(StateManager::new("trading_state.db").await?);

    // 3. Emergency System
    let emergency_monitor = Arc::new(Mutex::new(EmergencyMonitor::new(EmergencyConfig {
        max_loss_percent: -99.9,
        min_sol_balance: app_config.global_settings.min_sol_balance,
        min_asset_price: 0.0,
        enabled: true,
    })));

    // =========================================================================
    // GHOST POSITION PURGE: Verificar balance on-chain antes de trackear
    // =========================================================================
    if let Ok(db_positions) = state_manager.get_active_positions().await {
        let rpc_for_check = solana_client::rpc_client::RpcClient::new(rpc_url.clone());
        let wallet_pubkey = solana_sdk::pubkey::Pubkey::from_str(&wallet_addr)
            .expect("WALLET_ADDRESS inválida");

        for target in &db_positions {
            // Verificar si realmente tenemos tokens de esta posición
            let mint_pubkey = match solana_sdk::pubkey::Pubkey::from_str(&target.token_mint) {
                Ok(pk) => pk,
                Err(_) => {
                    println!("   ⚠️ Mint inválido en DB: {}, cerrando...", target.token_mint);
                    if let Err(e) = state_manager.close_position(&target.token_mint).await {
                        eprintln!("   ❌ DB ERROR cerrando mint inválido {}: {}", target.token_mint, e);
                    }
                    continue;
                }
            };

            let ata = spl_associated_token_account::get_associated_token_address(
                &wallet_pubkey,
                &mint_pubkey,
            );

            let has_balance = match rpc_for_check.get_account(&ata) {
                Ok(account_data) => {
                    match TokenAccount::unpack(&account_data.data) {
                        Ok(token_account) => token_account.amount > 0,
                        Err(_) => false,
                    }
                }
                Err(_) => false,
            };

            if !has_balance {
                println!(
                    "   🗑️ Ghost position detectada: {} ({}) — Sin balance on-chain. Cerrando en DB.",
                    target.symbol, &target.token_mint[..8]
                );
                if let Err(e) = state_manager.close_position(&target.token_mint).await {
                    eprintln!("   ❌ DB ERROR cerrando ghost position {}: {}", target.token_mint, e);
                }

                // Registrar como trade de limpieza
                let trade = state_manager::TradeRecord {
                    id: None,
                    signature: "GHOST_PURGE".to_string(),
                    token_mint: target.token_mint.clone(),
                    symbol: target.symbol.clone(),
                    trade_type: "GHOST_PURGE".to_string(),
                    amount_sol: 0.0,
                    tokens_amount: 0.0,
                    price: 0.0,
                    pnl_sol: Some(-target.amount_sol), // Pérdida total
                    pnl_percent: Some(-100.0),
                    route: "Boot Audit".to_string(),
                    price_impact_pct: 0.0,
                    fee_sol: 0.0,
                    timestamp: chrono::Utc::now().timestamp(),
                };
                if let Err(e) = state_manager.record_trade(trade).await {
                    eprintln!("   ❌ DB ERROR registrando ghost purge para {}: {}", target.token_mint, e);
                }
                continue;
            }

            // Posición válida: trackear
            emergency_monitor.lock().unwrap().add_position(Position {
                token_mint: target.token_mint.clone(),
                symbol: target.symbol.clone(),
                entry_price: target.entry_price,
                amount_invested: target.amount_sol,
                current_price: target.entry_price,
                current_value: target.amount_sol,
            });
            println!(
                "   ✅ Trackeando: {} (SL: {}%)",
                target.symbol, target.stop_loss_percent
            );
        }
    }

    // 4. Executor & Keypair
    let executor_config =
        ExecutorConfig::new(rpc_url.clone(), !app_config.global_settings.auto_execute);
    let executor = Arc::new(TradeExecutor::new(executor_config));
    let mut wallet_keypair: Option<Keypair> = None;

    if app_config.global_settings.auto_execute {
        if let Ok(kp) = load_keypair_from_env("WALLET_PRIVATE_KEY") {
            wallet_keypair = Some(kp);
        }
    }

    // 5. PriceFeed (Telemetría de alta velocidad)
    let feed_config = PriceFeedConfig::from_env();
    let mut monitored_tokens: Vec<MonitoredToken> = Vec::new();
    
    // Trackear el precio de SOL por defecto siempre
    monitored_tokens.push(MonitoredToken {
        mint: "So11111111111111111111111111111111111111112".to_string(),
        symbol: "SOL".to_string(),
        pool_account: None,
        coin_vault: None,
        pc_vault: None,
        token_decimals: 9, // SOL usa 9 decimales
    });
    
    if let Ok(db_positions) = state_manager.get_active_positions().await {
        for pos in db_positions {
            monitored_tokens.push(MonitoredToken {
                mint: pos.token_mint.clone(),
                symbol: pos.symbol.clone(),
                pool_account: None,
                coin_vault: None,
                pc_vault: None,
                token_decimals: 6,
            });
        }
    }

    let (price_rx, price_cache, feed_tx) = PriceFeed::start(feed_config, monitored_tokens);
    let price_rx = price_rx;
    let _buyer = Arc::new(crate::auto_buyer::AutoBuyer::new_with_cache(
        rpc_url.clone(),
        Some(Arc::clone(&price_cache)),
    )?);

    // 6. Telemetry Server (WebSocket para la UI)
    let telemetry_server = Arc::new(crate::telemetry_server::TelemetryServer::new(
        Arc::clone(&state_manager),
        Arc::clone(&price_cache),
        Arc::clone(&wallet_monitor),
    ));

    tokio::spawn(async move {
        let _ = telemetry_server.run("127.0.0.1:9001").await;
    });

    // 7. Telegram y Comandos
    let telegram = Arc::new(TelegramNotifier::new());
    let command_handler = Arc::new(CommandHandler::new());

    let cmd_handler_clone = Arc::clone(&command_handler);
    let cmd_wallet_monitor = Arc::clone(&wallet_monitor);
    let cmd_config = Arc::new(app_config.clone());
    let cmd_executor = Arc::clone(&executor);
    let cmd_state_manager = Arc::clone(&state_manager);

    tokio::spawn(async move {
        let _ = cmd_handler_clone
            .process_commands(
                cmd_wallet_monitor,
                cmd_executor,
                cmd_config,
                cmd_state_manager,
                feed_tx,
            )
            .await;
    });

    // 7. Hibernación

    let hibernate_wallet = Arc::clone(&wallet_monitor);
    let hibernate_telegram = Arc::clone(&telegram);
    let hibernate_min_balance = app_config.global_settings.min_sol_balance;

    tokio::spawn(async move {
        loop {
            if let Ok(current_balance) = hibernate_wallet.get_sol_balance() {
                let is_hibernating = telegram::commands::CommandHandler::is_hibernating();
                if current_balance < hibernate_min_balance && !is_hibernating {
                    telegram::commands::HIBERNATION_MODE
                        .store(true, std::sync::atomic::Ordering::Relaxed);
                    let _ = hibernate_telegram
                        .send_message(
                            &format!(
                                "🛑 <b>HIBERNACIÓN AUTOMÁTICA</b>\nBalance: {} SOL",
                                current_balance
                            ),
                            true,
                        )
                        .await;
                } else if current_balance >= (hibernate_min_balance + 0.04) && is_hibernating {
                    telegram::commands::HIBERNATION_MODE
                        .store(false, std::sync::atomic::Ordering::Relaxed);
                    let _ = hibernate_telegram
                        .send_message(
                            &format!(
                                "🟢 <b>SISTEMA REACTIVADO</b>\nBalance: {} SOL",
                                current_balance
                            ),
                            true,
                        )
                        .await;
                }
            }
            tokio::time::sleep(std::time::Duration::from_secs(30)).await;
        }
    });


    // ============================================================================
    // LOOP PRINCIPAL HFT (Arquitectura Desacoplada)
    // ============================================================================
    println!("🏎️  Chassis ensamblado. Arrancando subsistemas asíncronos...\n");

    use crate::engine::commands::{ExecutionCommand, ExecutionFeedback};
    let (cmd_tx, cmd_rx) = tokio::sync::mpsc::channel::<ExecutionCommand>(1024);
    let (feedback_tx, feedback_rx) = tokio::sync::mpsc::channel::<ExecutionFeedback>(1024);

    let engine = crate::engine::strategy::StrategyEngine::new(Arc::clone(&state_manager));
    tokio::spawn(async move {
        engine.run_loop(price_rx, cmd_tx, feedback_rx).await;
    });

    let router = crate::engine::router::ExecutionRouter::new(Arc::clone(&executor), Arc::clone(&state_manager), Arc::clone(&telegram), wallet_keypair, feedback_tx);
    let router_handle = tokio::spawn(async move {
        Arc::new(router).run_dashboard(cmd_rx).await;
    });

    println!("✅ The Chassis está en marcha. Pulsa Ctrl+C para detener.\n");
    
    // Mantener el proceso vivo hasta que se reciba una señal de interrupción
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            println!("\n🛑 Señal de parada recibida. Cerrando The Chassis...");
        }
        _ = router_handle => {
            println!("\n⚠️ El Execution Router se ha detenido inesperadamente.");
        }
    }

    Ok(())
}
