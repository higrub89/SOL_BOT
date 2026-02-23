//! # State Manager - Persistent Trading State
//! 
//! Sistema de persistencia para posiciones, configuración y historial de trades.
//! Usa SQLite para garantizar que el bot nunca pierda información crítica.

use anyhow::{Result, Context};
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use chrono::Utc;

// ============================================================================
// DATA STRUCTURES
// ============================================================================

/// Estado completo de una posición activa
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionState {
    pub id: Option<i64>,
    pub token_mint: String,
    pub symbol: String,
    pub entry_price: f64,
    pub amount_sol: f64,
    pub current_price: f64,
    pub stop_loss_percent: f64,
    pub trailing_enabled: bool,
    pub trailing_distance_percent: f64,
    pub trailing_activation_threshold: f64,
    pub trailing_highest_price: Option<f64>,
    pub trailing_current_sl: Option<f64>,
    pub tp_percent: Option<f64>, // Take Profit Target %
    pub tp_amount_percent: Option<f64>, // % of stack to sell
    pub tp_triggered: bool,
    pub tp2_percent: Option<f64>, // Moonbag TP Target %
    pub tp2_amount_percent: Option<f64>, // % of stack to sell for TP2
    pub tp2_triggered: bool,
    pub active: bool,
    pub created_at: i64,
    pub updated_at: i64,
}

/// Registro de un trade ejecutado
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeRecord {
    pub id: Option<i64>,
    pub signature: String,
    pub token_mint: String,
    pub symbol: String,
    pub trade_type: String, // "BUY" | "SELL" | "EMERGENCY_SELL" | "TAKE_PROFIT"
    pub amount_sol: f64,
    pub tokens_amount: f64,
    pub price: f64,
    pub pnl_sol: Option<f64>,
    pub pnl_percent: Option<f64>,
    pub route: String,
    pub price_impact_pct: f64,
    pub timestamp: i64,
}

/// Snapshot de configuración
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigSnapshot {
    pub id: Option<i64>,
    pub config_json: String,
    pub timestamp: i64,
}

// ============================================================================
// STATE MANAGER
// ============================================================================

pub struct StateManager {
    conn: Arc<Mutex<Connection>>,
}

impl StateManager {
    /// Inicializa el State Manager y crea las tablas si no existen
    pub fn new(db_path: &str) -> Result<Self> {
        let conn = Connection::open(db_path)
            .context("Failed to open SQLite database")?;
        
        let manager = Self {
            conn: Arc::new(Mutex::new(conn)),
        };
        
        manager.initialize_schema()?;
        manager.run_migrations()?;
        
        println!("✅ State Manager inicializado: {}", db_path);
        
        Ok(manager)
    }
    
    /// Crea las tablas necesarias
    fn initialize_schema(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        
        // Tabla de posiciones activas
        conn.execute(
            "CREATE TABLE IF NOT EXISTS positions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                token_mint TEXT NOT NULL UNIQUE,
                symbol TEXT NOT NULL,
                entry_price REAL NOT NULL,
                amount_sol REAL NOT NULL,
                current_price REAL NOT NULL,
                stop_loss_percent REAL NOT NULL,
                trailing_enabled INTEGER NOT NULL,
                trailing_distance_percent REAL NOT NULL,
                trailing_activation_threshold REAL NOT NULL,
                trailing_highest_price REAL,
                trailing_current_sl REAL,
                tp_percent REAL,
                tp_amount_percent REAL,
                tp_triggered INTEGER DEFAULT 0,
                tp2_percent REAL,
                tp2_amount_percent REAL,
                tp2_triggered INTEGER DEFAULT 0,
                active INTEGER NOT NULL,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            )",
            [],
        )?;
        
