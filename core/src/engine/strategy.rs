use std::sync::Arc;
use tokio::sync::mpsc;
use std::collections::{HashMap, HashSet};
use crate::price_feed::PriceUpdate;
use crate::engine::commands::{ExecutionCommand, ExecutionFeedback, CommandType, AuditMetadata};
use crate::state_manager::StateManager;
use crate::trailing_sl::TrailingStopLoss;
use chrono::Utc;

pub struct StrategyEngine {
    state_manager: Arc<StateManager>,
    sell_attempted: HashSet<String>,
    tp1_attempted: HashSet<String>,
    tp2_attempted: HashSet<String>,
    trailing_monitors: HashMap<String, TrailingStopLoss>,
    failure_counts: HashMap<String, u8>,
}

impl StrategyEngine {
    pub fn new(state_manager: Arc<StateManager>) -> Self {
        Self {
            state_manager,
            sell_attempted: HashSet::new(),
            tp1_attempted: HashSet::new(),
            tp2_attempted: HashSet::new(),
            trailing_monitors: HashMap::new(),
            failure_counts: HashMap::new(),
        }
    }

    pub async fn run_loop(
        mut self,
        mut price_rx: mpsc::Receiver<PriceUpdate>,
        cmd_tx: mpsc::Sender<ExecutionCommand>,
        mut feedback_rx: mpsc::Receiver<ExecutionFeedback>,
    ) {
        println!("🧠 Strategy Engine en línea. ECU operativa.");

        // Circuit Breaker System Variables
        let mut failed_execution_count = 0;
        let mut last_failure_time = tokio::time::Instant::now();
        let circuit_breaker_threshold = 3;
        let circuit_breaker_window = std::time::Duration::from_secs(60);
        let mut is_circuit_breaker_tripped = false;

        loop {
            tokio::select! {
                // CANAL 1: Telemetría de Mercado (Alta frecuencia)
                Some(tick) = price_rx.recv() => {
                    // Auto-reset Circuit Breaker if tripped for more than 5 minutes
                    if is_circuit_breaker_tripped {
                        let now = tokio::time::Instant::now();
                        if now.duration_since(last_failure_time) > std::time::Duration::from_secs(300) {
                            is_circuit_breaker_tripped = false;
                            failed_execution_count = 0;
                            println!("✅ [CIRCUIT BREAKER] Auto-restablecido (timeout de 5 min). Actuador en línea.");
                        }
                    }

                    if !is_circuit_breaker_tripped {
                        self.process_price_tick(tick, &cmd_tx).await;
                    }
                }

                // CANAL 2: Diagnóstico Interno (Resolución de fallos)
                Some(feedback) = feedback_rx.recv() => {
                    // Update Circuit Breaker
                    if let ExecutionFeedback::Failure { .. } = feedback {
                        let now = tokio::time::Instant::now();
                        if now.duration_since(last_failure_time) > circuit_breaker_window {
                            failed_execution_count = 1;
                        } else {
                            failed_execution_count += 1;
                        }
                        last_failure_time = now;
                        
                        if failed_execution_count >= circuit_breaker_threshold && !is_circuit_breaker_tripped {
                            is_circuit_breaker_tripped = true;
                            eprintln!("⚠️ [CIRCUIT BREAKER] ¡TRIPPED! 3 fallas ejecución en < 60s. ¡COMPRAS/VENTAS PAUSADAS!");
                        }
                    } else if let ExecutionFeedback::Success { .. } = feedback {
                        // Reset Circuit Breaker if a success happens
                        failed_execution_count = 0;
                        if is_circuit_breaker_tripped {
                            is_circuit_breaker_tripped = false;
                            println!("✅ [CIRCUIT BREAKER] ¡RESTABLECIDO! Actuador en línea.");
                        }
                    }

                    self.process_feedback(feedback).await;
                }

                else => {
                    println!("🛑 Señal de apagado recibida en ECU. Terminando loop.");
                    break;
                }
            }
        }
    }

