use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};
use crate::engine::commands::{ExecutionCommand, ExecutionFeedback, CommandType};
use crate::executor_v2::TradeExecutor;
use crate::state_manager::StateManager;
use crate::telegram::TelegramNotifier;
use solana_sdk::signature::Keypair;

pub struct ExecutionRouter {
    executor: Arc<TradeExecutor>,
    state_manager: Arc<StateManager>,
    telegram: Arc<TelegramNotifier>,
    wallet_kp: Option<Arc<Keypair>>,
    feedback_tx: mpsc::Sender<ExecutionFeedback>,
}

impl ExecutionRouter {
    pub fn new(
        executor: Arc<TradeExecutor>,
        state_manager: Arc<StateManager>,
        telegram: Arc<TelegramNotifier>,
        wallet_kp: Option<Keypair>,
        feedback_tx: mpsc::Sender<ExecutionFeedback>,
    ) -> Self {
        Self {
            executor,
            state_manager,
            telegram,
            wallet_kp: wallet_kp.map(Arc::new),
            feedback_tx,
        }
    }

    pub async fn run_dashboard(self: Arc<Self>, mut cmd_rx: mpsc::Receiver<ExecutionCommand>) {
        println!("⚙️ Execution Router online. Listos para actuación...");

        while let Some(command) = cmd_rx.recv().await {
            let router_clone = Arc::clone(&self);

            tokio::spawn(async move {
                router_clone.process_command(command).await;
            });
        }
    }

    async fn process_command(&self, command: ExecutionCommand) {
        match command {
            ExecutionCommand::StopLoss {
                mint,
                symbol,
                amount_invested,
                is_emergency,
            } => {
                println!("🚨 [RUTEO] Iniciando Emergency Sell para {}", symbol);
                self.execute_with_backoff(&mint, &symbol, amount_invested, 100, is_emergency, "AUTO_SL", CommandType::StopLoss).await;
            }
            ExecutionCommand::TakeProfit1 {
                mint,
                symbol,
                sell_amount_pct,
                amount_invested,
                .. 
            } => {
                println!("💰 [RUTEO] Procesando TAKE PROFIT 1 para {}", symbol);
                self.execute_with_backoff(&mint, &symbol, amount_invested, sell_amount_pct, false, "AUTO_TP1", CommandType::TakeProfit1).await;
            }
            ExecutionCommand::TakeProfit2 {
                mint,
                symbol,
                sell_amount_pct,
                amount_invested,
            } => {
                println!("💰💰 [RUTEO] Procesando TAKE PROFIT 2 para {}", symbol);
                self.execute_with_backoff(&mint, &symbol, amount_invested, sell_amount_pct, false, "AUTO_TP2", CommandType::TakeProfit2).await;
            }
        }
    }

    async fn execute_with_backoff(
        &self,
        mint: &str,
        symbol: &str,
        invested: f64,
        pct: u8,
        is_emergency: bool,
        trade_type: &str,
        cmd_type: CommandType,
    ) {
        let max_attempts = if is_emergency { 5 } else { 3 };
        let mut delay_ms = 500;
        let kp_ref = self.wallet_kp.as_deref();

        let mut final_result = None;

        for attempt in 1..=max_attempts {
            let result = self.executor.execute_sell_with_retry(
                mint.to_string(),
                kp_ref,
                pct,
                is_emergency,
            ).await;

            match result {
                Ok(res) => {
                    final_result = Some(res);
                    break;
                }
                Err(e) => {
                    eprintln!("⚠️ [RUTEO] Intento {}/{} fallido para {} ({}): {}", attempt, max_attempts, symbol, trade_type, e);

                    if attempt == max_attempts {
                        let error_msg = format!("❌ <b>Fallo definitivo ({}) en {}</b>: {}\nPosición sigue abierta. ¡Revisa manualmente!", trade_type, symbol, e);
                        let _ = self.telegram.send_error_alert(&error_msg).await;
                        
                        let _ = self.feedback_tx.send(ExecutionFeedback::Failure {
                            mint: mint.to_string(),
                            command_type: cmd_type,
                            reason: e.to_string(),
                        }).await;
                        
                        return;
                    }

                    sleep(Duration::from_millis(delay_ms)).await;
                    delay_ms *= 2; 
                }
            }
        }

        if let Some(res) = final_result {
            self.post_execution_cleanup(symbol, mint, invested, pct, res, trade_type, cmd_type).await;
        }
    }

