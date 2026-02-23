//! # Simple Jupiter Executor
//! 
//! Este módulo implementa la "Opción A": Ejecución asistida.
//! 
//! Cuando salta el SL:
//! 1. Consulta Jupiter API para el mejor precio (informativo).
//! 2. Genera una URL mágica de Jup.ag con el swap precargado.
//! 3. Abre tu navegador predeterminado para que confirmes la venta.

use anyhow::Result;
use crate::jupiter::JupiterClient;

pub struct SimpleExecutor {
    pub jupiter: JupiterClient,
}

impl Default for SimpleExecutor {
    fn default() -> Self {
        Self::new()
    }
}

impl SimpleExecutor {
    pub fn new() -> Self {
        Self {
            jupiter: JupiterClient::new(),
        }
    }

    /// Genera la URL de emergencia y abre el navegador
    pub async fn execute_emergency_sell_url(
        &self,
        token_mint: &str,
        _wallet_pubkey: &str, // Para referencia futura si queremos usar API directe
        amount_tokens: u64,  // Cantidad exacta en unidades atómicas (lamports)
        symbol: &str,
    ) -> Result<String> {
        println!("╔════════════════════════════════════════════════════════════╗");
        println!("║             🚀 SIMPLE EXECUTOR ACTIVADO 🚀               ║");
        println!("╚════════════════════════════════════════════════════════════╝\n");
        println!("🔍 Analizando mercado para vender {} {}...", amount_tokens, symbol);

        // 1. Obtener Quote Informativo (para que sepas a cuánto venderás)
        let sol_mint = "So11111111111111111111111111111111111111112";
        
        match self.jupiter.get_quote(token_mint, sol_mint, amount_tokens, 100).await {
            Ok(quote) => {
                println!("✅ Mejor precio encontrado en Jupiter:");
                self.jupiter.print_quote_summary(&quote);
            },
            Err(e) => {
                eprintln!("⚠️  No se pudo obtener quote en background: {}", e);
                eprintln!("   (Continuando con la generación de URL de todos modos...)");
            }
        }

        // 2. Construir la URL Mágica
        // Formato: https://jup.ag/swap/{INPUT_MINT}-{OUTPUT_MINT}?inAmount={AMOUNT}&slippage=1
        // Nota: Jupiter UI usa unidades "humanas" o "atómicas" dependiendo del endpoint,
        // pero la URL deep link suele ser más flexible. Vamos a probar con la estructura estándar.
        
        // Convertimos a string con decimales si es necesario, pero Jup prefiere el mint.
        // Jup URL format: https://jup.ag/swap/TOKEN_MINT-SOL
        // Podemos añadir ?inAmount=... pero requeriría saber los decimales exactos.
        // Para asegurar compatibilidad rápida, vamos a abrir el par directo.
        
        let url = format!(
            "https://jup.ag/swap/{}-SOL",
            token_mint
        );

        println!("\n🔗 URL Generada: {}", url);
        println!("⚠️  ACCIÓN REQUERIDA: Confirma la venta en el navegador.\n");

        // 3. Abrir Navegador
        // Intentamos abrir el navegador de forma cross-platform
        let open_result = webbrowser::open(&url);

        match open_result {
            Ok(_) => {
                println!("✅ Navegador abierto exitosamente.");
            },
            Err(e) => {
                eprintln!("❌ Error al abrir navegador: {}", e);
                eprintln!("👉 Copia y pega el enlace manualmente.");
            }
        }

        Ok(url)
    }
}
