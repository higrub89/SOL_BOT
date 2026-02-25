//! # Financial Data Validation
//!
//! Validación estricta de datos financieros para prevenir decisiones erróneas
//! causadas por APIs que devuelven datos corruptos o inesperados.

use anyhow::{Context, Result};

/// Validador de datos financieros
pub struct FinancialValidator;

impl FinancialValidator {
    /// Valida que un precio sea razonable y no corrupto
    ///
    /// # Errores
    /// - Precio <= 0
    /// - Precio NaN o Infinito
    /// - Precio absurdamente alto (posible error de API)
    pub fn validate_price(price: f64, context: &str) -> Result<f64> {
        if price <= 0.0 {
            anyhow::bail!("{}: Precio inválido o cero ({})", context, price);
        }

        if price.is_nan() {
            anyhow::bail!("{}: Precio es NaN (Not a Number)", context);
        }

        if price.is_infinite() {
            anyhow::bail!("{}: Precio es infinito", context);
        }

        // Detectar precios absurdos (probablemente error de API)
        // Un token de $1B es extremadamente raro
        if price > 1_000_000_000.0 {
            anyhow::bail!(
                "{}: Precio sospechosamente alto (${:.2}). Posible error de API.",
                context,
                price
            );
        }

        Ok(price)
    }

    /// Valida que un cambio de precio sea razonable (anti-glitch)
    ///
    /// Protege contra "price glitches" donde una API devuelve un precio
    /// temporalmente incorrecto que podría causar ventas de pánico.
    ///
    /// # Argumentos
    /// - `old_price`: Precio anterior conocido
    /// - `new_price`: Nuevo precio a validar
    /// - `max_change_percent`: Máximo cambio permitido (ej: 50.0 = 50%)
    pub fn validate_price_change(
        old_price: f64,
        new_price: f64,
        max_change_percent: f64,
        context: &str,
    ) -> Result<f64> {
        // Primero validar que ambos precios sean válidos
        Self::validate_price(old_price, &format!("{} (old)", context))?;
        Self::validate_price(new_price, &format!("{} (new)", context))?;

        let change_pct = ((new_price - old_price) / old_price).abs() * 100.0;

        if change_pct > max_change_percent {
            anyhow::bail!(
                "{}: Cambio de precio sospechoso: {:.2}% (límite: {:.2}%). \
                 Old: ${:.8}, New: ${:.8}. Posible glitch de API.",
                context,
                change_pct,
                max_change_percent,
                old_price,
                new_price
            );
        }

        Ok(new_price)
    }

    /// Valida un monto de tokens (debe ser > 0)
    pub fn validate_amount(amount: u64, context: &str) -> Result<u64> {
        if amount == 0 {
            anyhow::bail!("{}: Cantidad de tokens es cero", context);
        }

        Ok(amount)
    }

    /// Valida un monto en SOL (debe ser > 0)
    pub fn validate_sol_amount(amount: f64, context: &str) -> Result<f64> {
        if amount <= 0.0 {
            anyhow::bail!("{}: Cantidad de SOL inválida ({})", context, amount);
        }

        if amount.is_nan() || amount.is_infinite() {
            anyhow::bail!("{}: Cantidad de SOL no numérica", context);
        }

        Ok(amount)
    }

    /// Valida liquidez mínima (protege contra pools con liquidez muy baja)
    pub fn validate_liquidity(
        liquidity_usd: f64,
        min_liquidity: f64,
        context: &str,
    ) -> Result<f64> {
        if liquidity_usd < 0.0 {
            anyhow::bail!("{}: Liquidez negativa ({})", context, liquidity_usd);
        }

        if liquidity_usd < min_liquidity {
            anyhow::bail!(
                "{}: Liquidez demasiado baja (${:.2} < ${:.2}). Alto riesgo de slippage.",
                context,
                liquidity_usd,
                min_liquidity
            );
        }

        Ok(liquidity_usd)
    }

    /// Valida price impact (debe estar dentro de límites razonables)
    pub fn validate_price_impact(
        price_impact_pct: f64,
        max_impact: f64,
        context: &str,
    ) -> Result<f64> {
        if price_impact_pct < 0.0 {
            anyhow::bail!("{}: Price impact negativo ({}%)", context, price_impact_pct);
        }

        if price_impact_pct > max_impact {
            anyhow::bail!(
                "{}: Price impact demasiado alto ({:.2}% > {:.2}%). \
                 La operación causaría pérdidas significativas por slippage.",
                context,
                price_impact_pct,
                max_impact
            );
        }

        Ok(price_impact_pct)
    }

    /// Parsea un string a f64 con validación (Permisivo para tokens nuevos)
    pub fn parse_price_safe(price_str: &str, context: &str) -> Result<f64> {
        let price = price_str.parse::<f64>().unwrap_or(0.0);

        if price <= 0.0 {
            // No fallar, solo advertir. Es común en tokens de segundos de vida.
            return Ok(0.0);
        }

        Self::validate_price(price, context)
    }

    /// Parsea un string a u64 con validación
    pub fn parse_amount_safe(amount_str: &str, context: &str) -> Result<u64> {
        let amount = amount_str.parse::<u64>().with_context(|| {
            format!("{}: No se pudo parsear cantidad '{}'", context, amount_str)
        })?;

        Self::validate_amount(amount, context)
    }

    /// Valida que un mint sea un address de Solana válido
    ///
    /// Verifica:
    /// - Longitud correcta (43-44 caracteres en base58)
    /// - Caracteres válidos de base58
    /// - No contiene espacios ni caracteres especiales
    /// - No es un mint WSOL (para compras)
    pub fn validate_mint(mint: &str, context: &str) -> Result<String> {
        Self::validate_mint_internal(mint, context, false)
    }