    async fn post_execution_cleanup(
        &self,
        symbol: &str,
        mint: &str,
        invested: f64,
        pct: u8,
        res: crate::jupiter::SwapResult,
        trade_type: &str,
        cmd_type: CommandType,
    ) {
        let _ = self.telegram.send_message(
            &format!("✅ <b>{} EJECUTADO para {}</b>\nTx: {}\n⛽ Fee: {:.6} SOL", trade_type, symbol, res.signature, res.fee_sol),
            true
        ).await;

        let sol_received = res.output_amount;
        
        let invested_portion = invested * (pct as f64 / 100.0);
        let pnl_sol = sol_received - invested_portion;
        let pnl_pct = if invested_portion > 0.0 {
            ((sol_received / invested_portion) - 1.0) * 100.0
        } else { 
            0.0 
        };

        let price_executed = if res.input_amount > 0.0 { 
            sol_received / res.input_amount 
        } else { 
            0.0 
        };

        let trade = crate::state_manager::TradeRecord {
            id: None,
            signature: res.signature.clone(),
            token_mint: mint.to_string(),
            symbol: symbol.to_string(),
            trade_type: trade_type.to_string(),
            amount_sol: sol_received,
            tokens_amount: res.input_amount,
            price: price_executed,
            pnl_sol: Some(pnl_sol),
            pnl_percent: Some(pnl_pct),
            route: res.route.clone(),
            price_impact_pct: res.price_impact_pct,
            fee_sol: res.fee_sol,
            timestamp: chrono::Utc::now().timestamp(),
        };

        if let Err(e) = self.state_manager.record_trade(trade).await {
            eprintln!("❌ DB ERROR registrando {} para {}: {}", trade_type, symbol, e);
        }

        if trade_type == "AUTO_SL" || pct == 100 {
            let _ = self.state_manager.close_position(mint).await;
        } else if trade_type == "AUTO_TP1" {
            let _ = self.state_manager.mark_tp_triggered(mint).await;
            let remaining = invested * (1.0 - (pct as f64 / 100.0));
            let _ = self.state_manager.update_amount_invested(mint, remaining).await;
        } else if trade_type == "AUTO_TP2" {
            let _ = self.state_manager.mark_tp2_triggered(mint).await;
            let remaining = invested * (1.0 - (pct as f64 / 100.0));
            let _ = self.state_manager.update_amount_invested(mint, remaining).await;
        }

        let _ = self.feedback_tx.send(ExecutionFeedback::Success {
            mint: mint.to_string(),
            command_type: cmd_type,
        }).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use tokio::sync::mpsc;
    use solana_sdk::signature::Keypair;
    use crate::engine::commands::{CommandType, ExecutionCommand, ExecutionFeedback};
    use crate::executor_v2::{TradeExecutor, ExecutorConfig};
    use crate::state_manager::StateManager;
    use crate::telegram::TelegramNotifier;

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_actuator_feedback_loop_on_failure() {
        println!("🔧 Iniciando test estático del Actuador (Fallo de Red Simulado)");

        // 1. Configurar Canales de Telemetría
        let (cmd_tx, cmd_rx) = mpsc::channel::<ExecutionCommand>(10);
        let (feedback_tx, mut feedback_rx) = mpsc::channel::<ExecutionFeedback>(10);

        // 2. Mocking de Dependencias usando endpoints irreales locales para forzar Err inmediato
        let executor_config = ExecutorConfig::new("http://127.0.0.1:0".to_string(), true);
        let executor = Arc::new(TradeExecutor::new(executor_config)); 
        let state_manager = Arc::new(StateManager::new("sqlite::memory:").await.unwrap());
        let telegram = Arc::new(TelegramNotifier::new()); // Mock al estar vacío env
        
        let router = ExecutionRouter::new(
            executor,
            state_manager,
            telegram,
            Some(Keypair::new()), // Keypair aleatorio vacío
            feedback_tx,
        );

        // 3. Arrancar el Router en un hilo separado
        tokio::spawn(async move {
            Arc::new(router).run_dashboard(cmd_rx).await;
        });

        // 4. Inyectar un comando crítico simulado
        // is_emergency: true -> esto provocará 5 intentos
        let test_mint = "TokenFantasma111111111111111111111111111111".to_string();
        let cmd = ExecutionCommand::StopLoss {
            mint: test_mint.clone(),
            symbol: "GHOST".to_string(),
            amount_invested: 1.5,
            is_emergency: true, 
        };

        cmd_tx.send(cmd).await.expect("Fallo al inyectar comando en el bus");

        // 5. Escuchar la respuesta del ECU tras agotar backoff 
        // 500ms + 1000ms + 2000ms + 4000ms = 7.5segs aprox
        // Backoff: 400ms + 800ms + 1600ms + 3200ms = 6s + ~5x Reqwest timeout.
        let timeout_duration = std::time::Duration::from_secs(90);
        let feedback_result = tokio::time::timeout(timeout_duration, feedback_rx.recv()).await;

        match feedback_result {
            Ok(Some(ExecutionFeedback::Failure { mint, command_type, reason })) => {
                assert_eq!(mint, test_mint, "El mint del feedback no coincide");
                assert_eq!(command_type, CommandType::StopLoss, "El tipo de comando no coincide");
                println!("✅ Test Pasado: Feedback de fallo recibido correctamente. Razón: {}", reason);
            }
            Ok(Some(ExecutionFeedback::Success { .. })) => {
                panic!("❌ Test Fallido: Se esperaba un fallo, pero el actuador reportó éxito.");
            }
            Ok(None) => {
                panic!("❌ Test Fallido: El canal de feedback se cerró inesperadamente.");
            }
            Err(_) => {
                panic!("❌ Test Fallido: Timeout alcanzado. El actuador se quedó colgado y no respondió.");
            }
        }
    }
}