    async fn process_price_tick(&mut self, tick: PriceUpdate, cmd_tx: &mpsc::Sender<ExecutionCommand>) {
        let target = match self.state_manager.get_position(&tick.token_mint).await {
            Ok(Some(p)) if p.active => p,
            _ => return,
        };

        let current_gain_percent = ((tick.price_native - target.entry_price) / target.entry_price) * 100.0;
        
        let tsl = self.trailing_monitors.entry(target.symbol.clone()).or_insert_with(|| {
            let mut t_sl = TrailingStopLoss::new(
                target.entry_price,
                target.stop_loss_percent,
                target.trailing_distance_percent,
                target.trailing_activation_threshold,
            );
            if let Some(peak) = target.trailing_highest_price {
                t_sl.peak_price = t_sl.peak_price.max(peak);
            }
            if let Some(curr_sl) = target.trailing_current_sl {
                t_sl.current_sl_percent = curr_sl;
                t_sl.enabled = true;
            }
            t_sl
        });

        let mut tsl_changed = false;
        if target.trailing_enabled {
            tsl_changed = tsl.update(tick.price_native);
        }
        
        let effective_sl_percent = tsl.current_sl_percent.max(target.stop_loss_percent);

        if tsl_changed {
             let state_mgr_clone = Arc::clone(&self.state_manager);
             let trailing_current_sl = tsl.current_sl_percent;
             let trailing_highest_price = tsl.peak_price;
             let mint_clone = target.token_mint.clone();
             
             tokio::spawn(async move {
                  let _ = state_mgr_clone.update_trailing_sl(&mint_clone, trailing_highest_price, trailing_current_sl).await;
             });
        }

        // --- TAKE PROFIT 1 ---
        let tp_target = target.tp_percent.unwrap_or(100.0);
        if !target.tp_triggered && current_gain_percent >= tp_target && !self.tp1_attempted.contains(&target.token_mint) {
            self.tp1_attempted.insert(target.token_mint.clone());
            let audit = AuditMetadata {
                signal_id: format!("TP1_{}_{}", target.symbol, Utc::now().timestamp()),
                strategy_name: "TrendFollowing_v2".to_string(),
                rationale: format!("Gain {:.2}% reached TP1 threshold {:.2}%", current_gain_percent, tp_target),
                timestamp: Utc::now().timestamp(),
            };
            let _ = cmd_tx.send(ExecutionCommand::TakeProfit1 {
                mint: target.token_mint.clone(),
                symbol: target.symbol.clone(),
                sell_amount_pct: target.tp_amount_percent.unwrap_or(50.0) as u8,
                entry_price: target.entry_price,
                amount_invested: target.amount_sol,
                audit,
            }).await;
        }

        // --- TAKE PROFIT 2 ---
        if let Some(tp2_target) = target.tp2_percent {
            if !target.tp2_triggered && current_gain_percent >= tp2_target && !self.tp2_attempted.contains(&target.token_mint) {
                self.tp2_attempted.insert(target.token_mint.clone());
                let audit = AuditMetadata {
                    signal_id: format!("TP2_{}_{}", target.symbol, Utc::now().timestamp()),
                    strategy_name: "TrendFollowing_v2".to_string(),
                    rationale: format!("Gain {:.2}% reached TP2 threshold {:.2}%", current_gain_percent, tp2_target),
                    timestamp: Utc::now().timestamp(),
                };
                let _ = cmd_tx.send(ExecutionCommand::TakeProfit2 {
                    mint: target.token_mint.clone(),
                    symbol: target.symbol.clone(),
                    sell_amount_pct: target.tp2_amount_percent.unwrap_or(100.0) as u8,
                    amount_invested: target.amount_sol,
                    audit,
                }).await;
            }
        }

        // --- STOP LOSS ---
        if current_gain_percent <= effective_sl_percent && !self.sell_attempted.contains(&target.token_mint) {
            self.sell_attempted.insert(target.token_mint.clone());
            let audit = AuditMetadata {
                signal_id: format!("SL_{}_{}", target.symbol, Utc::now().timestamp()),
                strategy_name: "TrendFollowing_v2".to_string(),
                rationale: format!("Gain {:.2}% hit SL threshold {:.2}%", current_gain_percent, effective_sl_percent),
                timestamp: Utc::now().timestamp(),
            };
            let _ = cmd_tx.send(ExecutionCommand::StopLoss {
                mint: target.token_mint.clone(),
                symbol: target.symbol.clone(),
                amount_invested: target.amount_sol,
                is_emergency: true,
                audit,
            }).await;
        }
    }

    async fn process_feedback(&mut self, feedback: ExecutionFeedback) {
        match feedback {
            ExecutionFeedback::Failure { mint, command_type, reason } => {
                println!("⚠️ [ECU] Fallo actuador {}: {}. Liberando bloqueos.", mint, reason);
                
                match command_type {
                    CommandType::Buy => {
                        println!("❌ Fallo en COMPRA para {}. Abortando secuencia.", mint);
                    }
                    CommandType::StopLoss => {
                        let count = self.failure_counts.entry(mint.clone()).or_insert(0);
                        *count += 1;
                        if *count >= 5 {
                            let _ = self.state_manager.close_position(&mint).await;
                        } else {
                            self.sell_attempted.remove(&mint);
                        }
                    }
                    CommandType::TakeProfit1 => {
                        self.tp1_attempted.remove(&mint);
                    }
                    CommandType::TakeProfit2 => {
                        self.tp2_attempted.remove(&mint);
                    }
                }
            }
            ExecutionFeedback::Success { .. } => {}
        }
    }
}
