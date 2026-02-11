//! # The Chassis - Solana Trading Engine
//! 
//! v0.8.0 - Dynamic Configuration (Zero Recompile)

use anyhow::Result;
use chrono::Utc;
use solana_client::rpc_client::RpcClient;
use solana_sdk::signature::{Keypair, Signer};
use std::time::Instant;
use std::sync::{Arc, Mutex};
use clap::{Parser, Subcommand};

// ----------------------------------------------------------------------------
// EXPORTED MODULES (API PÚBLICA)
// ----------------------------------------------------------------------------

pub mod config;
pub mod latency;
pub mod geyser;
pub mod wallet;
pub mod emergency;
pub mod websocket;
pub mod scanner;
pub mod jupiter;
pub mod executor_simple;
pub mod executor_v2;
pub mod telegram;
pub mod telegram_commands;
pub mod trailing_sl;
pub mod liquidity_monitor;
pub mod raydium;
pub mod state_manager;
pub mod validation;

// 🏎️ Módulos del Framework Institucional (v2.0)
pub mod executor_trait;
pub mod observability;

// ----------------------------------------------------------------------------
// IMPORTS INTERNOS
// ----------------------------------------------------------------------------

use config::AppConfig;
use wallet::WalletMonitor;
use emergency::{EmergencyMonitor, EmergencyConfig, Position};
use scanner::PriceScanner;
use executor_v2::{TradeExecutor, ExecutorConfig};
use telegram::TelegramNotifier;
use telegram_commands::CommandHandler;
use trailing_sl::TrailingStopLoss;
use liquidity_monitor::{LiquidityMonitor, LiquiditySnapshot};

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
    /// Ejecuta una compra inmediata de un token
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
    /// Escanea la red en tiempo real (Sensor de Pump.fun)
    Scan,
    /// Inicia el monitor dinámico de posiciones (por defecto)
    Monitor,
}

/// Configuración del motor
const HELIUS_RPC: &str = "https://mainnet.helius-rpc.com/?api-key=";

/// Entry point de la librería
pub async fn run() -> Result<()> {
    // Cargar .env si no se ha cargado
    dotenv::dotenv().ok();
    
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Buy { mint, sol, slippage }) => {
            handle_buy_mode(mint, sol, slippage).await?;
        }
        Some(Commands::Scan) => {
            handle_scan_mode().await?;
        }
        _ => {
            run_monitor_mode().await?;
        }
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
        dry_run: false, // Forzar ejecución real
    };
    
    let executor = TradeExecutor::new(config);
    
    let priv_key = std::env::var("WALLET_PRIVATE_KEY").expect("WALLET_PRIVATE_KEY missing");
    let keypair = Keypair::from_base58_string(&priv_key);
    
    executor.execute_buy(&mint, Some(&keypair), sol).await?;
    
    Ok(())
}

async fn handle_scan_mode() -> Result<()> {
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║         📡 NETWORK SCANNER - Pump.fun Telemetry          ║");
    println!("║           Sensor de Alta Frecuencia Activado              ║");
    println!("╚════════════════════════════════════════════════════════════╝\n");
    
    use websocket::{WebSocketConfig, SolanaWebSocket};
    
    let config = WebSocketConfig::from_env();
    let scanner = SolanaWebSocket::new(config);
    
    println!("⚠️  AVISO: Este modo mostrará TODOS los eventos de Pump.fun en tiempo real.");
    println!("   Es posible que veas mucha actividad. Presiona Ctrl+C para salir.\n");
    
    scanner.listen_to_pump_events().await?;
    
    Ok(())
}

