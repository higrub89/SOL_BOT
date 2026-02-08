//! # The Chassis - Solana Trading Engine
//! 
//! v0.8.0 - Dynamic Configuration (Zero Recompile)

use anyhow::Result;
use chrono::Utc;
use solana_client::rpc_client::RpcClient;
use std::time::Instant;
use std::sync::{Arc, Mutex};
use std::fs;

mod config;
mod latency;
mod geyser;
mod wallet;
mod emergency;
mod websocket;
mod scanner;
mod jupiter;
mod executor_simple;
mod telegram;
mod telegram_commands;
mod trailing_sl;
mod liquidity_monitor;

use config::{AppConfig, TargetConfig};
use geyser::{GeyserClient, GeyserConfig};
use wallet::WalletMonitor;
use emergency::{EmergencyMonitor, EmergencyConfig, Position};
use scanner::PriceScanner;
use executor_simple::SimpleExecutor;
use telegram::TelegramNotifier;
use telegram_commands::CommandHandler;
use trailing_sl::TrailingStopLoss;
use liquidity_monitor::{LiquidityMonitor, LiquiditySnapshot};

/// Configuración del motor (API Keys siguen siendo estáticas por seguridad)
const HELIUS_RPC: &str = "https://mainnet.helius-rpc.com/?api-key=";