        // Tabla de historial de trades
        conn.execute(
            "CREATE TABLE IF NOT EXISTS trades (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                signature TEXT NOT NULL UNIQUE,
                token_mint TEXT NOT NULL,
                symbol TEXT NOT NULL,
                trade_type TEXT NOT NULL,
                amount_sol REAL NOT NULL,
                tokens_amount REAL NOT NULL,
                price REAL NOT NULL,
                pnl_sol REAL,
                pnl_percent REAL,
                route TEXT NOT NULL,
                price_impact_pct REAL NOT NULL,
                timestamp INTEGER NOT NULL
            )",
            [],
        )?;
        
        // Tabla de snapshots de configuración
        conn.execute(
            "CREATE TABLE IF NOT EXISTS config_snapshots (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                config_json TEXT NOT NULL,
                timestamp INTEGER NOT NULL
            )",
            [],
        )?;
        
        // Índices para búsquedas rápidas
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_positions_active ON positions(active)",
            [],
        )?;
        
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_trades_timestamp ON trades(timestamp DESC)",
            [],
        )?;
        
        Ok(())
    }

    /// Ejecuta migraciones de esquema para actualizar DBs existentes
    fn run_migrations(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();

        // Migration: Add TP columns if they don't exist
        // Note: SQLite doesn't support IF NOT EXISTS for ADD COLUMN, so we catch errors
        let _ = conn.execute("ALTER TABLE positions ADD COLUMN tp_percent REAL", []);
        let _ = conn.execute("ALTER TABLE positions ADD COLUMN tp_amount_percent REAL", []);
        let _ = conn.execute("ALTER TABLE positions ADD COLUMN tp_triggered INTEGER DEFAULT 0", []);
        let _ = conn.execute("ALTER TABLE positions ADD COLUMN tp2_percent REAL", []);
        let _ = conn.execute("ALTER TABLE positions ADD COLUMN tp2_amount_percent REAL", []);
        let _ = conn.execute("ALTER TABLE positions ADD COLUMN tp2_triggered INTEGER DEFAULT 0", []);

        Ok(())
    }
    
    // ========================================================================
    // POSITION OPERATIONS
    // ========================================================================
    
    /// Guarda o actualiza una posición
    pub fn upsert_position(&self, position: &PositionState) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        
        let now = Utc::now().timestamp();
        
        conn.execute(
            "INSERT INTO positions (
                token_mint, symbol, entry_price, amount_sol, current_price,
                stop_loss_percent, trailing_enabled, trailing_distance_percent,
                trailing_activation_threshold, trailing_highest_price,
                trailing_current_sl, tp_percent, tp_amount_percent, tp_triggered,
                tp2_percent, tp2_amount_percent, tp2_triggered,
                active, created_at, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20)
            ON CONFLICT(token_mint) DO UPDATE SET
                current_price = excluded.current_price,
                stop_loss_percent = excluded.stop_loss_percent,
                trailing_highest_price = excluded.trailing_highest_price,
                trailing_current_sl = excluded.trailing_current_sl,
                tp_percent = excluded.tp_percent,
                tp_amount_percent = excluded.tp_amount_percent,
                tp_triggered = excluded.tp_triggered,
                tp2_percent = excluded.tp2_percent,
                tp2_amount_percent = excluded.tp2_amount_percent,
                tp2_triggered = excluded.tp2_triggered,
                active = excluded.active,
                updated_at = excluded.updated_at",
            params![
                position.token_mint,
                position.symbol,
                position.entry_price,
                position.amount_sol,
                position.current_price,
                position.stop_loss_percent,
                position.trailing_enabled as i32,
                position.trailing_distance_percent,
                position.trailing_activation_threshold,
                position.trailing_highest_price,
                position.trailing_current_sl,
                position.tp_percent,
                position.tp_amount_percent,
                position.tp_triggered as i32,
                position.tp2_percent,
                position.tp2_amount_percent,
                position.tp2_triggered as i32,
                position.active as i32,
                position.created_at,
                now,
            ],
        )?;
        
        Ok(())
    }
    
    /// Obtiene todas las posiciones activas
    pub fn get_active_positions(&self) -> Result<Vec<PositionState>> {
        let conn = self.conn.lock().unwrap();
        
        let mut stmt = conn.prepare(
            "SELECT id, token_mint, symbol, entry_price, amount_sol, current_price,
                    stop_loss_percent, trailing_enabled, trailing_distance_percent,
                    trailing_activation_threshold, trailing_highest_price,
                    trailing_current_sl, tp_percent, tp_amount_percent, tp_triggered,
                    active, created_at, updated_at
             FROM positions
             WHERE active = 1
             ORDER BY created_at DESC"
        )?;
        
        let positions = stmt.query_map([], |row| {
            Ok(PositionState {
                id: Some(row.get(0)?),
                token_mint: row.get(1)?,
                symbol: row.get(2)?,
                entry_price: row.get(3)?,
                amount_sol: row.get(4)?,
                current_price: row.get(5)?,
                stop_loss_percent: row.get(6)?,
                trailing_enabled: row.get::<_, i32>(7)? != 0,
                trailing_distance_percent: row.get(8)?,
                trailing_activation_threshold: row.get(9)?,
                trailing_highest_price: row.get(10)?,
                trailing_current_sl: row.get(11)?,
                tp_percent: row.get(12).ok(),
                tp_amount_percent: row.get(13).ok(),
                tp_triggered: row.get::<_, i32>(14).unwrap_or(0) != 0,
                tp2_percent: row.get(15).ok(),
                tp2_amount_percent: row.get(16).ok(),
                tp2_triggered: row.get::<_, i32>(17).unwrap_or(0) != 0,
                active: row.get::<_, i32>(18)? != 0,
                created_at: row.get(19)?,
                updated_at: row.get(20)?,
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;
        
        Ok(positions)
    }
    
    /// Obtiene una posición específica por mint
    pub fn get_position(&self, token_mint: &str) -> Result<Option<PositionState>> {
        let conn = self.conn.lock().unwrap();
        
        let mut stmt = conn.prepare(
            "SELECT id, token_mint, symbol, entry_price, amount_sol, current_price,
                    stop_loss_percent, trailing_enabled, trailing_distance_percent,
                    trailing_activation_threshold, trailing_highest_price,
                    trailing_current_sl, tp_percent, tp_amount_percent, tp_triggered,
                    tp2_percent, tp2_amount_percent, tp2_triggered,
                    active, created_at, updated_at
             FROM positions
             WHERE token_mint = ?1"
        )?;
        
        let mut rows = stmt.query(params![token_mint])?;
        
        if let Some(row) = rows.next()? {
            Ok(Some(PositionState {
                id: Some(row.get(0)?),
                token_mint: row.get(1)?,
                symbol: row.get(2)?,
                entry_price: row.get(3)?,
                amount_sol: row.get(4)?,
                current_price: row.get(5)?,
                stop_loss_percent: row.get(6)?,
                trailing_enabled: row.get::<_, i32>(7)? != 0,
                trailing_distance_percent: row.get(8)?,
                trailing_activation_threshold: row.get(9)?,
                trailing_highest_price: row.get(10)?,
                trailing_current_sl: row.get(11)?,
                tp_percent: row.get(12).ok(),
                tp_amount_percent: row.get(13).ok(),
                tp_triggered: row.get::<_, i32>(14).unwrap_or(0) != 0,
                tp2_percent: row.get(15).ok(),
                tp2_amount_percent: row.get(16).ok(),
                tp2_triggered: row.get::<_, i32>(17).unwrap_or(0) != 0,
                active: row.get::<_, i32>(18)? != 0,
                created_at: row.get(19)?,
                updated_at: row.get(20)?,
            }))
        } else {
            Ok(None)
        }
    }
    
    /// Marca el TP como disparado
    pub fn mark_tp_triggered(&self, token_mint: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        
        conn.execute(
            "UPDATE positions SET tp_triggered = 1, updated_at = ?1 WHERE token_mint = ?2",
            params![Utc::now().timestamp(), token_mint],
        )?;
        
        Ok(())
    }

    /// Marca el Take Profit 2 (Moonbag) como ejecutado
    pub fn mark_tp2_triggered(&self, token_mint: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        
        conn.execute(
            "UPDATE positions SET tp2_triggered = 1, updated_at = ?1 WHERE token_mint = ?2",
            params![Utc::now().timestamp(), token_mint],
        )?;
        
        Ok(())
    }

    /// Actualiza el monto invertido (útil para TP parciales)
    pub fn update_amount_invested(&self, token_mint: &str, new_amount_sol: f64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        
        conn.execute(
            "UPDATE positions SET amount_sol = ?1, updated_at = ?2 WHERE token_mint = ?3",
            params![new_amount_sol, Utc::now().timestamp(), token_mint],
        )?;
        
        Ok(())
    }
    
    /// Marca una posición como inactiva (cerrada)
    pub fn close_position(&self, token_mint: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        
        conn.execute(
            "UPDATE positions SET active = 0, updated_at = ?1 WHERE token_mint = ?2",
            params![Utc::now().timestamp(), token_mint],
        )?;
        
        Ok(())
    }
    
    /// Actualiza el precio actual de una posición
    pub fn update_position_price(&self, token_mint: &str, current_price: f64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        
        conn.execute(
            "UPDATE positions SET current_price = ?1, updated_at = ?2 WHERE token_mint = ?3",
            params![current_price, Utc::now().timestamp(), token_mint],
        )?;
        
        Ok(())
    }
    
    /// Actualiza el estado del Trailing Stop Loss
    pub fn update_trailing_sl(
        &self,
        token_mint: &str,
        highest_price: f64,
        current_sl: f64,
    ) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        
        conn.execute(
            "UPDATE positions 
             SET trailing_highest_price = ?1, 
                 trailing_current_sl = ?2,
                 updated_at = ?3
             WHERE token_mint = ?4",
            params![highest_price, current_sl, Utc::now().timestamp(), token_mint],
        )?;
        
        Ok(())
    }
    
    // ========================================================================
    // TRADE HISTORY OPERATIONS
    // ========================================================================
    
    /// Registra un trade ejecutado
    pub fn record_trade(&self, trade: &TradeRecord) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        
        conn.execute(
            "INSERT INTO trades (
                signature, token_mint, symbol, trade_type, amount_sol,
                tokens_amount, price, pnl_sol, pnl_percent, route,
                price_impact_pct, timestamp
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                trade.signature,
                trade.token_mint,
                trade.symbol,
                trade.trade_type,
                trade.amount_sol,
                trade.tokens_amount,
                trade.price,
                trade.pnl_sol,
                trade.pnl_percent,
                trade.route,
                trade.price_impact_pct,
                trade.timestamp,
            ],
        )?;
        
        Ok(())
    }
    
    /// Obtiene el historial de trades (últimos N)
    pub fn get_trade_history(&self, limit: usize) -> Result<Vec<TradeRecord>> {
        let conn = self.conn.lock().unwrap();
        
        let mut stmt = conn.prepare(
            "SELECT id, signature, token_mint, symbol, trade_type, amount_sol,
                    tokens_amount, price, pnl_sol, pnl_percent, route,
                    price_impact_pct, timestamp
             FROM trades
             ORDER BY timestamp DESC
             LIMIT ?1"
        )?;
        
        let trades = stmt.query_map(params![limit], |row| {
            Ok(TradeRecord {
                id: Some(row.get(0)?),
                signature: row.get(1)?,
                token_mint: row.get(2)?,
                symbol: row.get(3)?,
                trade_type: row.get(4)?,
                amount_sol: row.get(5)?,
                tokens_amount: row.get(6)?,
                price: row.get(7)?,
                pnl_sol: row.get(8)?,
                pnl_percent: row.get(9)?,
                route: row.get(10)?,
                price_impact_pct: row.get(11)?,
                timestamp: row.get(12)?,
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;
        
        Ok(trades)
    }
    
    /// Calcula PnL total de todos los trades
    pub fn calculate_total_pnl(&self) -> Result<(f64, f64)> {
        let conn = self.conn.lock().unwrap();
        
        let mut stmt = conn.prepare(
            "SELECT COALESCE(SUM(pnl_sol), 0.0), COUNT(*) 
             FROM trades 
             WHERE pnl_sol IS NOT NULL"
        )?;
        
        let (total_pnl, count): (f64, i64) = stmt.query_row([], |row| {
            Ok((row.get(0)?, row.get(1)?))
        })?;
        
        Ok((total_pnl, count as f64))
    }
    
    // ========================================================================
    // CONFIG SNAPSHOT OPERATIONS
    // ========================================================================
    
    /// Guarda un snapshot de la configuración actual
    pub fn save_config_snapshot(&self, config_json: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        
        conn.execute(
            "INSERT INTO config_snapshots (config_json, timestamp) VALUES (?1, ?2)",
            params![config_json, Utc::now().timestamp()],
        )?;
        
        Ok(())
    }
    
    /// Obtiene el último snapshot de configuración
    pub fn get_latest_config_snapshot(&self) -> Result<Option<ConfigSnapshot>> {
        let conn = self.conn.lock().unwrap();
        
        let mut stmt = conn.prepare(
            "SELECT id, config_json, timestamp 
             FROM config_snapshots 
             ORDER BY timestamp DESC 
             LIMIT 1"
        )?;
        
        let mut rows = stmt.query([])?;
        
        if let Some(row) = rows.next()? {
            Ok(Some(ConfigSnapshot {
                id: Some(row.get(0)?),
                config_json: row.get(1)?,
                timestamp: row.get(2)?,
            }))
        } else {
            Ok(None)
        }
    }
    
    // ========================================================================
    // UTILITY OPERATIONS
    // ========================================================================
    
    /// Obtiene estadísticas del estado actual
    pub fn get_stats(&self) -> Result<StateStats> {
        let conn = self.conn.lock().unwrap();
        
        let active_positions: i64 = conn.query_row(
            "SELECT COUNT(*) FROM positions WHERE active = 1",
            [],
            |row| row.get(0),
        )?;
        
        let total_trades: i64 = conn.query_row(
            "SELECT COUNT(*) FROM trades",
            [],
            |row| row.get(0),
        )?;

        let mut stmt = conn.prepare(
            "SELECT COALESCE(SUM(pnl_sol), 0.0) FROM trades WHERE pnl_sol IS NOT NULL"
        )?;
        let total_pnl: f64 = stmt.query_row([], |row| row.get(0))?;
        
        Ok(StateStats {
            active_positions: active_positions as usize,
            total_trades: total_trades as usize,
            total_pnl_sol: total_pnl,
        })
    }
}

/// Estadísticas del estado
#[derive(Debug)]
pub struct StateStats {
    pub active_positions: usize,
    pub total_trades: usize,
    pub total_pnl_sol: f64,
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_state_manager_creation() {
        let manager = StateManager::new(":memory:").unwrap();
        let stats = manager.get_stats().unwrap();
        assert_eq!(stats.active_positions, 0);
        assert_eq!(stats.total_trades, 0);
    }
    
    #[test]
    fn test_position_lifecycle() {
        let manager = StateManager::new(":memory:").unwrap();
        
        let position = PositionState {
            id: None,
            token_mint: "TEST_MINT".to_string(),
            symbol: "TEST".to_string(),
            entry_price: 0.001,
            amount_sol: 1.0,
            current_price: 0.001,
            stop_loss_percent: -20.0,
            trailing_enabled: false,
            trailing_distance_percent: 5.0,
            trailing_activation_threshold: 10.0,
            trailing_highest_price: None,
            trailing_current_sl: None,
            tp_percent: None,
            tp_amount_percent: None,
            tp_triggered: false,
            active: true,
            created_at: Utc::now().timestamp(),
            updated_at: Utc::now().timestamp(),
        };
        
        manager.upsert_position(&position).unwrap();
        
        let retrieved = manager.get_position("TEST_MINT").unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().symbol, "TEST");
        
        manager.close_position("TEST_MINT").unwrap();
        
        let stats = manager.get_stats().unwrap();
        assert_eq!(stats.active_positions, 0);
    }
}
