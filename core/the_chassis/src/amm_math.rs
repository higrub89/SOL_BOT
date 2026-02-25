//! # AMM Math — Cálculo de Precio desde Reservas On-Chain
//!
//! Parsea el estado de los pools AMM (Raydium V4) directamente desde
//! los datos on-chain y calcula precios en tiempo real.
//!
//! ## Cómo funciona un AMM (Constant Product)
//! ```text
//!   Pool: { sol_reserve, token_reserve }
//!   Invariante: sol_reserve * token_reserve = K (constante)
//!   Precio:     1 token = sol_reserve / token_reserve  (en SOL)
//! ```
//!
//! ## Estrategia de datos
//!
//! En Raydium V4, las reserves reales están en SPL Token Accounts separadas
//! (coin_vault y pc_vault). Pero para calcular precio podemos:
//!
//! **Opción A (esta implementación):** Suscribirnos a las vault accounts via Geyser.
//! Cuando un vault cambia → su `amount` field refleja la nueva reserve.
//!
//! **Opción B:** Parsear el AMM state (752+ bytes) que tiene reserves parciales.
//! Menos fiable porque las reserves on-chain del AMM incluyen fees acumulados.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

// ═══════════════════════════════════════════════════════════════════
// RAYDIUM V4 AMM STATE LAYOUT
// ═══════════════════════════════════════════════════════════════════

/// Offsets conocidos del layout de Raydium AMM V4.
/// Referencia: https://github.com/raydium-io/raydium-amm
pub mod raydium_v4_layout {
    // Header
    pub const STATUS: usize = 0; // u64 (8 bytes)
    pub const NONCE: usize = 8; // u64
    pub const MAX_ORDER: usize = 16; // u64
    pub const DEPTH: usize = 24; // u64
    pub const BASE_DECIMAL: usize = 32; // u64
    pub const QUOTE_DECIMAL: usize = 40; // u64
    pub const STATE: usize = 48; // u64
    pub const RESET_FLAG: usize = 56; // u64
    pub const MIN_SIZE: usize = 64; // u64
    pub const VOL_MAX_CUT_RATIO: usize = 72; // u64
    pub const AMOUNT_WAVE_RATIO: usize = 80; // u64

    // Fees
    pub const BASE_LOT_SIZE: usize = 88; // u64
    pub const QUOTE_LOT_SIZE: usize = 96; // u64
    pub const MIN_PRICE_MULTIPLIER: usize = 104; // u64
    pub const MAX_PRICE_MULTIPLIER: usize = 112; // u64

    // Key accounts (Pubkeys = 32 bytes each)
    pub const SYSTEM_DECIMAL_VALUE: usize = 120; // u64
    pub const MIN_SEPARATE_NUMERATOR: usize = 128; // u64
    pub const MIN_SEPARATE_DENOMINATOR: usize = 136; // u64
    pub const TRADE_FEE_NUMERATOR: usize = 144; // u64
    pub const TRADE_FEE_DENOMINATOR: usize = 152; // u64
    pub const PNL_NUMERATOR: usize = 160; // u64
    pub const PNL_DENOMINATOR: usize = 168; // u64
    pub const SWAP_FEE_NUMERATOR: usize = 176; // u64
    pub const SWAP_FEE_DENOMINATOR: usize = 184; // u64

    // Need/Pnl amounts
    pub const BASE_NEED_TAKE_PNL: usize = 192; // u64
    pub const QUOTE_NEED_TAKE_PNL: usize = 200; // u64
    pub const QUOTE_TOTAL_PNL: usize = 208; // u64
    pub const BASE_TOTAL_PNL: usize = 216; // u64
                                           // System info (u128 = 16 bytes each)
    pub const QUOTE_TOTAL_DEPOSITED: usize = 224; // u128
    pub const BASE_TOTAL_DEPOSITED: usize = 240; // u128
    pub const SWAP_BASE_IN_AMOUNT: usize = 256; // u128
    pub const SWAP_QUOTE_OUT_AMOUNT: usize = 272; // u128

    pub const SWAP_BASE2_QUOTE_FEE: usize = 288; // u64
    pub const SWAP_QUOTE_IN_AMOUNT: usize = 296; // u128
    pub const SWAP_BASE_OUT_AMOUNT: usize = 312; // u128
    pub const SWAP_QUOTE2_BASE_FEE: usize = 328; // u64

