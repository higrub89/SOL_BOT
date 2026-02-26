//! # The Chassis - Solana Trading Engine
//!
//! v2.0.0-HFT - Asynchronous Execution & Dynamic Configuration

use anyhow::Result;
use chrono::Utc;
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
pub mod latency;
pub mod liquidity_monitor;
pub mod price_feed;
pub mod raydium;
pub mod scanner;
pub mod state_manager;
pub mod telegram; // El módulo telegram ahora incluye commands internamente
pub mod trailing_sl;
pub mod validation;
pub mod wallet;
pub mod websocket;
pub mod ws_feed;

// 🏎️ Módulos del Framework Institucional (v2.0)
pub mod engine;
pub mod executor_trait;
pub mod generated;
pub mod observability;
pub mod sensors;

// ----------------------------------------------------------------------------
// IMPORTS INTERNOS
// ----------------------------------------------------------------------------

use config::AppConfig;
use emergency::{EmergencyConfig, EmergencyMonitor, Position};
use executor_v2::{ExecutorConfig, TradeExecutor};
use liquidity_monitor::LiquidityMonitor;
use price_feed::{MonitoredToken, PriceFeed, PriceFeedConfig};
use state_manager::StateManager;
use telegram::TelegramNotifier;
use telegram::commands::CommandHandler;
use trailing_sl::TrailingStopLoss;
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
                    let _ = state_manager.close_position(&target.token_mint).await;
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
                let _ = state_manager.close_position(&target.token_mint).await;

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
                let _ = state_manager.record_trade(trade).await;
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
    let mut price_rx = price_rx;
    let buyer = Arc::new(crate::auto_buyer::AutoBuyer::new_with_cache(
        rpc_url.clone(),
        Some(Arc::clone(&price_cache)),
    )?);

    // 6. Telegram y Comandos
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

    // 7. Trailing SL & Hibernación
    let mut trailing_monitors: std::collections::HashMap<String, TrailingStopLoss> =
        std::collections::HashMap::new();
    let mut liquidity_monitors: std::collections::HashMap<String, LiquidityMonitor> =
        std::collections::HashMap::new();

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
    // LOOP PRINCIPAL HFT (Non-Blocking I/O)
    // ============================================================================
    println!("🏎️  Motor asíncrono activado. Esperando ticks de telemetría...\n");

    let mut sell_attempted: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut tp1_attempted: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut tp2_attempted: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut sl_alerted: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut last_log_time: std::collections::HashMap<String, i64> =
        std::collections::HashMap::new();

    let monitor_clone = Arc::clone(&emergency_monitor);
    let telegram_clone = Arc::clone(&telegram);
    let executor_clone = Arc::clone(&executor);

    while let Some(price_update) = price_rx.recv().await {
        // Fetch en caliente desde DB pool
        let target = match state_manager.get_position(&price_update.token_mint).await {
            Ok(Some(p)) if p.active => p,
            _ => continue,
        };

        buyer
            .record_price_tick(&price_update.token_mint, price_update.price_usd)
            .await;
        let current_value = (target.amount_sol / target.entry_price) * price_update.price_native;

        {
            let mut monitor = monitor_clone.lock().unwrap();
            if monitor.get_position(&target.token_mint).is_none() {
                monitor.add_position(crate::emergency::Position {
                    token_mint: target.token_mint.clone(),
                    symbol: target.symbol.clone(),
                    entry_price: target.entry_price,
                    amount_invested: target.amount_sol,
                    current_price: price_update.price_native,
                    current_value,
                });
            }
            monitor.update_position(&target.token_mint, price_update.price_native, current_value);
        }

        if target.trailing_enabled && !trailing_monitors.contains_key(&target.symbol) {
            let mut tsl = TrailingStopLoss::new(
                target.entry_price,
                target.stop_loss_percent,
                target.trailing_distance_percent,
                target.trailing_activation_threshold,
            );
            if let Some(peak) = target.trailing_highest_price {
                tsl.peak_price = tsl.peak_price.max(peak);
            }
            if let Some(curr_sl) = target.trailing_current_sl {
                tsl.current_sl_percent = curr_sl;
                tsl.enabled = true;
            }
            trailing_monitors.insert(target.symbol.clone(), tsl);
        }
        if !liquidity_monitors.contains_key(&target.symbol) {
            liquidity_monitors.insert(target.symbol.clone(), LiquidityMonitor::new(20.0, 5.0));
        }

        // Actualización DB Asíncrona (No bloquea el loop)
        let _ = state_manager
            .update_position_price(&target.token_mint, price_update.price_native)
            .await;

        let position_data = monitor_clone
            .lock()
            .unwrap()
            .get_position(&target.token_mint)
            .cloned();

        if let Some(pos) = position_data {
            let dd = pos.drawdown_percent();
            let effective_sl_percent = if let Some(tsl) = trailing_monitors.get(&target.symbol) {
                tsl.current_sl_percent.max(target.stop_loss_percent)
            } else {
                target.stop_loss_percent
            };

            let current_gain_percent =
                ((pos.current_price - pos.entry_price) / pos.entry_price) * 100.0;

            // Logs limitados
            let now = Utc::now().timestamp();
            let last_printed = *last_log_time.get(&target.token_mint).unwrap_or(&0);
            if now - last_printed >= 15 {
                println!("┌────────────────────────────────────────────────────────┐");
                println!(
                    "│ {} Status [{}] {:>20} │",
                    target.symbol, price_update.source, dd
                );
                println!("├────────────────────────────────────────────────────────┤");
                println!(
                    "│   Price:    {:.8} SOL                  │",
                    pos.current_price
                );
                println!(
                    "│   PnL:      {:.2}%                                  │",
                    current_gain_percent
                );
                println!(
                    "│   SL Limit: {:.1}%                                  │",
                    effective_sl_percent
                );
                println!("└────────────────────────────────────────────────────────┘");
                last_log_time.insert(target.token_mint.clone(), now);
            }

            // --- 💰 TAKE PROFIT 1 💰 ---
            let tp_target_percent = target.tp_percent.unwrap_or(100.0);
            if !target.tp_triggered && current_gain_percent >= tp_target_percent {
                if !telegram::commands::CommandHandler::is_hibernating()
                    && app_config.global_settings.auto_execute
                    && !tp1_attempted.contains(&target.token_mint)
                {
                    tp1_attempted.insert(target.token_mint.clone());
                    println!(
                        "⚡ HFT Fire-and-Forget: TAKE PROFIT 1 para {}",
                        target.symbol
                    );

                    let mint_clone = target.token_mint.clone();
                    let kp_wrapper = wallet_keypair.as_ref().map(|k| k.insecure_clone());
                    let exc = Arc::clone(&executor_clone);
                    let state_mgr = Arc::clone(&state_manager);
                    let tel = Arc::clone(&telegram_clone);
                    let symbol = target.symbol.clone();
                    let sell_amount_pct = target.tp_amount_percent.unwrap_or(50.0) as u8;
                    let entry_price = pos.entry_price;
                    let amount_sol_invested = target.amount_sol;

                    // ⚡ DESACOPLAMIENTO: Hilo independiente
                    tokio::spawn(async move {
                        let sell_result = exc
                            .execute_sell_with_retry(
                                mint_clone.clone(),
                                kp_wrapper.as_ref(),
                                sell_amount_pct,
                                false,
                            )
                            .await;
                        if let Ok(res) = sell_result {
                            let _ = tel
                                .send_message(
                                    &format!(
                                        "💰 <b>TP1 HIT</b> para {}! Tx: {}\n⛽ Fee: {:.6} SOL",
                                        symbol, res.signature, res.fee_sol
                                    ),
                                    true,
                                )
                                .await;
                            // ⚡ Registrar trade TP1 con fee real
                            let sol_received = res.output_amount;
                            let trade = crate::state_manager::TradeRecord {
                                id: None,
                                signature: res.signature.clone(),
                                token_mint: mint_clone.clone(),
                                symbol: symbol.clone(),
                                trade_type: "AUTO_TP1".to_string(),
                                amount_sol: sol_received,
                                tokens_amount: res.input_amount,
                                price: if res.input_amount > 0.0 { sol_received / res.input_amount } else { 0.0 },
                                pnl_sol: Some(sol_received - (amount_sol_invested * (sell_amount_pct as f64 / 100.0))),
                                pnl_percent: Some(((sol_received / (amount_sol_invested * (sell_amount_pct as f64 / 100.0))) - 1.0) * 100.0),
                                route: res.route.clone(),
                                price_impact_pct: res.price_impact_pct,
                                fee_sol: res.fee_sol,
                                timestamp: chrono::Utc::now().timestamp(),
                            };
                            let _ = state_mgr.record_trade(trade).await;
                            let _ = state_mgr.mark_tp_triggered(&mint_clone).await;
                            // Actualizar amount asíncronamente
                            let remaining =
                                amount_sol_invested * (1.0 - (sell_amount_pct as f64 / 100.0));
                            let _ = state_mgr
                                .update_amount_invested(&mint_clone, remaining)
                                .await;
                            let _ = entry_price; // usado en pnl calc
                        }
                    });
                }
            }

            // --- 🚀 TAKE PROFIT 2 (MOONBAG) 🚀 ---
            let tp2_target_percent = target.tp2_percent.unwrap_or(200.0);
            if target.tp_triggered
                && !target.tp2_triggered
                && current_gain_percent >= tp2_target_percent
            {
                if !telegram::commands::CommandHandler::is_hibernating()
                    && app_config.global_settings.auto_execute
                    && !tp2_attempted.contains(&target.token_mint)
                {
                    tp2_attempted.insert(target.token_mint.clone());
                    println!(
                        "⚡ HFT Fire-and-Forget: TAKE PROFIT 2 para {}",
                        target.symbol
                    );

                    let mint_clone = target.token_mint.clone();
                    let kp_wrapper = wallet_keypair.as_ref().map(|k| k.insecure_clone());
                    let exc = Arc::clone(&executor_clone);
                    let state_mgr = Arc::clone(&state_manager);
                    let tel = Arc::clone(&telegram_clone);
                    let symbol = target.symbol.clone();
                    let sell_amount_pct = target.tp2_amount_percent.unwrap_or(100.0) as u8;
                    let amount_sol_invested = target.amount_sol;

                    tokio::spawn(async move {
                        let sell_result = exc
                            .execute_sell_with_retry(
                                mint_clone.clone(),
                                kp_wrapper.as_ref(),
                                sell_amount_pct,
                                false,
                            )
                            .await;
                        if let Ok(res) = sell_result {
                            let _ = tel
                                .send_message(
                                    &format!(
                                        "🚀 <b>TP2 HIT</b> para {}! Tx: {}\n⛽ Fee: {:.6} SOL",
                                        symbol, res.signature, res.fee_sol
                                    ),
                                    true,
                                )
                                .await;
                            // ⚡ Registrar trade TP2 con fee real
                            let sol_received = res.output_amount;
                            let trade = crate::state_manager::TradeRecord {
                                id: None,
                                signature: res.signature.clone(),
                                token_mint: mint_clone.clone(),
                                symbol: symbol.clone(),
                                trade_type: "AUTO_TP2".to_string(),
                                amount_sol: sol_received,
                                tokens_amount: res.input_amount,
                                price: if res.input_amount > 0.0 { sol_received / res.input_amount } else { 0.0 },
                                pnl_sol: Some(sol_received - (amount_sol_invested * (sell_amount_pct as f64 / 100.0))),
                                pnl_percent: Some(((sol_received / (amount_sol_invested * (sell_amount_pct as f64 / 100.0)).max(0.00000001)) - 1.0) * 100.0),
                                route: res.route.clone(),
                                price_impact_pct: res.price_impact_pct,
                                fee_sol: res.fee_sol,
                                timestamp: chrono::Utc::now().timestamp(),
                            };
                            let _ = state_mgr.record_trade(trade).await;
                            let _ = state_mgr.mark_tp2_triggered(&mint_clone).await;

                            if sell_amount_pct == 100 {
                                let _ = state_mgr.close_position(&mint_clone).await;
                            }
                        }
                    });
                }
            }

            // --- 🚨 STOP LOSS (EMERGENCY) 🚨 ---
            if dd <= effective_sl_percent {
                if telegram::commands::CommandHandler::is_hibernating() {
                    if !sl_alerted.contains(&target.token_mint) {
                        sl_alerted.insert(target.token_mint.clone());
                        let _ = telegram_clone.send_message(&format!("🛑 <b>SL de {} ({:.2}%) alcanzado pero bot hibernando.</b> Vende en Jupiter.", target.symbol, dd), true).await;
                    }
                } else if app_config.global_settings.auto_execute
                    && !sell_attempted.contains(&target.token_mint)
                {
                    sell_attempted.insert(target.token_mint.clone());
                    println!(
                        "⚡ HFT INJECTION: Emergency Sell para {} desatada.",
                        target.symbol
                    );

                    let mint_clone = target.token_mint.clone();
                    let kp_wrapper = wallet_keypair.as_ref().map(|k| k.insecure_clone());
                    let exc = Arc::clone(&executor_clone);
                    let state_mgr = Arc::clone(&state_manager);
                    let tel = Arc::clone(&telegram_clone);
                    let symbol = target.symbol.clone();

                    // ⚡ DESACOPLAMIENTO ABSOLUTO + OVERRIDE DE ECU
                    let amount_sol_invested = target.amount_sol;
                    tokio::spawn(async move {
                        // is_emergency = true (Activa modo agresivo Degen de slippage si falla)
                        let sell_result = exc
                            .execute_sell_with_retry(
                                mint_clone.clone(),
                                kp_wrapper.as_ref(),
                                100,
                                true,
                            )
                            .await;

                        match sell_result {
                            Ok(res) => {
                                println!("✅ SL HFT Ejecutado: {} | Fee: {:.6} SOL", res.signature, res.fee_sol);
                                let _ = tel
                                    .send_message(
                                        &format!(
                                            "✅ <b>SL HFT Completado para {}.</b>\nTx: {}\n⛽ Fee: {:.6} SOL",
                                            symbol, res.signature, res.fee_sol
                                        ),
                                        true,
                                    )
                                    .await;

                                // ⚡ Registrar trade SL con fee real
                                let sol_received = res.output_amount;
                                let trade = crate::state_manager::TradeRecord {
                                    id: None,
                                    signature: res.signature.clone(),
                                    token_mint: mint_clone.clone(),
                                    symbol: symbol.clone(),
                                    trade_type: "AUTO_SL".to_string(),
                                    amount_sol: sol_received,
                                    tokens_amount: res.input_amount,
                                    price: if res.input_amount > 0.0 { sol_received / res.input_amount } else { 0.0 },
                                    pnl_sol: Some(sol_received - amount_sol_invested),
                                    pnl_percent: Some(((sol_received / amount_sol_invested.max(0.000001)) - 1.0) * 100.0),
                                    route: res.route.clone(),
                                    price_impact_pct: res.price_impact_pct,
                                    fee_sol: res.fee_sol,
                                    timestamp: chrono::Utc::now().timestamp(),
                                };
                                let _ = state_mgr.record_trade(trade).await;

                                // ⚡ Cierre Atómico DB
                                if let Err(e) = state_mgr.close_position(&mint_clone).await {
                                    eprintln!("❌ Error de integridad cerrando posición atómica en SQLite: {}", e);
                                }
                            }
                            Err(e) => {
                                eprintln!("💥 Fallo irrecuperable en SL para {}: {}", symbol, e);
                                // NO CERRAMOS EN DB. SE PERMITE INTENTO MANUAL.
                                let _ = tel.send_error_alert(&format!("❌ <b>Fallo en SL de {}</b>: {}\nLa posición SIGUE ABIERTA en base de datos. Vende manualmente.", symbol, e)).await;
                            }
                        }
                    });
                } else if !app_config.global_settings.auto_execute
                    && !sl_alerted.contains(&target.token_mint)
                {
                    sl_alerted.insert(target.token_mint.clone());
                    let _ = telegram_clone
                        .send_message(
                            &format!(
                                "⚠️ <b>SL de {} ({:.2}%) alcanzado. Auto-execute OFF.</b>",
                                target.symbol, dd
                            ),
                            true,
                        )
                        .await;
                }
            }
        }

        // Update Trailing SL localmente
        if let Some(tsl) = trailing_monitors.get_mut(&target.symbol) {
            let _ = tsl.update(price_update.price_usd);
        }
    }

    Ok(())
}
