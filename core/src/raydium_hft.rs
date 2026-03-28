use bytemuck::{Pod, Zeroable};
use std::fmt;

pub const RAYDIUM_V4_PROGRAM_ID: &str = "675kPX9MHTjS2tw1y8qyxokq1tKho2FpT1GEnbUX245R";

/// Errores deterministas de HFT para Raydium
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HftError {
    InvalidDataSize(usize),
    MathOverflow,
}

impl fmt::Display for HftError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HftError::InvalidDataSize(size) => {
                write!(f, "Invalid payload size: expected 752, got {}", size)
            }
            HftError::MathOverflow => {
                write!(f, "Critical math overflow during swap calculation")
            }
        }
    }
}
impl std::error::Error for HftError {}

/// Mapeo zero-copy del Estado del Pool de Raydium AMM V4.
/// Layout exacto de 752 bytes según el IDL oficial de Raydium.
/// Ref: https://github.com/raydium-io/raydium-amm/blob/master/program/src/state.rs
///
/// NOTA CRÍTICA: Los balances reales de reserva (coin/pc amount) NO están en este state.
/// Están en las cuentas vault separadas (`pool_coin_vault`, `pool_pc_vault`).
/// Este struct solo contiene metadatos, fees, y pubkeys de las cuentas asociadas.
///
/// Usamos arrays de u64 para evitar problemas de alineación con u128 y bytemuck.
#[repr(C)]
#[derive(Clone, Copy, Zeroable, Pod, Debug)]
pub struct RaydiumAmmV4State {
    // BLOQUE 0-192: 24 campos u64 de configuración
    pub config: [u64; 24], // 0..192

    // BLOQUE 192-320: PnL y métricas (16 campos u64)
    // Incluye need_take_pnl_coin (idx 0), need_take_pnl_pc (idx 1), etc.
    pub pnl_metrics: [u64; 16], // 192..320

    // BLOQUE 320-752: Pubkeys de cuentas (13 pubkeys de 32 bytes + 16 bytes finales)
    pub pool_coin_vault: [u8; 32],     // 320..352
    pub pool_pc_vault: [u8; 32],       // 352..384
    pub coin_mint: [u8; 32],           // 384..416
    pub pc_mint: [u8; 32],             // 416..448
    pub lp_mint: [u8; 32],             // 448..480
    pub amm_open_orders: [u8; 32],     // 480..512
    pub serum_market: [u8; 32],        // 512..544
    pub serum_program_id: [u8; 32],    // 544..576
    pub amm_target_orders: [u8; 32],   // 576..608
    pub pool_withdraw_queue: [u8; 32], // 608..640
    pub pool_temp_lp: [u8; 32],        // 640..672
    pub amm_owner: [u8; 32],           // 672..704
    pub pnl_owner: [u8; 32],           // 704..736
    pub srm_token_account: [u8; 16],   // 736..752
}

impl RaydiumAmmV4State {
    // ═══════════════════════════════════════════════════════════════════════════
    // ÍNDICES DE config[24] — Offsets 0..192
    // ═══════════════════════════════════════════════════════════════════════════
    const IDX_COIN_DECIMALS: usize = 4; // 32..40
    const IDX_PC_DECIMALS: usize = 5; // 40..48
    const IDX_SWAP_FEE_NUM: usize = 22; // 176..184
    const IDX_SWAP_FEE_DEN: usize = 23; // 184..192

    // ═══════════════════════════════════════════════════════════════════════════
    // ÍNDICES DE pnl_metrics[16] — Offsets 192..320
    // ═══════════════════════════════════════════════════════════════════════════
    const IDX_NEED_TAKE_PNL_COIN: usize = 0; // 192..200
    const IDX_NEED_TAKE_PNL_PC: usize = 1; // 200..208
    const IDX_POOL_OPEN_TIME: usize = 4; // 224..232

    /// O(1) parser usando zero-allocation casting.
    #[inline(always)]
    pub fn parse(data: &[u8]) -> Result<&Self, HftError> {
        if data.len() != 752 {
            return Err(HftError::InvalidDataSize(data.len()));
        }
        Ok(bytemuck::from_bytes(data))
    }

    /// PnL pendiente de retirar del token base (coin).
    /// Offset 192. Acceso directo O(1).
    #[inline(always)]
    pub fn base_need_take_pnl(&self) -> u64 {
        self.pnl_metrics[Self::IDX_NEED_TAKE_PNL_COIN]
    }

    /// PnL pendiente de retirar del token quote (PC).
    /// Offset 200. Acceso directo O(1).
    #[inline(always)]
    pub fn quote_need_take_pnl(&self) -> u64 {
        self.pnl_metrics[Self::IDX_NEED_TAKE_PNL_PC]
    }

    /// Pubkey del vault de coin (base token).
    /// NOTA: Este es el ADDRESS del vault, NO el balance.
    /// Para obtener el balance real, lee esta cuenta con getAccountInfo.
    #[inline(always)]
    pub fn coin_vault_pubkey(&self) -> [u8; 32] {
        self.pool_coin_vault
    }

    /// Pubkey del vault de PC (quote token).
    /// NOTA: Este es el ADDRESS del vault, NO el balance.
    #[inline(always)]
    pub fn pc_vault_pubkey(&self) -> [u8; 32] {
        self.pool_pc_vault
    }