    // Pool vault reserves (what we need!)
    pub const POOL_COIN_AMOUNT: usize = 336; // u64 - Base token reserve
    pub const POOL_PC_AMOUNT: usize = 344; // u64 - Quote token reserve

    // Pubkeys
    pub const COIN_MINT: usize = 400; // Pubkey (32 bytes)
    pub const PC_MINT: usize = 432; // Pubkey
    pub const COIN_VAULT: usize = 464; // Pubkey
    pub const PC_VAULT: usize = 496; // Pubkey

    pub const MIN_DATA_LEN: usize = 528; // Mínimo para leer lo esencial
}

// ═══════════════════════════════════════════════════════════════════
// SPL TOKEN ACCOUNT PARSING
// ═══════════════════════════════════════════════════════════════════

/// Parsea el `amount` de un SPL Token Account desde los bytes crudos.
/// Layout: mint(32) + owner(32) + amount(8) + ...
/// Total mínimo: 165 bytes
pub fn parse_spl_token_account_amount(data: &[u8]) -> Option<u64> {
    if data.len() < 165 {
        return None;
    }
    // amount está en offset 64 (después de mint[32] + owner[32])
    data.get(64..72)
        .map(|slice| u64::from_le_bytes(slice.try_into().unwrap()))
}

/// Extrae el mint address de un SPL Token Account
pub fn parse_spl_token_account_mint(data: &[u8]) -> Option<[u8; 32]> {
    if data.len() < 165 {
        return None;
    }
    let mut mint = [0u8; 32];
    mint.copy_from_slice(&data[0..32]);
    Some(mint)
}

// ═══════════════════════════════════════════════════════════════════
// RAYDIUM AMM STATE PARSING
// ═══════════════════════════════════════════════════════════════════

/// Estado parseado de un pool Raydium AMM V4
#[derive(Debug, Clone)]
pub struct RaydiumPoolState {
    /// Decimales del token base (coin)
    pub base_decimals: u8,
    /// Decimales del token quote (pc, normalmente SOL=9 o USDC=6)
    pub quote_decimals: u8,
    /// Reserva del token base en el pool (raw, sin ajustar decimales)
    pub base_reserve_raw: u64,
    /// Reserva del token quote en el pool (raw)  
    pub quote_reserve_raw: u64,
    /// Coin mint (base)
    pub coin_mint: [u8; 32],
    /// PC mint (quote)
    pub pc_mint: [u8; 32],
    /// Coin vault pubkey
    pub coin_vault: [u8; 32],
    /// PC vault pubkey
    pub pc_vault: [u8; 32],
    /// Trade fee numerator
    pub trade_fee_numerator: u64,
    /// Trade fee denominator
    pub trade_fee_denominator: u64,
}

impl RaydiumPoolState {
    /// Parsea el estado de un pool desde los raw bytes de la cuenta AMM
    pub fn from_account_data(data: &[u8]) -> Option<Self> {
        use raydium_v4_layout as layout;

        if data.len() < layout::MIN_DATA_LEN {
            return None;
        }

        let read_u64 = |offset: usize| -> u64 {
            u64::from_le_bytes(data[offset..offset + 8].try_into().unwrap())
        };

        let read_pubkey = |offset: usize| -> [u8; 32] {
            let mut key = [0u8; 32];
            key.copy_from_slice(&data[offset..offset + 32]);
            key
        };

        let base_decimals = read_u64(layout::BASE_DECIMAL) as u8;
        let quote_decimals = read_u64(layout::QUOTE_DECIMAL) as u8;
        let base_reserve_raw = read_u64(layout::POOL_COIN_AMOUNT);
        let quote_reserve_raw = read_u64(layout::POOL_PC_AMOUNT);

        // Sanity checks
        if base_decimals > 18 || quote_decimals > 18 {
            return None;
        }
        if base_reserve_raw == 0 || quote_reserve_raw == 0 {
            return None; // Pool vacío o inválido
        }

        Some(Self {
            base_decimals,
            quote_decimals,
            base_reserve_raw,
            quote_reserve_raw,
            coin_mint: read_pubkey(layout::COIN_MINT),
            pc_mint: read_pubkey(layout::PC_MINT),
            coin_vault: read_pubkey(layout::COIN_VAULT),
            pc_vault: read_pubkey(layout::PC_VAULT),
            trade_fee_numerator: read_u64(layout::TRADE_FEE_NUMERATOR),
            trade_fee_denominator: read_u64(layout::TRADE_FEE_DENOMINATOR),
        })
    }

