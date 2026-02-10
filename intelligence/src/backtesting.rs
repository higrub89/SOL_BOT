use crate::strategy_engine::{Strategy, MarketData, TradeAction};
use std::collections::HashMap;
use anyhow::{Result, Context};
use std::time::Instant;

/// Resultados detallados de una sesión de backtesting
#[derive(Debug)]
pub struct BacktestResult {
    pub strategy_name: String,
    pub total_trades: u32,
    pub win_rate: f64,
    pub final_balance: f64,
    pub max_drawdown: f64,
    pub total_fees_paid: f64,
}

/// Simulador de mercado para backtesting de alta fidelidad
pub struct MarketSimulator {
    pub initial_balance: f64,
    pub slippage_maker: f64, // 0.1%
    pub slippage_taker: f64, // 0.3%
    pub fee_per_trade: f64,  // 0.000005 SOL
}

impl MarketSimulator {
    pub fn new(initial_balance: f64) -> Self {
        Self {
            initial_balance,
            slippage_maker: 0.001,
            slippage_taker: 0.003,
            fee_per_trade: 0.000005,
        }
    }

    /// Ejecuta una estrategia sobre un dataset histórico
    pub fn run<S: Strategy>(&self, strategy: &mut S, data: &[MarketData]) -> Result<BacktestResult> {
        let start_time = Instant::now();
        println!("🧪 Iniciando backtesting para '{}' con {} puntos de datos...", strategy.name(), data.len());
        
        strategy.initialize()?;
        
        let mut balance = self.initial_balance;
        let mut position: Option<(f64, f64)> = None; // (amount, entry_price)
        let mut trades = 0;
        let mut wins = 0;
        let mut max_balance = balance;
        let mut max_drawdown = 0.0;
        let mut fees_paid = 0.0;

        for tick in data {
            let action = strategy.on_price_update(tick)?;
            
            match action {
                TradeAction::Buy { confidence, .. } => {
                    if position.is_none() && balance > 0.01 {
                        // Simular compra con slippage de taker
                        let execution_price = tick.price * (1.0 + self.slippage_taker);
                        let amount = (balance - self.fee_per_trade) / execution_price;
                        
                        fees_paid += self.fee_per_trade;
                        balance -= amount * execution_price + self.fee_per_trade;
                        position = Some((amount, execution_price));
                        
                        println!("   🟢 BUY @ {:.6} (Amt: {:.4})", execution_price, amount);
                    }
                },
                TradeAction::Sell { amount_percent, .. } => {
                    if let Some((amount, entry)) = position {
                        // Simular venta
                        let execution_price = tick.price * (1.0 - self.slippage_taker);
                        let sell_amount = amount * (amount_percent as f64 / 100.0);
                        
                        let revenue = sell_amount * execution_price - self.fee_per_trade;
                        balance += revenue;
                        fees_paid += self.fee_per_trade;
                        
                        // Check Win
                        trades += 1;
                        if execution_price > entry {
                            wins += 1;
                        }

                        // Calcular Drawdown
                        if balance > max_balance {
                            max_balance = balance;
                        }
                        let dd = (max_balance - balance) / max_balance * 100.0;
                        if dd > max_drawdown {
                            max_drawdown = dd;
                        }

                        position = None; // Asumimos venta total por simplicidad
                        println!("   🔴 SELL @ {:.6} (P/L: {:.2}%)", execution_price, (execution_price/entry - 1.0)*100.0);
                    }
                },
                TradeAction::Hold => {}
            }
        }

        // Cerrar posición al final si existe
        if let Some((amount, entry)) = position {
            let last_price = data.last().map(|d| d.price).unwrap_or(0.0);
            let revenue = amount * last_price;
            balance += revenue;
            println!("   ⚠️  Cierre forzado al final @ {:.6}", last_price);
        }

        let duration = start_time.elapsed();
        println!("✅ Backtesting completado en {:?}", duration);

        Ok(BacktestResult {
            strategy_name: strategy.name().to_string(),
            total_trades: trades,
            win_rate: if trades > 0 { (wins as f64 / trades as f64) * 100.0 } else { 0.0 },
            final_balance: balance,
            max_drawdown,
            total_fees_paid: fees_paid,
        })
    }
}
