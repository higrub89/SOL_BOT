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
pub mod auto_buyer;
pub mod state_manager;
pub mod validation;
pub mod price_feed;
pub mod amm_math;
pub mod ws_feed;
pub mod jito;

// 🏎️ Módulos del Framework Institucional (v2.0)
pub mod engine;
pub mod sensors;
pub mod executor_trait;
pub mod observability;
pub mod generated;

// ----------------------------------------------------------------------------
// IMPORTS INTERNOS
// ----------------------------------------------------------------------------

use config::AppConfig;
use wallet::{WalletMonitor, load_keypair_from_env};
use emergency::{EmergencyMonitor, EmergencyConfig, Position};
use executor_v2::{TradeExecutor, ExecutorConfig};
use telegram::TelegramNotifier;
use telegram_commands::CommandHandler;
use trailing_sl::TrailingStopLoss;
use liquidity_monitor::{LiquidityMonitor, LiquiditySnapshot};
use state_manager::{StateManager, TradeRecord};
use price_feed::{PriceFeed, PriceFeedConfig, MonitoredToken};

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
    // Cargar .env si no se ha cargado
    dotenv::dotenv().ok();
    
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Buy { mint, sol, slippage }) => {
            handle_buy_mode(mint, sol, slippage).await?;
        }
        Some(Commands::AutoBuy { mint, sol, symbol, monitor }) => {
            handle_auto_buy_mode(mint, sol, symbol, monitor).await?;
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
    
    let keypair = load_keypair_from_env("WALLET_PRIVATE_KEY")?;
    
    executor.execute_buy(&mint, Some(&keypair), sol).await?;
    
    Ok(())
}