    /// Reserva base ajustada con decimales (valor humano)
    pub fn base_reserve(&self) -> f64 {
        self.base_reserve_raw as f64 / 10f64.powi(self.base_decimals as i32)
    }

    /// Reserva quote ajustada con decimales
    pub fn quote_reserve(&self) -> f64 {
        self.quote_reserve_raw as f64 / 10f64.powi(self.quote_decimals as i32)
    }

    /// Precio de 1 unidad de base token en quote token
    /// Ejemplo: si base=ICEBEAR, quote=SOL → devuelve precio en SOL
    pub fn price_in_quote(&self) -> f64 {
        let base = self.base_reserve();
        let quote = self.quote_reserve();

        if base == 0.0 {
            return 0.0;
        }

        quote / base
    }

    /// Liquidez total del pool en quote token (ambos lados)
    /// TVL ≈ 2 * quote_reserve (asumiendo precio equilibrado)
    pub fn liquidity_in_quote(&self) -> f64 {
        self.quote_reserve() * 2.0
    }

    /// Constante K del pool (x * y = K)
    pub fn constant_product(&self) -> f64 {
        self.base_reserve() * self.quote_reserve()
    }

    /// Calcula el impacto en precio para un swap de `amount_in` base tokens
    pub fn price_impact_for_sell(&self, amount_in_base: f64) -> f64 {
        let base = self.base_reserve();
        let quote = self.quote_reserve();

        if base == 0.0 || amount_in_base == 0.0 {
            return 0.0;
        }

        let price_before = quote / base;
        let new_base = base + amount_in_base;
        let new_quote = (base * quote) / new_base; // K = constant
        let price_after = new_quote / new_base;

        ((price_after - price_before) / price_before) * 100.0
    }
}

// ═══════════════════════════════════════════════════════════════════
// VAULT PAIR TRACKER — Monitoreo de reserves via Vault Accounts
// ═══════════════════════════════════════════════════════════════════

/// Información sobre un par de vaults que forman un pool
#[derive(Debug, Clone)]
pub struct VaultPair {
    /// Mint del token (la "base" que nos interesa)
    pub token_mint: String,
    /// Símbolo humano
    pub symbol: String,
    /// Address del coin vault (base token)
    pub coin_vault: String,
    /// Address del pc vault (quote, ej: WSOL)
    pub pc_vault: String,
    /// Decimales del base token
    pub base_decimals: u8,
    /// Decimales del quote token (SOL = 9)
    pub quote_decimals: u8,
    /// Última reserve conocida del coin vault
    pub last_coin_reserve: Option<u64>,
    /// Última reserve conocida del pc vault
    pub last_pc_reserve: Option<u64>,
}

impl VaultPair {
    /// ¿Tenemos ambas reserves para calcular precio?
    pub fn is_ready(&self) -> bool {
        self.last_coin_reserve.is_some() && self.last_pc_reserve.is_some()
    }

    /// Actualiza una de las reserves según la vault address que cambió.
    /// Retorna true si la reserve fue actualizada.
    pub fn update_reserve(&mut self, vault_address: &str, amount: u64) -> bool {
        if vault_address == self.coin_vault {
            self.last_coin_reserve = Some(amount);
            true
        } else if vault_address == self.pc_vault {
            self.last_pc_reserve = Some(amount);
            true
        } else {
            false
        }
    }

    /// Calcula el precio en quote (ej: SOL) si ambas reserves están disponibles
    pub fn calculate_price_in_quote(&self) -> Option<f64> {
        let coin = self.last_coin_reserve? as f64 / 10f64.powi(self.base_decimals as i32);
        let pc = self.last_pc_reserve? as f64 / 10f64.powi(self.quote_decimals as i32);

        if coin == 0.0 {
            return None;
        }

        Some(pc / coin)
    }

    /// Calcula la liquidez total en quote (≈ 2 * pc_reserve)
    pub fn calculate_liquidity_in_quote(&self) -> Option<f64> {
        let pc = self.last_pc_reserve? as f64 / 10f64.powi(self.quote_decimals as i32);
        Some(pc * 2.0)
    }
}

/// Tracker thread-safe para múltiples vault pairs
pub type VaultTracker = Arc<RwLock<HashMap<String, VaultPair>>>;