async fn run_monitor_mode() -> Result<()> {
    // Inicializar sistema de observabilidad (JSON en producción, texto en desarrollo)
    let obs_config = if std::env::var("RUST_LOG").is_ok() {
        observability::ObservabilityConfig::production()
    } else {
        observability::ObservabilityConfig::development()
    };
    let _ = observability::init_observability(obs_config);

    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║         🏎️  THE CHASSIS - Solana Trading Engine          ║");
    println!("║           v2.0.0-alpha - Institutional Grade               ║");
    println!("╚════════════════════════════════════════════════════════════╝\n");

    // Cargar configuración dinámica
    println!("📂 Cargando configuración dinámica desde targets.json...");
    let app_config = match AppConfig::load() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("❌ Error cargando targets.json: {}", e);
            eprintln!("   Asegúrate de que el archivo existe en el directorio actual.");
            return Ok(());
        }
    };
    
    println!("✅ Configuración cargada:");
    println!("   • Targets activos: {}", app_config.targets.len());
    println!("   • Auto-Execute:    {}", if app_config.global_settings.auto_execute { "ACTIVADO 🔴" } else { "DESACTIVADO 🟡 (Dry-Run)" });
    println!("   • Intervalo:       {}s", app_config.global_settings.monitor_interval_sec);

    let api_key = std::env::var("HELIUS_API_KEY")
        .expect("HELIUS_API_KEY must be set");
    let wallet_addr = std::env::var("WALLET_ADDRESS")
        .expect("WALLET_ADDRESS must be set");
    
    let rpc_url = format!("{}{}", HELIUS_RPC, api_key);
    
    // 1. Wallet Monitor
    println!("\n🏦 WALLET STATUS:");
    let wallet_monitor = Arc::new(WalletMonitor::new(rpc_url.clone(), &wallet_addr)?);
    let sol_balance = wallet_monitor.get_sol_balance()?;
    println!("   • Balance:   {:.4} SOL", sol_balance);
    
    println!("\n───────────────────────────────────────────────────────────\n");

    // 2. Emergency System Multi-Target Setup
    println!("🛡️  EMERGENCY SYSTEM (Multi-Target):");
    
    let emergency_monitor = Arc::new(Mutex::new(
        EmergencyMonitor::new(EmergencyConfig {
            max_loss_percent: -99.9,
            min_sol_balance: app_config.global_settings.min_sol_balance,
            min_asset_price: 0.0,
            enabled: true,
        })
    ));
    
    // Cargar targets activos
    for target in &app_config.targets {
        if !target.active { continue; }
        
        emergency_monitor.lock().unwrap().add_position(Position {
            token_mint: target.symbol.clone(),
            entry_price: target.entry_price,
            amount_invested: target.amount_sol,
            current_price: target.entry_price,
            current_value: target.amount_sol,
        });
        println!("   • Cargado: {} (SL: {}%)", target.symbol, target.stop_loss_percent);
    }

    println!("\n───────────────────────────────────────────────────────────\n");

    // 3. Executor & Telegram Setup
    println!("⚡ EXECUTOR STATUS: V2 (Auto-Sell Ready)");
    let executor_config = ExecutorConfig::new(
        rpc_url.clone(),
        !app_config.global_settings.auto_execute, // dry_run es el inverso de auto_execute
    );
    let executor = Arc::new(TradeExecutor::new(executor_config));

    // 3.1 Carga segura del Keypair si auto_execute está activado
    let mut wallet_keypair: Option<Keypair> = None;
    
    if app_config.global_settings.auto_execute {
        println!("🔑 Modo Auto-Execute: Cargando Keypair...");
        if let Ok(pk_bs58) = std::env::var("WALLET_PRIVATE_KEY") {
            // En esta versión del SDK, from_base58_string devuelve Keypair directamente
            let kp = Keypair::from_base58_string(&pk_bs58);
            println!("   • Keypair cargado correctamente para {}", kp.pubkey());
            wallet_keypair = Some(kp);
        } else {
            eprintln!("   • ❌ Error: WALLET_PRIVATE_KEY no encontrado en .env.");
        }
    }
    
    if app_config.global_settings.auto_execute && wallet_keypair.is_none() {
        println!("\n⚠️  ATENCIÓN: Auto-Execute está activado pero el Keypair no pudo ser cargado. El sistema operará en modo DRY-RUN o ALERTA como medida de seguridad.\n");
    }

    // 3.5 Telegram Notifier & Command Handler Setup
    let telegram = Arc::new(TelegramNotifier::new());
    let command_handler = Arc::new(CommandHandler::new());
    
    // Lanzar el receptor de comandos en segundo plano
    let cmd_handler_clone = Arc::clone(&command_handler);
    let cmd_emergency_monitor = Arc::clone(&emergency_monitor);
    let cmd_wallet_monitor = Arc::clone(&wallet_monitor);
    let cmd_config = Arc::new(app_config.clone());
    let cmd_executor = Arc::clone(&executor);
    
    tokio::spawn(async move {
        println!("📱 Telegram Command Handler: ACTIVADO");
        let _ = cmd_handler_clone.process_commands(
            cmd_emergency_monitor,
            cmd_wallet_monitor,
            cmd_executor,
            cmd_config
        ).await;
    });
    
    println!("\n───────────────────────────────────────────────────────────\n");

    // 4. Network Benchmark
    println!("📡 NETWORK STATUS:");
    let start = Instant::now();
    let rpc_client = RpcClient::new(rpc_url.clone());
    if let Ok(slot) = rpc_client.get_slot() {
        let latency = start.elapsed().as_millis();
        println!("   • Slot:     {}", slot);
        println!("   • Latency:  {}ms (HTTP)", latency);
    }

    println!("\n═══════════════════════════════════════════════════════════");
    println!("  🚀 INICIANDO MONITOR DINÁMICO v0.9.0");
    println!("═══════════════════════════════════════════════════════════\n");
    println!("⏰ Start Time: {}", Utc::now().format("%Y-%m-%d %H:%M:%S UTC"));
    println!("💡 Tip: Edita targets.json y reinicia para cambiar SL, Auto-Execute, etc.\n");
    
    println!("───────────────────────────────────────────────────────────\n");

    // 5. Price Scanner Dinámico
    let scanner = PriceScanner::new();
    let monitor_clone = Arc::clone(&emergency_monitor);
    let telegram_clone = Arc::clone(&telegram);
    let executor_clone = Arc::clone(&executor);
    let active_targets = app_config.targets.clone();

    // Setup de Trailing SL y Liquidez para cada target
    let mut trailing_monitors: std::collections::HashMap<String, TrailingStopLoss> = std::collections::HashMap::new();
    let mut liquidity_monitors: std::collections::HashMap<String, LiquidityMonitor> = std::collections::HashMap::new();

    for target in &active_targets {
        if target.active {
            if target.trailing_enabled {
                trailing_monitors.insert(
                    target.symbol.clone(),
                    TrailingStopLoss::new(
                        target.entry_price,
                        target.stop_loss_percent,
                        target.trailing_distance_percent,
                        target.trailing_activation_threshold,
                    )
                );
            }
            liquidity_monitors.insert(target.symbol.clone(), LiquidityMonitor::new(20.0, 5.0));
        }
    }
    
    // Loop principal de monitoreo
    loop {
        // === HIBERNATION CHECK: Balance bajo === 
        if let Ok(current_balance) = wallet_monitor.get_sol_balance() {
            if current_balance < app_config.global_settings.min_sol_balance 
               && !telegram_commands::CommandHandler::is_hibernating() 
            {
                telegram_commands::HIBERNATION_MODE.store(true, std::sync::atomic::Ordering::Relaxed);
                eprintln!("\n🛑 HIBERNACIÓN AUTOMÁTICA: Balance ({:.4} SOL) < Mínimo ({:.4} SOL)",
                    current_balance, app_config.global_settings.min_sol_balance);
                let _ = telegram_clone.send_message(
                    &format!("🛑 **HIBERNACIÓN AUTOMÁTICA**\n\nBalance: {:.4} SOL < Mínimo: {:.4} SOL\n\nEl bot ha detenido toda ejecución para proteger tus fondos.\nUsa `/wake` después de fondear la wallet.",
                        current_balance, app_config.global_settings.min_sol_balance),
                    true
                ).await;
            }
        }

        for target in &active_targets {
            if !target.active { continue; }
            
            // 1. Obtener precio
            match scanner.get_token_price(&target.mint).await {
                Ok(price) => {
                    let tokens_held = target.amount_sol / target.entry_price;
                    let current_value = tokens_held * price.price_usd;
                    
                    let mut monitor = monitor_clone.lock().unwrap();
                    monitor.update_position(&target.symbol, price.price_usd, current_value);
                    
                    if let Some(pos) = monitor.get_position(&target.symbol) {
                        let dd = pos.drawdown_percent();
                        let dist_to_sl = dd - target.stop_loss_percent;
                        let status_emoji = if dist_to_sl > 10.0 { "🟢" } else if dist_to_sl > 5.0 { "🟡" } else { "🔴" };
                        
                        // Trailing SL status
                        let tsl_info = if let Some(tsl) = trailing_monitors.get(&target.symbol) {
                            format!(" | TSL: {}", tsl.status_string())
                        } else {
                            String::new()
                        };

                        println!("┌────────────────────────────────────────────────────────┐");
                        println!("│ {} {} Status                                    │", status_emoji, target.symbol);
                        println!("├────────────────────────────────────────────────────────┤");
                        println!("│   Price:    ${:.8}                         │", pos.current_price);
                        println!("│   Drawdown: {:.2}%                                  │", dd);
                        println!("│   SL Limit: {:.1}% (Dist: {:.2}%)                    │", target.stop_loss_percent, dist_to_sl);
                        if !tsl_info.is_empty() {
                            println!("│   {:<53}│", tsl_info.trim());
                        }
                        println!("└────────────────────────────────────────────────────────┘");

                        // 5. Lógica de Emergencia Dinámica (con Auto-Sell)
                        if dd <= target.stop_loss_percent {
                            println!("\n╔════════════════════════════════════════════════════════════╗");
                            println!("║                  🚨 EMERGENCY ALERT! 🚨                   ║");
                            println!("║         SL ACTIVADO: {} @ {:.2}% (Limit: {:.1}%)          ║", target.symbol, dd, target.stop_loss_percent);
                            println!("╚════════════════════════════════════════════════════════════╝\n");
                            
                            // Verificar hibernación antes de ejecutar
                            if telegram_commands::CommandHandler::is_hibernating() {
                                println!("🛑 Bot en HIBERNACIÓN — no se ejecuta auto-sell.");
                            } else if app_config.global_settings.auto_execute {
                                println!("⚡ AUTO-EXECUTING EMERGENCY SELL...");
                                
                                let sell_result = executor_clone.execute_emergency_sell(
                                    &target.mint,
                                    wallet_keypair.as_ref(),
                                    100,
                                ).await;

                                match sell_result {
                                    Ok(swap_result) => {
                                        println!("✅ Venta automática completada: {}", swap_result.signature);
                                        let _ = telegram_clone.send_message(
                                            &format!("✅ Venta automática de {} completada.\nSignature: {}", target.symbol, swap_result.signature),
                                            true
                                        ).await;
                                    }
                                    Err(e) => {
                                        eprintln!("❌ Error en auto-sell: {}", e);
                                        println!("⚠️  ACCIÓN MANUAL REQUERIDA: VENDER EN TROJAN O JUPITER");
                                        let _ = telegram_clone.send_error_alert(
                                            &format!("❌ Error en auto-sell para {}: {}. SE REQUIERE ACCIÓN MANUAL.", target.symbol, e)
                                        ).await;
                                    }
                                }

                            } else {
                                println!("⚠️  ACCIÓN MANUAL REQUERIDA (Auto-Execute desactivado)");
                                let url = format!("https://jup.ag/swap/{}-SOL", target.mint);
                                let _ = telegram_clone.send_stop_loss_alert(
                                    &target.symbol,
                                    pos.current_price,
                                    pos.entry_price,
                                    dd,
                                    target.stop_loss_percent,
                                    &url
                                ).await;
                            }
                        }
                    }
                    if let Some(tsl) = trailing_monitors.get_mut(&target.symbol) {
                        if tsl.update(price.price_usd) {}
                    }
                    if let Some(lm) = liquidity_monitors.get_mut(&target.symbol) {
                        let snapshot = LiquiditySnapshot {
                            timestamp: Utc::now().timestamp(),
                            liquidity_usd: price.liquidity_usd,
                            volume_24h: price.volume_24h,
                            price_usd: price.price_usd,
                            holders_count: None,
                        };
                        let alerts = lm.add_snapshot(snapshot);
                        for alert in alerts {
                            let msg = alert.to_telegram_message(&target.symbol);
                            let _ = telegram_clone.send_message(&msg, true).await;
                        }
                    }
                    let current_sl = if let Some(tsl) = trailing_monitors.get(&target.symbol) {
                        tsl.current_sl_percent
                    } else {
                        target.stop_loss_percent
                    };

                    let drawdown = ((price.price_usd - target.entry_price) / target.entry_price) * 100.0;
                    
                    if drawdown <= current_sl {
                        let url = format!("https://jup.ag/swap/{}-SOL", target.mint);
                        let _ = telegram_clone.send_stop_loss_alert(
                            &target.symbol,
                            price.price_usd,
                            target.entry_price,
                            drawdown,
                            current_sl,
                            &url
                        ).await;
                    }
                }
                Err(e) => {
                    eprintln!("⚠️  Error obteniendo precio de {}: {}", target.symbol, e);
                }
            }
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        }
        tokio::time::sleep(std::time::Duration::from_secs(app_config.global_settings.monitor_interval_sec)).await;
        println!("");
    }
}
