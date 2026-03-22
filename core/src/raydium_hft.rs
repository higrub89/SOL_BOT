use bytemuck::{Pod, Zeroable};
use std::fmt;

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

/// Mapeo de bytes C-Rep del Estado del Pool de Raydium V4
/// Alineación perfecta (752 bytes). Cero heaps, cero json.
#[repr(C)]
#[derive(Clone, Copy, Zeroable, Pod, Debug)]
pub struct RaydiumAmmV4State {
    pub padding_0_15: [u8; 16],     // 0..16
    pub amm_authority: [u8; 32],    // 16..48
    pub amm_open_orders: [u8; 32],  // 48..80
    pub padding_80_175: [u8; 96],   // 80..176
    pub serum_market: [u8; 32],     // 176..208
    pub padding_208_367: [u64; 20], // 208..368
    pub lp_mint: [u8; 32],          // 368..400
    pub base_mint: [u8; 32],        // 400..432
    pub quote_mint: [u8; 32],       // 432..464
    pub base_vault: [u8; 32],       // 464..496
    pub quote_vault: [u8; 32],      // 496..528
    pub padding_528_751: [u64; 28], // 528..752
}

impl RaydiumAmmV4State {
    /// O(1) parser usando zero-allocation casting.
    #[inline(always)]
    pub fn parse(data: &[u8]) -> Result<&Self, HftError> {
        if data.len() != 752 {
            return Err(HftError::InvalidDataSize(data.len()));
        }
        // bytemuck chequea el alignment y cast de forma hipersegura.
        // Si incoming data es U8, siempre cumple el casting a raw bytes layout.
        Ok(bytemuck::from_bytes(data))
    }

    #[inline(always)]
    pub fn base_need_take_pnl(&self) -> u64 {
        // En Raydium v4, base_need_take_pnl es u64 en offset 256.
        // Array empieza en 208 -> (256 - 208) / 8 = index 6
        self.padding_208_367[6]
    }

    #[inline(always)]
    pub fn quote_need_take_pnl(&self) -> u64 {
        // quote_need_take_pnl es u64 en offset 264.
        self.padding_208_367[7]
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

        // Simular base_need_take_pnl = 42 (Offset 256)
        let val_bytes = 42u64.to_le_bytes();
        mock_data[256..264].copy_from_slice(&val_bytes);

        // Parse O(1)
        let state = RaydiumAmmV4State::parse(&mock_data).unwrap();

        assert_eq!(state.base_need_take_pnl(), 42);
        assert_eq!(state.quote_need_take_pnl(), 0);
    }

    #[test]
    fn test_swap_math() {
        let reserve_in = 1_000_000;
        let reserve_out = 2_000_000;
        let amount_in = 10_000;

        // Esperado aprox: 10000 * 0.9975 = 9975. ratio 2:1 -> 19950 out.
        let out = RaydiumAmmV4State::calculate_swap_amount_out(amount_in, reserve_in, reserve_out)
            .unwrap();

        // Exact formula checks
        let exact = (10_000u128 * 9975 * 2_000_000) / (1_000_000 * 10000 + 10_000 * 9975);
        assert_eq!(out, exact as u64);
    }

    #[test]
    fn test_invalid_payload_rejected() {
        let bad_data = vec![0u8; 751]; // Falta un byte
        assert_eq!(
            RaydiumAmmV4State::parse(&bad_data).unwrap_err(),
            HftError::InvalidDataSize(751)
        );
    }
}