/// Crea un VaultTracker indexado por vault address (para lookup O(1) en Geyser)
pub fn build_vault_tracker(pairs: Vec<VaultPair>) -> (VaultTracker, HashMap<String, String>) {
    let mut tracker_map = HashMap::new();
    // vault_address → token_mint (para saber a qué par pertenece una vault)
    let mut vault_to_mint = HashMap::new();

    for pair in pairs {
        vault_to_mint.insert(pair.coin_vault.clone(), pair.token_mint.clone());
        vault_to_mint.insert(pair.pc_vault.clone(), pair.token_mint.clone());
        tracker_map.insert(pair.token_mint.clone(), pair);
    }

    (Arc::new(RwLock::new(tracker_map)), vault_to_mint)
}

// ═══════════════════════════════════════════════════════════════════
// PRICE CONVERSION
// ═══════════════════════════════════════════════════════════════════

/// Precio de SOL en USD (se actualiza desde DexScreener)
/// Thread-safe para compartir entre tasks.
pub type SolPriceUsd = Arc<RwLock<f64>>;

/// Crea un tracker del precio de SOL inicializado en 0
pub fn new_sol_price_tracker() -> SolPriceUsd {
    Arc::new(RwLock::new(0.0))
}

/// Convierte un precio en SOL a USD
pub async fn sol_to_usd(price_in_sol: f64, sol_price: &SolPriceUsd) -> f64 {
    let sol_usd = *sol_price.read().await;
    if sol_usd == 0.0 {
        return 0.0; // No tenemos precio de SOL todavía
    }
    price_in_sol * sol_usd
}

// ═══════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_price_calculation() {
        let mut pair = VaultPair {
            token_mint: "TEST".to_string(),
            symbol: "TEST".to_string(),
            coin_vault: "vault_coin".to_string(),
            pc_vault: "vault_pc".to_string(),
            base_decimals: 6,
            quote_decimals: 9,
            last_coin_reserve: None,
            last_pc_reserve: None,
        };

        assert!(!pair.is_ready());
        assert!(pair.calculate_price_in_quote().is_none());

        // Simular: 1,000,000 tokens (6 dec), 10 SOL (9 dec)
        pair.update_reserve("vault_coin", 1_000_000_000_000); // 1M tokens
        pair.update_reserve("vault_pc", 10_000_000_000); // 10 SOL

        assert!(pair.is_ready());

        let price = pair.calculate_price_in_quote().unwrap();
        // price = 10 SOL / 1M tokens = 0.00001 SOL per token
        assert!((price - 0.00001).abs() < 0.0000001);
    }

    #[test]
    fn test_price_impact() {
        let state = RaydiumPoolState {
            base_decimals: 6,
            quote_decimals: 9,
            base_reserve_raw: 1_000_000_000_000, // 1M tokens
            quote_reserve_raw: 10_000_000_000,   // 10 SOL
            coin_mint: [0; 32],
            pc_mint: [0; 32],
            coin_vault: [0; 32],
            pc_vault: [0; 32],
            trade_fee_numerator: 25,
            trade_fee_denominator: 10000,
        };

        // Vender 1% del pool (10,000 tokens)
        let impact = state.price_impact_for_sell(10_000.0);
        // Debería ser aprox -1% (el precio baja cuando inyectas más tokens)
        assert!(impact < 0.0);
        assert!(impact > -2.0);
    }

    #[test]
    fn test_spl_token_parsing() {
        // Crear un fake SPL Token Account de 165 bytes
        let mut data = vec![0u8; 165];
        // Escribir amount = 42 en offset 64
        let amount: u64 = 42;
        data[64..72].copy_from_slice(&amount.to_le_bytes());

        assert_eq!(parse_spl_token_account_amount(&data), Some(42));
    }

    #[test]
    fn test_vault_tracker_lookup() {
        let pairs = vec![VaultPair {
            token_mint: "TOKEN_A".to_string(),
            symbol: "A".to_string(),
            coin_vault: "vault_a_coin".to_string(),
            pc_vault: "vault_a_pc".to_string(),
            base_decimals: 6,
            quote_decimals: 9,
            last_coin_reserve: None,
            last_pc_reserve: None,
        }];

        let (_tracker, vault_to_mint) = build_vault_tracker(pairs);

        assert_eq!(
            vault_to_mint.get("vault_a_coin"),
            Some(&"TOKEN_A".to_string())
        );
        assert_eq!(
            vault_to_mint.get("vault_a_pc"),
            Some(&"TOKEN_A".to_string())
        );
        assert_eq!(vault_to_mint.get("unknown"), None);
    }
}