    /// Valida un mint, con la opción de permitir WSOL internamente
    fn validate_mint_internal(mint: &str, context: &str, allow_wsol: bool) -> Result<String> {
        let mint = mint.trim();

        // Validar que no esté vacío
        if mint.is_empty() {
            anyhow::bail!("{}: Mint está vacío", context);
        }

        // Validar longitud típica de mints de Solana (43-44 caracteres en base58)
        if mint.len() < 43 || mint.len() > 44 {
            anyhow::bail!(
                "{}: Mint tiene longitud inválida ({} caracteres). \
                 Los mints de Solana tienen 43-44 caracteres. Mint: {}",
                context,
                mint.len(),
                mint
            );
        }

        // Validar caracteres base58 válidos (no contiene '0', 'O', 'I', 'l')
        // Los caracteres válidos son: 123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz
        let valid_chars = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";
        for c in mint.chars() {
            if !valid_chars.contains(c) {
                anyhow::bail!(
                    "{}: Mint contiene caracteres inválidos para base58 '{}'. \
                     Mint: {}. Caracteres válidos: {}",
                    context,
                    c,
                    mint,
                    valid_chars
                );
            }
        }

        // Validar que no sea el WSOL mint (si no está explícitamente permitido)
        const WSOL_MINT: &str = "So11111111111111111111111111111111111111112";
        if !allow_wsol && mint == WSOL_MINT {
            anyhow::bail!(
                "{}: No puedes comprar WSOL (wrapped SOL nativo). \
                 Usa el SOL nativo directamente.",
                context
            );
        }

        Ok(mint.to_string())
    }

    /// Valida un par de mints para swaps (input y output no pueden ser iguales)
    pub fn validate_mint_pair(input_mint: &str, output_mint: &str, context: &str) -> Result<()> {
        // Permitimos WSOL en los pares internos porque Jupiter usa WSOL para rutar desde SOL nativo
        Self::validate_mint_internal(input_mint, &format!("{} (input)", context), true)?;
        Self::validate_mint_internal(output_mint, &format!("{} (output)", context), true)?;

        if input_mint == output_mint {
            anyhow::bail!(
                "{}: Input y output mints no pueden ser iguales: {}",
                context,
                input_mint
            );
        }

        Ok(())
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_price_valid() {
        assert!(FinancialValidator::validate_price(0.001, "test").is_ok());
        assert!(FinancialValidator::validate_price(100.0, "test").is_ok());
        assert!(FinancialValidator::validate_price(1_000_000.0, "test").is_ok());
    }

    #[test]
    fn test_validate_price_invalid() {
        assert!(FinancialValidator::validate_price(0.0, "test").is_err());
        assert!(FinancialValidator::validate_price(-1.0, "test").is_err());
        assert!(FinancialValidator::validate_price(f64::NAN, "test").is_err());
        assert!(FinancialValidator::validate_price(f64::INFINITY, "test").is_err());
        assert!(FinancialValidator::validate_price(2_000_000_000.0, "test").is_err());
    }

    #[test]
    fn test_validate_price_change() {
        // Cambio normal (10%)
        assert!(FinancialValidator::validate_price_change(0.001, 0.0011, 50.0, "test").is_ok());

        // Cambio grande pero dentro del límite (40%)
        assert!(FinancialValidator::validate_price_change(0.001, 0.0014, 50.0, "test").is_ok());

        // Cambio demasiado grande (100%)
        assert!(FinancialValidator::validate_price_change(0.001, 0.002, 50.0, "test").is_err());

        // Caída demasiado grande (-60%)
        assert!(FinancialValidator::validate_price_change(0.001, 0.0004, 50.0, "test").is_err());
    }

    #[test]
    fn test_validate_amount() {
        assert!(FinancialValidator::validate_amount(1000, "test").is_ok());
        assert!(FinancialValidator::validate_amount(0, "test").is_err());
    }

    #[test]
    fn test_validate_liquidity() {
        assert!(FinancialValidator::validate_liquidity(10000.0, 1000.0, "test").is_ok());

        assert!(FinancialValidator::validate_liquidity(500.0, 1000.0, "test").is_err());

        assert!(FinancialValidator::validate_liquidity(-100.0, 1000.0, "test").is_err());
    }

    #[test]
    fn test_validate_price_impact() {
        assert!(FinancialValidator::validate_price_impact(1.5, 5.0, "test").is_ok());

        assert!(FinancialValidator::validate_price_impact(10.0, 5.0, "test").is_err());

        assert!(FinancialValidator::validate_price_impact(-1.0, 5.0, "test").is_err());
    }

    #[test]
    fn test_parse_price_safe() {
        assert_eq!(
            FinancialValidator::parse_price_safe("0.001", "test").unwrap(),
            0.001
        );

        assert_eq!(
            FinancialValidator::parse_price_safe("invalid", "test").unwrap(),
            0.0
        );
        assert_eq!(
            FinancialValidator::parse_price_safe("0", "test").unwrap(),
            0.0
        );
    }

    #[test]
    fn test_parse_amount_safe() {
        assert_eq!(
            FinancialValidator::parse_amount_safe("1000", "test").unwrap(),
            1000
        );

        assert!(FinancialValidator::parse_amount_safe("invalid", "test").is_err());
        assert!(FinancialValidator::parse_amount_safe("0", "test").is_err());
    }
}