    /// Mint address del token base (coin).
    #[inline(always)]
    pub fn base_mint_bytes(&self) -> [u8; 32] {
        self.coin_mint
    }

    /// Mint address del token quote (PC).
    #[inline(always)]
    pub fn quote_mint_bytes(&self) -> [u8; 32] {
        self.pc_mint
    }

    /// Decimales del token base.
    #[inline(always)]
    pub fn base_decimals(&self) -> u8 {
        self.config[Self::IDX_COIN_DECIMALS] as u8
    }

    /// Decimales del token quote.
    #[inline(always)]
    pub fn quote_decimals(&self) -> u8 {
        self.config[Self::IDX_PC_DECIMALS] as u8
    }

    /// Fee de swap expresado como fracción (numerador/denominador).
    /// Típicamente 25/10000 = 0.25%
    #[inline(always)]
    pub fn swap_fee(&self) -> (u64, u64) {
        (
            self.config[Self::IDX_SWAP_FEE_NUM],
            self.config[Self::IDX_SWAP_FEE_DEN],
        )
    }

    /// Timestamp Unix de apertura del pool.
    #[inline(always)]
    pub fn pool_open_time(&self) -> u64 {
        self.pnl_metrics[Self::IDX_POOL_OPEN_TIME]
    }

    /// O(1) Constant Product Swap Calculation
    /// `amount_in`: Tokens ingresados por el operador.
    /// `reserve_in`: Balance real en el vault de entrada menos pnl_take (vault - need_take_pnl).
    /// `reserve_out`: Balance real en el vault de salida menos pnl_take.
    ///
    /// Retorna `Result<amount_out, HftError>` (100% determinísta, no panics).
    #[inline(always)]
    pub fn calculate_swap_amount_out(
        amount_in: u64,
        reserve_in: u64,
        reserve_out: u64,
    ) -> Result<u64, HftError> {
        // Constant Product AMM (x * y = k) con % Fee
        let amount_in_u128 = amount_in as u128;
        let reserve_in_u128 = reserve_in as u128;
        let reserve_out_u128 = reserve_out as u128;

        // Multiplicador 9975 y divisor 10000 para cuota del 0.25% en Raydium
        let fee_multiplier: u128 = 9975;
        let fee_denominator: u128 = 10000;

        let amount_in_with_fee = amount_in_u128
            .checked_mul(fee_multiplier)
            .ok_or(HftError::MathOverflow)?;

        let numerator = amount_in_with_fee
            .checked_mul(reserve_out_u128)
            .ok_or(HftError::MathOverflow)?;

        let denominator_p1 = reserve_in_u128
            .checked_mul(fee_denominator)
            .ok_or(HftError::MathOverflow)?;

        let denominator = denominator_p1
            .checked_add(amount_in_with_fee)
            .ok_or(HftError::MathOverflow)?;

        let amount_out_u128 = numerator
            .checked_div(denominator)
            .ok_or(HftError::MathOverflow)?;

        // Downgrade seguro
        if amount_out_u128 > u64::MAX as u128 {
            return Err(HftError::MathOverflow);
        }

        Ok(amount_out_u128 as u64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zero_alloc_parsing() {
        let mut mock_data = vec![0u8; 752];

        // need_take_pnl_coin está en offset 192 (corregido del 256 erróneo)
        let val_bytes = 42u64.to_le_bytes();
        mock_data[192..200].copy_from_slice(&val_bytes);

        // need_take_pnl_pc en offset 200
        let val2_bytes = 100u64.to_le_bytes();
        mock_data[200..208].copy_from_slice(&val2_bytes);

        // coin_decimals en offset 32
        let decimals_bytes = 9u64.to_le_bytes();
        mock_data[32..40].copy_from_slice(&decimals_bytes);

        // Parse O(1)
        let state = RaydiumAmmV4State::parse(&mock_data).expect("parse debe funcionar");

        assert_eq!(state.base_need_take_pnl(), 42);
        assert_eq!(state.quote_need_take_pnl(), 100);
        assert_eq!(state.base_decimals(), 9);
    }

    #[test]
    fn test_struct_size() {
        // Verificar que el struct tiene exactamente 752 bytes
        assert_eq!(std::mem::size_of::<RaydiumAmmV4State>(), 752);
    }

    #[test]
    fn test_swap_math() {
        let reserve_in = 1_000_000;
        let reserve_out = 2_000_000;
        let amount_in = 10_000;

        // Esperado aprox: 10000 * 0.9975 = 9975. ratio 2:1 -> 19950 out.
        let out = RaydiumAmmV4State::calculate_swap_amount_out(amount_in, reserve_in, reserve_out)
            .expect("swap math no debe fallar");

        // Exact formula checks
        let exact = (10_000u128 * 9975 * 2_000_000) / (1_000_000 * 10000 + 10_000 * 9975);
        assert_eq!(out, exact as u64);
    }

    #[test]
    fn test_invalid_payload_rejected() {
        let bad_data = vec![0u8; 751]; // Falta un byte
        assert_eq!(
            RaydiumAmmV4State::parse(&bad_data).expect_err("debe rechazar payload inválido"),
            HftError::InvalidDataSize(751)
        );
    }
}
