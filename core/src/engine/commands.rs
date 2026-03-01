#[derive(Debug, Clone, PartialEq)]
pub enum CommandType {
    TakeProfit1,
    TakeProfit2,
    StopLoss,
}

#[derive(Debug, Clone)]
pub enum ExecutionCommand {
    TakeProfit1 {
        mint: String,
        symbol: String,
        sell_amount_pct: u8,
        entry_price: f64,
        amount_invested: f64,
    },
    TakeProfit2 {
        mint: String,
        symbol: String,
        sell_amount_pct: u8,
        amount_invested: f64,
    },
    StopLoss {
        mint: String,
        symbol: String,
        amount_invested: f64,
        is_emergency: bool,
    },
}

#[derive(Debug, Clone)]
pub enum ExecutionFeedback {
    /// El actuador falló definitivamente tras agotar los reintentos
    Failure {
        mint: String,
        command_type: CommandType,
        reason: String,
    },
    /// (Opcional) El actuador reporta éxito, útil si la ECU necesita llevar contadores internos
    Success {
        mint: String,
        command_type: CommandType,
    },
}