async fn handle_auto_buy_mode(mint: String, sol: f64, symbol: Option<String>, add_to_monitor: bool) -> Result<()> {
    use auto_buyer::{AutoBuyer, AutoBuyConfig};
    
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║      🤖 AUTO-BUY INTELIGENTE - Raydium Directo           ║");
    println!("╚════════════════════════════════════════════════════════════╝\n");
    
    // Configurar
    let api_key = std::env::var("HELIUS_API_KEY").expect("HELIUS_API_KEY missing");
    let rpc_url = format!("{}{}", HELIUS_RPC, api_key);
    
    let keypair = load_keypair_from_env("WALLET_PRIVATE_KEY")?;
    
    // Crear AutoBuyer
    let buyer = AutoBuyer::new(rpc_url)?;
    
    // Configurar compra
    let config = AutoBuyConfig {
        token_mint: mint.clone(),
        symbol,
        amount_sol: sol,
        slippage_bps: 300, // 3% default
        add_to_monitoring: add_to_monitor,
        stop_loss_percent: -60.0,
        trailing_enabled: true,
        fast_mode: false, // Compra manual: análisis completo (más preciso)
    };
    
    // Ejecutar
    match buyer.buy(&config, &keypair).await {
        Ok(result) => {
            println!("\n╔════════════════════════════════════════════════════════════╗");
            println!("║                  ✅ COMPRA EXITOSA ✅                      ║");
            println!("╚════════════════════════════════════════════════════════════╝\n");
            println!("📊 Resumen:");
            println!("   • Token:          {}", result.token_mint);
            println!("   • Inversión:      {:.4} SOL", result.amount_sol);
            println!("   • Tokens:         {:.2}", result.tokens_received);
            println!("   • Precio:         ${:.8}", result.effective_price);
            println!("   • Ruta:           {}", result.route);
            println!("   • Signature:      {}", result.signature);
            println!("\n🔗 Ver TX: https://solscan.io/tx/{}", result.signature);
            
            if add_to_monitor {
                println!("\n✅ Token añadido al monitoreo automático");
                println!("⚠️  Reinicia el bot para activar el monitoreo:");
                println!("   docker-compose restart");
            }
        },
        Err(e) => {
            eprintln!("\n❌ Error en compra automática: {}", e);
            eprintln!("\n💡 Sugerencias:");
            eprintln!("   1. Verifica que el pool existe en Raydium");
            eprintln!("   2. Usa DexScreener para confirmar el mint: {}", mint);
            eprintln!("   3. Verifica tu balance SOL");
            return Err(e);
        }
    }
    
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
    println!("📂 Cargando configuración global dinámica desde settings.json...");
    let app_config = match AppConfig::load() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("❌ Error cargando settings.json: {}", e);
            eprintln!("   Asegúrate de que el archivo existe en el directorio actual.");
            return Ok(());
        }
    };
    
    println!("✅ Configuración global cargada:");
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
    let state_manager = Arc::new(StateManager::new("trading_state.db").await?);
    
    if let Ok(db_positions) = state_manager.get_active_positions().await {
        for target in &db_positions {
            emergency_monitor.lock().unwrap().add_position(Position {
                token_mint: target.token_mint.clone(),
                symbol: target.symbol.clone(),
                entry_price: target.entry_price,
                amount_invested: target.amount_sol,
                current_price: target.entry_price,
                current_value: target.amount_sol,
            });
            println!("   • Cargado: {} (SL: {}%)", target.symbol, target.stop_loss_percent);
        }
    }

    println!("\n───────────────────────────────────────────────────────────\n");

    // Eliminado el código legacy de migración desde targets.json
    
    // Show stats
    let stats = state_manager.get_stats().await?;
    println!("   • Active Positions: {}", stats.active_positions);
    println!("   • Total Trades:     {}", stats.total_trades);
    println!("   • Total PnL:        {:.4} SOL", stats.total_pnl_sol);
    println!("✅ STATE MANAGER inicializado correctamente");

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
        match load_keypair_from_env("WALLET_PRIVATE_KEY") {
            Ok(kp) => {
                println!("   • Keypair cargado correctamente para {}", kp.pubkey());
                wallet_keypair = Some(kp);
            }
            Err(e) => {
                eprintln!("   • ❌ Error cargando WALLET_PRIVATE_KEY: {}", e);
            }
        }
    }
    
    if app_config.global_settings.auto_execute && wallet_keypair.is_none() {
        println!("\n⚠️  ATENCIÓN: Auto-Execute está activado pero el Keypair no pudo ser cargado. El sistema operará en modo DRY-RUN o ALERTA como medida de seguridad.\n");
    }

    // 5. PriceFeed — Hub unificado de precios
    println!("📡 PRICE FEED SETUP:");
    let feed_config = PriceFeedConfig::from_env();
    
    // ── UNIFIED MONITORING LIST (Targets + DB) ──
    let mut monitored_tokens: Vec<MonitoredToken> = Vec::new();

    // 1. Add from StateManager (Dynamic/DB)
    if let Ok(db_positions) = state_manager.get_active_positions().await {
        for pos in db_positions {
            monitored_tokens.push(MonitoredToken {
                mint: pos.token_mint.clone(),
                symbol: pos.symbol.clone(),
                pool_account: None, // Will fallback to DexScreener/WebSocket
                coin_vault: None,
                pc_vault: None,
                token_decimals: 6, // Default
            });
        }
    }

    println!("   • Total Monitored: {}", monitored_tokens.len());
    
    if feed_config.geyser_enabled {
        println!("   ⚡ Modo:       HFT (Geyser gRPC + DexScreener fallback)");
    } else {
        println!("   📡 Modo:       Standard (DexScreener HTTP)");
    }
    
    let (price_rx, price_cache, feed_tx) = PriceFeed::start(feed_config, monitored_tokens);
    let mut price_rx = price_rx; // Re-bind to allow loop consumption
    println!("✅ PriceFeed inicializado");

    // 🧠 Inicializar AutoBuyer para seguimiento de Momentum Real
    let buyer = Arc::new(crate::auto_buyer::AutoBuyer::new_with_cache(
        rpc_url.clone(),
        Some(Arc::clone(&price_cache))
    )?);

    println!("📣 Inicializando Telegram Notifier & Command Handler...");
    // 3.5 Telegram Notifier & Command Handler Setup
    let telegram = Arc::new(TelegramNotifier::new());
    let command_handler = Arc::new(CommandHandler::new());
    println!("✅ Telegram components creados");
    
    // Lanzar el receptor de comandos en segundo plano
    let cmd_handler_clone = Arc::clone(&command_handler);
    let cmd_emergency_monitor = Arc::clone(&emergency_monitor);
    let cmd_wallet_monitor = Arc::clone(&wallet_monitor);
    let cmd_config = Arc::new(app_config.clone());
    let cmd_executor = Arc::clone(&executor);
    let cmd_state_manager = Arc::clone(&state_manager);
    let cmd_feed_tx = feed_tx.clone();
    
    tokio::spawn(async move {
        println!("📱 Telegram Command Handler: ACTIVADO");
        let _ = cmd_handler_clone.process_commands(
            cmd_emergency_monitor,
            cmd_wallet_monitor,
            cmd_executor,
            cmd_config,
            cmd_state_manager,
            cmd_feed_tx
        ).await;
    });

    println!("\n───────────────────────────────────────────────────────────\n");

    let monitor_clone = Arc::clone(&emergency_monitor);
    let telegram_clone = Arc::clone(&telegram);
    let executor_clone = Arc::clone(&executor);

    // Setup de Trailing SL y Liquidez para cada target
    let mut trailing_monitors: std::collections::HashMap<String, TrailingStopLoss> = std::collections::HashMap::new();
    let mut liquidity_monitors: std::collections::HashMap<String, LiquidityMonitor> = std::collections::HashMap::new();

    if let Ok(db_positions) = state_manager.get_active_positions().await {
        for target in db_positions {
            if target.trailing_enabled {
                let mut tsl = TrailingStopLoss::new(
                    target.entry_price,
                    target.stop_loss_percent,
                    target.trailing_distance_percent,
                    target.trailing_activation_threshold,
                );

                // 💧 Hydrate TSL from StateManager (DB)
                match state_manager.get_position(&target.token_mint).await {
                    Ok(Some(saved_pos)) => {
                        if let Some(peak) = saved_pos.trailing_highest_price {
                            if peak > tsl.peak_price {
                                tsl.peak_price = peak;
                            }
                        }
                        
                        if let Some(curr_sl) = saved_pos.trailing_current_sl {
                            if curr_sl > tsl.current_sl_percent {
                                tsl.current_sl_percent = curr_sl;
                                tsl.enabled = true;
                                println!("   💧 TSL Restored for {}: SL={:.2}% | Peak=${:.4}", 
                                    target.symbol, curr_sl, tsl.peak_price);
                            }
                        }
                    },
                    Ok(None) => {},
                    Err(e) => eprintln!("⚠️ Failed to hydrate TSL for {}: {}", target.symbol, e),
                }

                trailing_monitors.insert(
                    target.symbol.clone(),
                    tsl
                );
            }
            liquidity_monitors.insert(target.symbol.clone(), LiquidityMonitor::new(20.0, 5.0));
        }
    }

    // ── Hibernation checker (cada 30 segundos en background) ──
    let hibernate_wallet = Arc::clone(&wallet_monitor);
    let hibernate_telegram = Arc::clone(&telegram);
    let hibernate_min_balance = app_config.global_settings.min_sol_balance;
    
    tokio::spawn(async move {
        loop {
            if let Ok(current_balance) = hibernate_wallet.get_sol_balance() {
                if current_balance < hibernate_min_balance 
                   && !telegram_commands::CommandHandler::is_hibernating() 
                {
                    telegram_commands::HIBERNATION_MODE.store(true, std::sync::atomic::Ordering::Relaxed);
                    eprintln!("\n🛑 HIBERNACIÓN AUTOMÁTICA: Balance ({:.4} SOL) < Mínimo ({:.4} SOL)",
                        current_balance, hibernate_min_balance);
                    let _ = hibernate_telegram.send_message(
                        &format!("<b>🛑 HIBERNACIÓN AUTOMÁTICA</b>\n<b>━━━━━━━━━━━━━━━━━━━━━━</b>\n<b>⬡ Balance:</b> <code>{:.4} SOL</code>\n<b>⬡ Mínimo:</b> <code>{:.4} SOL</code>\n\nEl bot ha detenido toda ejecución para proteger tus fondos.\nUsa /wake después de fondear la wallet.",
                            current_balance, hibernate_min_balance),
                        true
                    ).await;
                }
            }
            tokio::time::sleep(std::time::Duration::from_secs(30)).await;
        }
    });
    
    // ══════════════════════════════════════════════════════════
    //  LOOP PRINCIPAL — Consume del PriceFeed en tiempo real
    // ══════════════════════════════════════════════════════════
    println!("🏎️  Loop principal activo. Esperando datos de precio...\n");

    // HashSet para evitar intentar vender el mismo token múltiples veces en el mismo ciclo de vida
    // Previene el bucle infinito de alertas cuando la venta falla
    let mut sell_attempted: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut tp1_attempted: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut tp2_attempted: std::collections::HashSet<String> = std::collections::HashSet::new();
    // HashSet para evitar spamear alertas de hibernación + SL cada tick
    let mut sl_alerted: std::collections::HashSet<String> = std::collections::HashSet::new();

    // HashMap para limitar el ruido en consola: (Token -> Última vez que se imprimió log)
    let mut last_log_time: std::collections::HashMap<String, i64> = std::collections::HashMap::new();

    while let Some(price_update) = price_rx.recv().await {
        // Fetch from DB (live hot-swap of parameters and active state)
        let target = match state_manager.get_position(&price_update.token_mint).await {
            Ok(Some(p)) if p.active => p,
            _ => continue,
        };

        // 📈 Registrar tick de precio para cálculo de Momentum Real
        buyer.record_price_tick(&price_update.token_mint, price_update.price_usd).await;
        
        let tokens_held = target.amount_sol / target.entry_price;
        let current_value = tokens_held * price_update.price_native;
        
        // Actualizar EmergencyMonitor con precios NATIVOS (SOL) para consistencia en PnL
        {
            let mut monitor = monitor_clone.lock().unwrap();
            
            // 🆕 INYECCIÓN DINÁMICA: Si es un token nuevo comprado por Telegram, añadirlo!
            if monitor.get_position(&target.token_mint).is_none() {
                println!("➕ [Monitor] Inyección dinámica de nueva posición: {} ({})", target.symbol, target.token_mint);
                monitor.add_position(crate::emergency::Position {
                    token_mint: target.token_mint.clone(),
                    symbol: target.symbol.clone(),
                    entry_price: target.entry_price,
                    amount_invested: target.amount_sol,
                    current_price: price_update.price_native,
                    current_value: current_value,
                });
            }
            
            // 🚨 BUGFIX: Pasando target.token_mint en lugar de symbol a emergency.rs
            monitor.update_position(&target.token_mint, price_update.price_native, current_value);
        }

        // 🆕 Dinámicamente inicializar Trailing SL & Liquidez si no existen (compras Telegram)
        if target.trailing_enabled && !trailing_monitors.contains_key(&target.symbol) {
             let mut tsl = TrailingStopLoss::new(
                 target.entry_price,
                 target.stop_loss_percent,
                 target.trailing_distance_percent,
                 target.trailing_activation_threshold,
             );
             if let Some(peak) = target.trailing_highest_price { tsl.peak_price = tsl.peak_price.max(peak); }
             if let Some(curr_sl) = target.trailing_current_sl { tsl.current_sl_percent = curr_sl; tsl.enabled = true; }
             trailing_monitors.insert(target.symbol.clone(), tsl);
        }
        if !liquidity_monitors.contains_key(&target.symbol) {
             liquidity_monitors.insert(target.symbol.clone(), LiquidityMonitor::new(20.0, 5.0));
        }

        // Actualizar estado persistente con precio NATIVO (SOL)
        if let Err(e) = state_manager.update_position_price(&target.token_mint, price_update.price_native).await {
            eprintln!("⚠️ Error updating persistent state for {}: {}", target.symbol, e);
        }
        
        // Obtener datos de la posición para evaluar
        let position_data = {
            let monitor = monitor_clone.lock().unwrap();
            // 🚨 BUGFIX: Pasando target.token_mint en lugar de symbol
            monitor.get_position(&target.token_mint).cloned()
        };
        
        if let Some(pos) = position_data {
            let dd = pos.drawdown_percent();
            
            // Determinar el Stop-Loss Efectivo (el máximo entre el SL inicial y el TSL si está activo)
            let effective_sl_percent = if let Some(tsl) = trailing_monitors.get(&target.symbol) {
                if tsl.current_sl_percent > target.stop_loss_percent {
                    tsl.current_sl_percent
                } else {
                    target.stop_loss_percent
                }
            } else {
                target.stop_loss_percent
            };
            
            let dist_to_sl = dd - effective_sl_percent;
            let status_emoji = if dist_to_sl > 10.0 { "🟢" } else if dist_to_sl > 5.0 { "🟡" } else { "🔴" };
            let source_tag = format!("[{}]", price_update.source);
            
            // Trailing SL status
            let tsl_info = if let Some(tsl) = trailing_monitors.get(&target.symbol) {
                if let Err(e) = state_manager.update_trailing_sl(&target.token_mint, tsl.peak_price, tsl.current_sl_percent).await {
                    eprintln!("⚠️ Error updating trailing SL persistence: {}", e);
                }
                format!(" | TSL: {}", tsl.status_string())
            } else {
                String::new()
            };

            // 💰 TAKE PROFIT LOGIC 💰
            // Read TP settings from target (which is fetched from DB in this tick)
            let tp_target_percent = target.tp_percent.unwrap_or(100.0);
            let tp_amount_percent = target.tp_amount_percent.unwrap_or(50.0);
            let tp_triggered = target.tp_triggered;
            
            let tp2_target_percent = target.tp2_percent.unwrap_or(200.0);
            let tp2_amount_percent = target.tp2_amount_percent.unwrap_or(100.0); // 100% of the REMAINING balance
            let tp2_triggered = target.tp2_triggered;
            
            let db_amount_sol = target.amount_sol;

            let current_gain_percent = ((pos.current_price - pos.entry_price) / pos.entry_price) * 100.0;
            let tp_price = pos.entry_price * (1.0 + tp_target_percent / 100.0);
            let tp2_price = pos.entry_price * (1.0 + tp2_target_percent / 100.0);

            let tp_status = if tp_triggered && tp2_triggered {
                "✅ ALL TP HIT".to_string()
            } else if tp_triggered {
                format!("✅ TP1 HIT | TP2: {:.1}% ({:.6} SOL)", tp2_target_percent, tp2_price)
            } else {
                format!("TP1: {:.1}% ({:.6} SOL) | TP2: {:.1}%", tp_target_percent, tp_price, tp2_target_percent)
            };

            // Limit LOG spam: solo imprimir la tarjeta una vez cada 15 segundos por token
            let now = chrono::Utc::now().timestamp();
            let last_printed = *last_log_time.get(&target.token_mint).unwrap_or(&0);
            
            if now - last_printed >= 15 {
                println!("┌────────────────────────────────────────────────────────┐");
                println!("│ {} {} Status {:>30} │", status_emoji, target.symbol, source_tag);
                println!("├────────────────────────────────────────────────────────┤");
                println!("│   Price:    {:.8} SOL                  │", pos.current_price);
                println!("│   PnL:      {:.2}%                                  │", current_gain_percent);
                println!("│   SL Limit: {:.1}% (Dist: {:.2}%)                    │", effective_sl_percent, dist_to_sl);
                if !tsl_info.is_empty() {
                    println!("│   {:<53}│", tsl_info.trim());
                }
                println!("│   {:<53}│", tp_status);
                println!("└────────────────────────────────────────────────────────┘");
                
                last_log_time.insert(target.token_mint.clone(), now);
            }

            // ── Lógica de Take Profit (Múltiples Niveles) ──
            
            // TP1 (Primer Escalon)
            if !tp_triggered && current_gain_percent >= tp_target_percent {
                 println!("\n╔════════════════════════════════════════════════════════════╗");
                 println!("║                 💰 TAKE PROFIT 1 TRIGGERED! 💰            ║");
                 println!("║         Gain: {:.2}% >= Target {:.1}%                     ║", current_gain_percent, tp_target_percent);
                 println!("╚════════════════════════════════════════════════════════════╝\n");

                 if telegram_commands::CommandHandler::is_hibernating() {
                     // Log warning - bot suspended
                 } else if app_config.global_settings.auto_execute && !tp1_attempted.contains(&target.token_mint) {
                     tp1_attempted.insert(target.token_mint.clone());
                     println!("⚡ AUTO-EXECUTING TAKE PROFIT 1 ({}%)...", tp_amount_percent);
                     let sell_amount_pct = tp_amount_percent as u8;

                     let mint_clone = target.token_mint.clone();
                     let kp_wrapper = wallet_keypair.as_ref().map(|k| k.insecure_clone());
                     let exc = Arc::clone(&executor_clone);
                     let state_mgr = Arc::clone(&state_manager);
                     let tel = Arc::clone(&telegram_clone);
                     let symbol = target.symbol.clone();
                     let price_usd = price_update.price_usd;
                     
                     tokio::spawn(async move {
                         let sell_result = exc.execute_emergency_sell(
                            &mint_clone,
                            kp_wrapper.as_ref(), // pass as reference inside task
                            sell_amount_pct,
                        ).await;

                        match sell_result {
                            Ok(swap_result) => {
                                 println!("✅ TP1 Parcial completado: {}", swap_result.signature);
                                 let _ = tel.send_message(
                                    &format!("💰 <b>TAKE PROFIT 1 HIT</b> for {}!\n\
                                              <b>⬡ Gain:</b> {:.2}%\n\
                                              <b>⬡ Sold:</b> {}%\n\
                                              <b>⬡ Tx:</b> {}",
                                              symbol, current_gain_percent, sell_amount_pct, swap_result.signature),
                                    true
                                ).await;

                                // Marcar TP como ejecutado en DB
                                let _ = state_mgr.mark_tp_triggered(&mint_clone).await;

                                // Actualizar amount restante en DB (aprox)
                                let remaining_sol = db_amount_sol * (1.0 - (tp_amount_percent / 100.0));
                                let _ = state_mgr.update_amount_invested(&mint_clone, remaining_sol).await;

                                // Registrar Trade en historial
                                let pnl_sold_portion = swap_result.output_amount - (db_amount_sol * (tp_amount_percent / 100.0));
                                let trade_record = TradeRecord {
                                    id: None,
                                    signature: swap_result.signature.clone(),
                                    token_mint: mint_clone,
                                    symbol,
                                    trade_type: "TAKE_PROFIT_1".to_string(),
                                    amount_sol: swap_result.output_amount,
                                    tokens_amount: 0.0,
                                    price: price_usd,
                                    pnl_sol: Some(pnl_sold_portion),
                                    pnl_percent: Some(current_gain_percent),
                                    route: "Jupiter".to_string(),
                                    price_impact_pct: 0.0,
                                    timestamp: Utc::now().timestamp(),
                                };
                                let _ = state_mgr.record_trade(trade_record).await;
                            },
                            Err(e) => eprintln!("❌ Error executing TP1: {}", e),
                        }
                     });
                 }
            }
            
            // TP2 (Moonbag Escalon)
            if tp_triggered && !tp2_triggered && current_gain_percent >= tp2_target_percent {
                 println!("\n╔════════════════════════════════════════════════════════════╗");
                 println!("║                 🚀 TAKE PROFIT 2 (MOONBAG) TRIGGERED!     ║");
                 println!("║         Gain: {:.2}% >= Target {:.1}%                     ║", current_gain_percent, tp2_target_percent);
                 println!("╚════════════════════════════════════════════════════════════╝\n");

                 if telegram_commands::CommandHandler::is_hibernating() {
                     // Log warning - bot suspended
                 } else if app_config.global_settings.auto_execute && !tp2_attempted.contains(&target.token_mint) {
                     tp2_attempted.insert(target.token_mint.clone());
                     println!("⚡ AUTO-EXECUTING TAKE PROFIT 2 ({}%)...", tp2_amount_percent);
                     let sell_amount_pct = tp2_amount_percent as u8;

                     let mint_clone = target.token_mint.clone();
                     let kp_wrapper = wallet_keypair.as_ref().map(|k| k.insecure_clone());
                     let exc = Arc::clone(&executor_clone);
                     let state_mgr = Arc::clone(&state_manager);
                     let tel = Arc::clone(&telegram_clone);
                     let symbol = target.symbol.clone();
                     let price_usd = price_update.price_usd;
                     
                     tokio::spawn(async move {
                         let sell_result = exc.execute_emergency_sell(
                            &mint_clone,
                            kp_wrapper.as_ref(),
                            sell_amount_pct,
                        ).await;

                        match sell_result {
                            Ok(swap_result) => {
                                 println!("✅ TP2 completado: {}", swap_result.signature);
                                 let _ = tel.send_message(
                                    &format!("🚀 <b>TAKE PROFIT 2 (MOONBAG) HIT</b> for {}!\n\
                                              <b>⬡ Gain:</b> {:.2}%\n\
                                              <b>⬡ Sold:</b> {}%\n\
                                              <b>⬡ Tx:</b> {}",
                                              symbol, current_gain_percent, sell_amount_pct, swap_result.signature),
                                    true
                                ).await;

                                // Marcar TP2 como ejecutado en DB
                                let _ = state_mgr.mark_tp2_triggered(&mint_clone).await;
                                
                                // Si se vendió todo lo restante, cerramos posición
                                if sell_amount_pct == 100 {
                                    let _ = state_mgr.close_position(&mint_clone).await;
                                } else {
                                    let remaining_sol = db_amount_sol * (1.0 - (tp2_amount_percent / 100.0));
                                    let _ = state_mgr.update_amount_invested(&mint_clone, remaining_sol).await;
                                }

                                // Registrar Trade en historial
                                let pnl_sold_portion = swap_result.output_amount - (db_amount_sol * (tp2_amount_percent / 100.0));
                                let trade_record = TradeRecord {
                                    id: None,
                                    signature: swap_result.signature.clone(),
                                    token_mint: mint_clone,
                                    symbol,
                                    trade_type: "TAKE_PROFIT_2".to_string(),
                                    amount_sol: swap_result.output_amount,
                                    tokens_amount: 0.0,
                                    price: price_usd,
                                    pnl_sol: Some(pnl_sold_portion),
                                    pnl_percent: Some(current_gain_percent),
                                    route: "Jupiter".to_string(),
                                    price_impact_pct: 0.0,
                                    timestamp: Utc::now().timestamp(),
                                };
                                let _ = state_mgr.record_trade(trade_record).await;
                            },
                            Err(e) => eprintln!("❌ Error executing TP2: {}", e),
                        }
                     });
                 }
            }

            // ── Lógica de Emergencia (Stop-Loss + Auto-Sell + TSL) ──
            if dd <= effective_sl_percent {
                println!("\n╔════════════════════════════════════════════════════════════╗");
                println!("║                  🚨 EMERGENCY ALERT! 🚨                   ║");
                println!("║         SL ACTIVADO: {} @ {:.2}% (Limit: {:.1}%)          ║", target.symbol, dd, effective_sl_percent);
                println!("╚════════════════════════════════════════════════════════════╝\n");

                if telegram_commands::CommandHandler::is_hibernating() {
                    // Solo alertar UNA vez por token para no spamear
                    if !sl_alerted.contains(&target.token_mint) {
                        sl_alerted.insert(target.token_mint.clone());
                        println!("🛑 Bot en HIBERNACIÓN — no se ejecuta auto-sell para {}.", target.symbol);
                        let _ = telegram_clone.send_message(
                            &format!("🛑 <b>SL alcanzado para {}</b> ({:.2}%), pero el bot está en hibernación.\nVende manualmente en: <a href='https://jup.ag/swap/{}-SOL'>Jupiter</a>", target.symbol, dd, target.token_mint),
                            true
                        ).await;
                    }
                } else if app_config.global_settings.auto_execute {
                    // Solo intentar la venta UNA vez para no spamear en caso de fallo
                    if !sell_attempted.contains(&target.token_mint) {
                        sell_attempted.insert(target.token_mint.clone());
                        println!("⚡ AUTO-EXECUTING EMERGENCY SELL para {}...", target.symbol);

                        let mint_clone = target.token_mint.clone();
                        let kp_wrapper = wallet_keypair.as_ref().map(|k| k.insecure_clone());
                        let exc = Arc::clone(&executor_clone);
                        let state_mgr = Arc::clone(&state_manager);
                        let tel = Arc::clone(&telegram_clone);
                        let symbol = target.symbol.clone();
                        let price_usd = price_update.price_usd;
                        let target_amount_sol = target.amount_sol;
                        let target_entry_price = target.entry_price;

                        tokio::spawn(async move {
                            let sell_result = exc.execute_emergency_sell(
                                &mint_clone,
                                kp_wrapper.as_ref(),
                                100,
                            ).await;

                            match sell_result {
                                Ok(swap_result) => {
                                    println!("✅ Venta automática completada: {}", swap_result.signature);
                                    let _ = tel.send_message(
                                        &format!("✅ Venta automática de {} completada.\nSignature: {}", symbol, swap_result.signature),
                                        true
                                    ).await;

                                    let trade_record = TradeRecord {
                                        id: None,
                                        signature: swap_result.signature.clone(),
                                        token_mint: mint_clone.clone(),
                                        symbol: symbol.clone(),
                                        trade_type: "EMERGENCY_SELL".to_string(),
                                        amount_sol: swap_result.output_amount,
                                        tokens_amount: target_amount_sol / target_entry_price,
                                        price: price_usd,
                                        pnl_sol: Some(swap_result.output_amount - target_amount_sol),
                                        pnl_percent: Some(((swap_result.output_amount - target_amount_sol) / target_amount_sol) * 100.0),
                                        route: "Jupiter".to_string(),
                                        price_impact_pct: 0.0,
                                        timestamp: Utc::now().timestamp(),
                                    };

                                    if let Err(e) = state_mgr.record_trade(trade_record).await {
                                        eprintln!("❌ Error recording trade to DB: {}", e);
                                    }
                                    if let Err(e) = state_mgr.close_position(&mint_clone).await {
                                        eprintln!("❌ Error closing position in DB: {}", e);
                                    }
                                }
                                Err(e) => {
                                    eprintln!("❌ Error en auto-sell para {}: {}", symbol, e);
                                    println!("⚠️  ACCIÓN MANUAL REQUERIDA: VENDER EN TROJAN O JUPITER");

                                    // Cerrar la posición en DB para no seguir intentando
                                    let _ = state_mgr.close_position(&mint_clone).await;

                                    let _ = tel.send_error_alert(
                                        &format!(
                                            "❌ <b>Error en auto-sell para {}:</b> {}\n\n\
                                            ⚠️ Posición marcada como CERRADA en DB.\n\
                                            Vende manualmente: <a href='https://jup.ag/swap/{}-SOL'>Jupiter</a>",
                                            symbol, e, mint_clone
                                        )
                                    ).await;
                                }
                            }
                        });
                    } else {
                        // Ya se intentó — solo loguear localmente, no spamear Telegram
                        println!("⚠️  [{}] SL en -{}% pero ya se intentó sell. Esperando cierre del loop.", target.symbol, dd.abs());
                    }
                } else {
                    // Solo alertar UNA vez (sin auto-execute)
                    if !sl_alerted.contains(&target.token_mint) {
                        sl_alerted.insert(target.token_mint.clone());
                        println!("⚠️  ACCIÓN MANUAL REQUERIDA (Auto-Execute desactivado)");
                        let url = format!("https://jup.ag/swap/{}-SOL", target.token_mint);
                        let _ = telegram_clone.send_stop_loss_alert(
                            &target.symbol,
                            pos.current_price,
                            pos.entry_price,
                            dd,
                            effective_sl_percent,
                            &url
                        ).await;
                    }
                }
            }
        }
        
        // Actualizar Trailing SL
        if let Some(tsl) = trailing_monitors.get_mut(&target.symbol) {
            let _ = tsl.update(price_update.price_usd);
        }
        
        // Actualizar Monitor de Liquidez
        if let Some(lm) = liquidity_monitors.get_mut(&target.symbol) {
            let snapshot = LiquiditySnapshot {
                timestamp: Utc::now().timestamp(),
                liquidity_usd: price_update.liquidity_usd,
                volume_24h: price_update.volume_24h,
                price_usd: price_update.price_usd,
                holders_count: None,
            };
            let alerts = lm.add_snapshot(snapshot);
            for alert in alerts {
                let msg = alert.to_telegram_message(&target.symbol);
                let _ = telegram_clone.send_message(&msg, true).await;
            }
        }
    }
    
    eprintln!("⚠️  PriceFeed cerrado. El monitor se ha detenido.");
    Ok(())
}