#[tokio::main]
async fn main() -> Result<()> {
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║         🏎️  THE CHASSIS - Solana Trading Engine          ║");
    println!("║       v0.8.0 - Dynamic Config (Zero Recompile)            ║");
    println!("╚════════════════════════════════════════════════════════════╝\n");

    // Cargar config y .env
    dotenv::dotenv().ok();
    
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
    println!("   • Auto-Execute:    {}", if app_config.global_settings.auto_execute { "ACTIVADO 🔴" } else { "DESACTIVADO 🟡" });
    println!("   • Intervalo:       {}s", app_config.global_settings.monitor_interval_sec);

    let api_key = std::env::var("HELIUS_API_KEY")
        .unwrap_or_else(|_| "1d8b1813-084e-41ed-8e93-87a503c496c6".to_string());
    let wallet_addr = std::env::var("WALLET_ADDRESS")
        .unwrap_or_else(|_| "6EJeiMFoBgQrUfbpt8jjXZdc5nASe2Kc8qzfVSyGrPQv".to_string());
    
    let rpc_url = format!("{}{}", HELIUS_RPC, api_key);
    
    // 1. Wallet Monitor
    println!("\n🏦 WALLET STATUS:");
    let wallet_monitor = Arc::new(WalletMonitor::new(rpc_url.clone(), &wallet_addr)?);
    let sol_balance = wallet_monitor.get_sol_balance()?;
    println!("   • Balance:   {:.4} SOL", sol_balance);
    
    println!("\n───────────────────────────────────────────────────────────\n");

    // 2. Emergency System Multi-Target Setup
    println!("🛡️  EMERGENCY SYSTEM (Multi-Target):");
    
    // Configuración base de emergencia
    let base_emergency_config = EmergencyConfig {
        max_loss_percent: -99.9,
        min_sol_balance: app_config.global_settings.min_sol_balance,
        min_asset_price: 0.0,
        enabled: true,
    };
    
    let emergency_monitor = Arc::new(Mutex::new(
        EmergencyMonitor::new(base_emergency_config)
    ));
    
    // Cargar targets activos
    for target in &app_config.targets {
        if !target.active { continue; }
        
        let position = Position {
            token_mint: target.symbol.clone(),
            entry_price: target.entry_price,
            amount_invested: target.amount_sol,
            current_price: target.entry_price,
            current_value: target.amount_sol,
        };
        
        emergency_monitor.lock().unwrap().add_position(position);
        println!("   • Cargado: {} (SL: {}%)", target.symbol, target.stop_loss_percent);
    }

    println!("\n───────────────────────────────────────────────────────────\n");

    // 3. Executor & Telegram Setup
    println!("⚡ EXECUTOR STATUS: SIMPLE (Browser-based)");
    let executor = Arc::new(SimpleExecutor::new());

    // 3.5 Telegram Notifier & Command Handler Setup
    let telegram = Arc::new(TelegramNotifier::new());
    let command_handler = Arc::new(CommandHandler::new());
    
    // Lanzar el receptor de comandos en segundo plano
    let cmd_handler_clone = Arc::clone(&command_handler);
    let cmd_emergency_monitor = Arc::clone(&emergency_monitor);
    let cmd_wallet_monitor = Arc::clone(&wallet_monitor);
    let cmd_config = Arc::new(app_config.clone());
    
    tokio::spawn(async move {
        println!("📱 Telegram Command Handler: ACTIVADO");
        let _ = cmd_handler_clone.process_commands(
            cmd_emergency_monitor,
            cmd_wallet_monitor,
            cmd_config
        ).await;
    });
    
    println!("\n───────────────────────────────────────────────────────────\n");

    // 4. Network Benchmark
    println!("📡 NETWORK STATUS:");
    let start = Instant::now();
    let rpc_client = RpcClient::new(rpc_url);
    if let Ok(slot) = rpc_client.get_slot() {
        let latency = start.elapsed().as_millis();
        println!("   • Slot:     {}", slot);
        println!("   • Latency:  {}ms (HTTP)", latency);
    }

    println!("\n═══════════════════════════════════════════════════════════");
    println!("  🚀 INICIANDO MONITOR DINÁMICO v0.8.0");
    println!("═══════════════════════════════════════════════════════════\n");
    println!("⏰ Start Time: {}", Utc::now().format("%Y-%m-%d %H:%M:%S UTC"));
    println!("💡 Tip: Edita targets.json para cambiar SL en caliente (requiere reinicio rápido)\n");
    
    println!("───────────────────────────────────────────────────────────\n");

    // 5. Price Scanner Dinámico
    let scanner = PriceScanner::new();
    let monitor_clone = Arc::clone(&emergency_monitor);
    let telegram_clone = Arc::clone(&telegram);
    let executor_clone = Arc::clone(&executor);
    let active_targets = app_config.targets.clone();
    let wallet_addr_clone = wallet_addr.clone();

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
            // Monitor de liquidez por defecto para todos los activos
            liquidity_monitors.insert(target.symbol.clone(), LiquidityMonitor::new(20.0, 5.0));
        }
    }
    
    // Loop principal de monitoreo
    loop {
        for target in &active_targets {
            if !target.active { continue; }
            
            // 1. Obtener precio
            match scanner.get_token_price(&target.mint).await {
                Ok(price) => {
                    // 2. Calcular valores
                    let tokens_held = target.amount_sol / target.entry_price;
                    let current_value = tokens_held * price.price_usd;
                    
                    // 3. Actualizar monitor
                    let mut monitor = monitor_clone.lock().unwrap();
                    monitor.update_position(&target.symbol, price.price_usd, current_value);
                    
                    // 4. Check específico para este target
                    if let Some(pos) = monitor.get_position(&target.symbol) {
                        let dd = pos.drawdown_percent();
                        let dist_to_sl = dd - target.stop_loss_percent;
                        
                        // Emoji status
                        let status_emoji = if dist_to_sl > 10.0 { "🟢" } else if dist_to_sl > 5.0 { "🟡" } else { "🔴" };
                        
                        println!("┌────────────────────────────────────────────────────────┐");
                        println!("│ {} {} Status                                    │", status_emoji, target.symbol);
                        println!("├────────────────────────────────────────────────────────┤");
                        println!("│   Price:    ${:.8}                         │", pos.current_price);
                        println!("│   Drawdown: {:.2}%                                  │", dd);
                        println!("│   SL Limit: {:.1}% (Dist: {:.2}%)                    │", target.stop_loss_percent, dist_to_sl);
                        println!("└────────────────────────────────────────────────────────┘");

                        // 5. Lógica de Emergencia Dinámica
                        if dd <= target.stop_loss_percent {
                            println!("\n╔════════════════════════════════════════════════════════════╗");
                            println!("║                  🚨 EMERGENCY ALERT! 🚨                   ║");
                            println!("║         SL ACTIVADO: {} @ {:.2}% (Limit: {:.1}%)          ║", target.symbol, dd, target.stop_loss_percent);
                            println!("╚════════════════════════════════════════════════════════════╝\n");
                            
                            if app_config.global_settings.auto_execute {
                                println!("⚡ ABRIENDO JUPITER EN NAVEGADOR...");
                                // Prepara la transacción visual
                                match executor_clone.execute_emergency_sell_url(
                                    &target.mint,
                                    &wallet_addr_clone,
                                    // Estimamos balance total basado en inversión inicial (muy aproximado)
                                    (tokens_held * 1_000_000.0) as u64, // Asumimos 6 decimales para quote informativo
                                    &target.symbol
                                ).await {
                                    Ok(url) => {
                                        // 📱 Enviar alerta de Telegram
                                        let _ = telegram_clone.send_stop_loss_alert(
                                            &target.symbol,
                                            pos.current_price,
                                            pos.entry_price,
                                            dd,
                                            target.stop_loss_percent,
                                            &url
                                        ).await;
                                    }
                                    Err(e) => {
                                        eprintln!("❌ Error ejecutando sell: {}", e);
                                        let _ = telegram_clone.send_error_alert(
                                            &format!("Error ejecutando venta de {}: {}", target.symbol, e)
                                        ).await;
                                    }
                                }
                            } else {
                                println!("⚠️  ACCIÓN MANUAL REQUERIDA: VENDER EN TROJAN O JUPITER");
                                // Enviar alerta de Telegram incluso en modo manual
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
                    // 1. Logica de Trailing Stop-Loss (Feature B)
                    if let Some(tsl) = trailing_monitors.get_mut(&target.symbol) {
                        if tsl.update(price.price_usd) {
                            // Si el SL cambió, podemos avisar o simplemente dejar que actúe en el próximo check
                        }
                    }

                    // 2. Logica de Liquidez (Feature C)
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

                    // 3. Logica de Emergencia (Standard / Trailing)
                    let current_sl = if let Some(tsl) = trailing_monitors.get(&target.symbol) {
                        tsl.current_sl_percent
                    } else {
                        target.stop_loss_percent
                    };

                    let drawdown = ((price.price_usd - target.entry_price) / target.entry_price) * 100.0;
                    
                    if drawdown <= current_sl {
                        // DISPARAR ALERTA
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
            
            // Breve pausa entre targets para no saturar
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        }
        
        // Pausa global del ciclo
        tokio::time::sleep(std::time::Duration::from_secs(app_config.global_settings.monitor_interval_sec)).await;
        println!(""); // Salto de línea entre ciclos
    }
}
