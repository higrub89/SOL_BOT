use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};
use crate::engine::commands::{ExecutionCommand, ExecutionFeedback, CommandType, AuditMetadata};
use crate::executor_v2::TradeExecutor;
use crate::state_manager::{StateManager, ExecutionAudit, PositionState};
use crate::telegram::TelegramNotifier;
use solana_sdk::signature::Keypair;
use chrono::Utc;
use crate::jupiter::SwapResult;

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
        println!("⚙️ Execution Router v2.0 (Deterministic) Online.");

        while let Some(command) = cmd_rx.recv().await {
            let router_clone = Arc::clone(&self);

            tokio::spawn(async move {
                router_clone.process_command(command).await;
            });
        }
    }

    async fn process_command(&self, command: ExecutionCommand) {
        match command {
            ExecutionCommand::Buy { mint, symbol, amount_sol, slippage_bps, priority_fee, audit } => {
                println!("🛍️ [RUTEO] Procesando COMPRA para {} ({} SOL)", symbol, amount_sol);
                self.execute_buy_with_audit(&mint, &symbol, amount_sol, slippage_bps, priority_fee, audit).await;
            }
            ExecutionCommand::PanicAll => {
                println!("💥 [RUTEO] PANIC ALL EJECUTADO");
                if let Ok(positions) = self.state_manager.get_active_positions().await {
                    for pos in positions {
                        let audit = AuditMetadata {
                            signal_id: format!("PANIC_{}", Utc::now().timestamp()),
                            strategy_name: "SYSTEM_PANIC".to_string(),
                            rationale: "Manual Panic All Triggered".to_string(),
                            timestamp: Utc::now().timestamp(),
                        };
                        self.execute_sell_with_audit(&pos.token_mint, &pos.symbol, pos.amount_sol, 100, true, "PANIC_ALL", CommandType::StopLoss, audit).await;
                    }
                }
            }
            ExecutionCommand::StopLoss { mint, symbol, amount_invested, is_emergency, audit } => {
                println!("🚨 [RUTEO] Stop Loss para {}", symbol);
                self.execute_sell_with_audit(&mint, &symbol, amount_invested, 100, is_emergency, "AUTO_SL", CommandType::StopLoss, audit).await;
            }
            ExecutionCommand::TakeProfit1 { mint, symbol, sell_amount_pct, amount_invested, audit, .. } => {
                println!("💰 [RUTEO] Take Profit 1 para {}", symbol);
                self.execute_sell_with_audit(&mint, &symbol, amount_invested, sell_amount_pct, false, "AUTO_TP1", CommandType::TakeProfit1, audit).await;
            }
            ExecutionCommand::TakeProfit2 { mint, symbol, sell_amount_pct, amount_invested, audit } => {
                println!("💰💰 [RUTEO] Take Profit 2 para {}", symbol);
                self.execute_sell_with_audit(&mint, &symbol, amount_invested, sell_amount_pct, false, "AUTO_TP2", CommandType::TakeProfit2, audit).await;
            }
        }
    }

    async fn execute_buy_with_audit(
        &self,
        mint: &str,
        symbol: &str,
        amount_sol: f64,
        _slippage: u16,
        _priority_fee: u64,
        audit: AuditMetadata,
    ) {
        let kp_ref = self.wallet_kp.as_deref();
        
        // Simular ejecución (Integrar con Executor real en v2.1)
        let result = self.executor.execute_buy(mint, kp_ref, amount_sol).await;

        let (success, signature, error_msg) = match result {
            Ok(res) => (true, Some(res.signature), None),
            Err(e) => (false, None, Some(e.to_string())),
        };

        let audit_record = ExecutionAudit {
            id: None,
            signal_id: audit.signal_id,
            token_mint: mint.to_string(),
            command_type: "BUY".to_string(),
            strategy_name: audit.strategy_name.clone(),
            rationale: audit.rationale,
            decision_timestamp: audit.timestamp,
            execution_timestamp: Some(Utc::now().timestamp()),
            signature: signature.clone(),
            success,
            error_msg: error_msg.clone(),
        };

        let _ = self.state_manager.record_audit(audit_record).await;

        if success {
            println!("✅ COMPRA EJECUTADA: {} | Sig: {}", symbol, signature.as_deref().unwrap_or(""));
            // Crear posición en StateManager
            let pos = PositionState {
                id: None,
                token_mint: mint.to_string(),
                symbol: symbol.to_string(),
                entry_price: 0.0, // Pendiente de actualizar con precio real del swap
                amount_sol,
                current_price: 0.0,
                stop_loss_percent: -30.0, // Default SL
                trailing_enabled: true,
                trailing_distance_percent: 5.0,
                trailing_activation_threshold: 10.0,
                trailing_highest_price: None,
                trailing_current_sl: None,
                tp_percent: Some(50.0),
                tp_amount_percent: Some(50.0),
                tp_triggered: false,
                tp2_percent: Some(100.0),
                tp2_amount_percent: Some(100.0),
                tp2_triggered: false,
                active: true,
                created_at: Utc::now().timestamp(),
                updated_at: Utc::now().timestamp(),
            };
            let _ = self.state_manager.upsert_position(pos).await;
            let _ = self.feedback_tx.send(ExecutionFeedback::Success { mint: mint.to_string(), command_type: CommandType::Buy }).await;
            
            // Notificación Telegram
            let msg = format!(
                "🚀 <b>COMPRA EJECUTADA</b>\nToken: <code>{}</code>\nEstrategia: {}\nTx: <a href=\"https://solscan.io/tx/{}\">{}</a>",
                mint, audit.strategy_name, signature.as_deref().unwrap_or(""),
                &signature.as_deref().unwrap_or("          ")[..8]
            );
            let _ = self.telegram.send_message(&msg, true).await;
        } else {
            let msg = format!(
                "❌ <b>ERROR DE COMPRA</b>\nToken: <code>{}</code>\nError: <code>{}</code>",
                mint, error_msg.as_ref().unwrap_or(&"Error desconocido".to_string())
            );
            let _ = self.telegram.send_message(&msg, true).await;

            let _ = self.feedback_tx.send(ExecutionFeedback::Failure { 
                mint: mint.to_string(), 
                command_type: CommandType::Buy, 
                reason: error_msg.unwrap_or("Unknown buy error".to_string()) 
            }).await;
        }
    }

    async fn execute_sell_with_audit(
        &self,
        mint: &str,
        symbol: &str,
        invested: f64,
        pct: u8,
        is_emergency: bool,
        trade_type: &str,
        cmd_type: CommandType,
        audit: AuditMetadata,
    ) {
        let max_attempts = if is_emergency { 5 } else { 3 };
        let mut final_res = None;
        let mut error_msg = None;

        for attempt in 1..=max_attempts {
            let res = self.executor.execute_sell_with_retry(
                mint.to_string(),
                self.wallet_kp.as_deref(),
                pct,
                is_emergency,
            ).await;

            match res {
                Ok(r) => {
                    final_res = Some(r);
                    break;
                }
                Err(e) => {
                    error_msg = Some(e.to_string());
                    if attempt < max_attempts {
                        sleep(Duration::from_millis(500 * attempt as u64)).await;
                    }
                }
            }
        }

        let audit_record = ExecutionAudit {
            id: None,
            signal_id: audit.signal_id,
            token_mint: mint.to_string(),
            command_type: format!("{:?}", cmd_type),
            strategy_name: audit.strategy_name.clone(),
            rationale: audit.rationale,
            decision_timestamp: audit.timestamp,
            execution_timestamp: Some(Utc::now().timestamp()),
            signature: final_res.as_ref().map(|r| r.signature.clone()),
            success: final_res.is_some(),
            error_msg: error_msg.clone(),
        };

        let _ = self.state_manager.record_audit(audit_record).await;

        if let Some(res) = final_res {
            self.post_execution_cleanup(symbol, mint, invested, pct, res, trade_type, cmd_type).await;
        } else {
            let _ = self.feedback_tx.send(ExecutionFeedback::Failure { 
                mint: mint.to_string(), 
                command_type: cmd_type, 
                reason: error_msg.unwrap_or_default() 
            }).await;
        }
    }

    async fn post_execution_cleanup(
        &self,
        symbol: &str,
        mint: &str,
        invested: f64,
        pct: u8,
        res: SwapResult,
        trade_type: &str,
        cmd_type: CommandType,
    ) {
        // ... (Lógica de limpieza existente, adaptada si es necesario)
        let invested_portion = invested * (pct as f64 / 100.0);
        let sol_received = res.output_amount;
        let pnl_sol = sol_received - invested_portion;
        
        let trade = crate::state_manager::TradeRecord {
            id: None,
            signature: res.signature.clone(),
            token_mint: mint.to_string(),
            symbol: symbol.to_string(),
            trade_type: trade_type.to_string(),
            amount_sol: sol_received,
            tokens_amount: res.input_amount,
            price: if res.input_amount > 0.0 { sol_received / res.input_amount } else { 0.0 },
            pnl_sol: Some(pnl_sol),
            pnl_percent: Some(if invested_portion > 0.0 { (pnl_sol / invested_portion) * 100.0 } else { 0.0 }),
            route: res.route.clone(),
            price_impact_pct: res.price_impact_pct,
            fee_sol: res.fee_sol,
            timestamp: Utc::now().timestamp(),
        };

        let _ = self.state_manager.record_trade(trade).await;

        if trade_type == "AUTO_SL" || pct == 100 {
            let _ = self.state_manager.close_position(mint).await;
        }

        let _ = self.feedback_tx.send(ExecutionFeedback::Success { mint: mint.to_string(), command_type: cmd_type }).await;
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
        let telegram = Arc::new(TelegramNotifier::new().expect("Failed to initialize mock/test telegram")); 
        
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
        let audit = crate::engine::commands::AuditMetadata {
            signal_id: format!("TEST_SL_{}", Utc::now().timestamp()),
            strategy_name: "TEST".to_string(),
            rationale: "Unit test stoploss".to_string(),
            timestamp: Utc::now().timestamp(),
        };

        let cmd = ExecutionCommand::StopLoss {
            mint: test_mint.clone(),
            symbol: "GHOST".to_string(),
            amount_invested: 1.5,
            is_emergency: true,
            audit,
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
