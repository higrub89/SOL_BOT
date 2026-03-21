use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CommandType {
    Buy,
    TakeProfit1,
    TakeProfit2,
    StopLoss,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditMetadata {
    pub signal_id: String,
    pub strategy_name: String,
    pub rationale: String,
    pub timestamp: i64,
}

#[derive(Debug, Clone)]
pub enum ExecutionCommand {
    Buy {
        mint: String,
        symbol: String,
        amount_sol: f64,
        slippage_bps: u16,
        priority_fee: u64,
        audit: AuditMetadata,
    },
    TakeProfit1 {
        mint: String,
        symbol: String,
        sell_amount_pct: u8,
        entry_price: f64,
        amount_invested: f64,
        audit: AuditMetadata,
    },
    TakeProfit2 {
        mint: String,
        symbol: String,
        sell_amount_pct: u8,
        amount_invested: f64,
        audit: AuditMetadata,
    },
    StopLoss {
        mint: String,
        symbol: String,
        amount_invested: f64,
        is_emergency: bool,
        audit: AuditMetadata,
    },
    PanicAll,
}

#[derive(Debug, Clone)]
pub enum ExecutionFeedback {
    Failure {
        mint: String,
        command_type: CommandType,
        reason: String,
    },
    Success {
        mint: String,
        command_type: CommandType,
    },
}
