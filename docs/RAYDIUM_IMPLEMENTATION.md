# 🚀 RAYDIUM DIRECT SWAP - Roadmap Técnico de Implementación

**Objetivo:** Eliminar la dependencia de APIs externas (Jupiter) para lograr **soberanía total** en la ejecución de swaps.

**Beneficios:**
- ⚡ **Latencia Ultra-baja:** No hay llamadas HTTP. Solo RPC → Blockchain.
- 🛡️ **Sin Puntos de Fallo:** Si Jupiter (la web) se cae, nuestro bot sigue operando.
- 🎯 **Precisión Quirúrgica:** Control total sobre slippage, cuentas y fees.

---

## 📋 Fases de Implementación

### Fase 1: Comprensión de la Arquitectura Raydium AMM v4

#### 1.1 Estructura de un Pool (Liquidity Pool)
Un pool de Raydium tiene estas cuentas principales:
- **AMM ID:** Identificador único del pool.
- **AMM Authority:** Cuenta PDA (Program Derived Address) con permisos sobre el pool.
- **AMM Open Orders:** Cuenta de órdenes en Serum (DEX subyacente).
- **Coin Vault:** Caja fuerte del Token A (e.g., SOL).
- **PC Vault:** Caja fuerte del Token B (e.g., USDC o memecoin).
- **LP Mint:** Mint de los tokens de liquidez (LP tokens).
- **Target Orders:** Cuentas de gestión de órdenes.

#### 1.2 Layout del Estado de la Cuenta AMM
El programa de Raydium almacena el estado del pool en un formato binario específico. Necesitamos:
1. **Leer la cuenta del AMM ID** usando `getAccountInfo` del RPC.
2. **Deserializar los bytes** según el layout oficial de Raydium.

**Referencia oficial:** [Raydium SDK (TypeScript)](https://github.com/raydium-io/raydium-sdk)

---

### Fase 2: Descubrimiento de Pools (Pool Discovery)

#### Opción A: RPC Filtering (getProgramAccounts)
```rust
// Buscar pools que contengan el mint de nuestro token
let filters = vec![
    RpcFilterType::Memcmp(Memcmp {
        offset: 400, // Offset del coinMint en la estructura
        bytes: MemcmpEncodedBytes::Base58(token_mint.to_string()),
        encoding: None,
    }),
];

let accounts = rpc_client.get_program_accounts_with_config(
    &raydium_program_id,
    RpcProgramAccountsConfig {
        filters: Some(filters),
        ..Default::default()
    },
)?;
```

**Problema:** `getProgramAccounts` es lento (1-3 segundos) y puede saturar RPCs públicos.

#### Opción B: Cache de Pools (Recomendado)
1. **Pre-cachear** los pools más comunes (SOL/USDC, SOL/USDT).
2. Para tokens nuevos (Pump.fun graduados), usar una **API estática de pools** (e.g., DexScreener API devuelve el pool ID).
3. Guardar en un archivo `pools_cache.json`.

**Ventaja:** Latencia casi cero. Solo consultamos el RPC una vez.

---

### Fase 3: Construcción de la Instrucción Swap

#### 3.1 Discriminator y Datos
Raydium usa un discriminador único para cada instrucción. Para `SwapBaseIn`:
- **Discriminator:** `0x09` (1 byte)
- **Amount In:** Cantidad de tokens a vender (8 bytes, u64, little-endian)
- **Min Amount Out:** Cantidad mínima a recibir (8 bytes, u64, little-endian)

```rust
let mut data = Vec::with_capacity(17);
data.push(9); // SwapBaseIn
data.extend_from_slice(&amount_in.to_le_bytes());
data.extend_from_slice(&min_amount_out.to_le_bytes());
```

#### 3.2 Cuentas Requeridas (Orden Estricto)
```rust
let accounts = vec![
    AccountMeta::new_readonly(spl_token::id(), false),               // 0. Token Program
    AccountMeta::new(amm_id, false),                                  // 1. AMM ID
    AccountMeta::new_readonly(amm_authority, false),                  // 2. AMM Authority
    AccountMeta::new(amm_open_orders, false),                         // 3. AMM Open Orders
    AccountMeta::new(pool_coin_token_account, false),                 // 4. Pool Coin Account
    AccountMeta::new(pool_pc_token_account, false),                   // 5. Pool PC Account
    AccountMeta::new_readonly(serum_program_id, false),               // 6. Serum Program
    AccountMeta::new(serum_market, false),                            // 7. Serum Market
    AccountMeta::new(serum_bids, false),                              // 8. Serum Bids
    AccountMeta::new(serum_asks, false),                              // 9. Serum Asks
    AccountMeta::new(serum_event_queue, false),                       // 10. Serum Event Queue
    AccountMeta::new(serum_coin_vault_account, false),                // 11. Serum Coin Vault
    AccountMeta::new(serum_pc_vault_account, false),                  // 12. Serum PC Vault
    AccountMeta::new_readonly(serum_vault_signer, false),             // 13. Serum Vault Signer
    AccountMeta::new(user_source_token_account, false),               // 14. User Source Account
    AccountMeta::new(user_destination_token_account, false),          // 15. User Dest Account
    AccountMeta::new_readonly(user_owner.pubkey(), true),             // 16. User Owner (Signer)
];
```

**Nota Crítica:** El orden de las cuentas es **estricto**. Un error aquí causa un fallo de transacción.

---

### Fase 4: Cálculo de Slippage y Min Amount Out

```rust
// Fórmula simplificada del AMM (x * y = k)
// Precio = reserve_pc / reserve_coin
let price = pool_pc_amount as f64 / pool_coin_amount as f64;
let expected_out = (amount_in as f64) * price;

// Aplicar slippage (e.g., 1%)
let slippage = 0.01;
let min_amount_out = (expected_out * (1.0 - slippage)) as u64;
```

**Mejora Avanzada:** Leer las **reservas actuales** del pool en tiempo real desde la cuenta del AMM.

---

### Fase 5: Ejecución y Firmado

```rust
let ix = Instruction {
    program_id: raydium_program_id,
    accounts,
    data,
};

let recent_blockhash = rpc_client.get_latest_blockhash()?;
let tx = Transaction::new_signed_with_payer(
    &[ix],
    Some(&user_keypair.pubkey()),
    &[&user_keypair],
    recent_blockhash,
);

let signature = rpc_client.send_and_confirm_transaction(&tx)?;
println!("✅ Swap ejecutado: {}", signature);
```

---

## 🛠️ Implementación Incremental (Próxima Sesión)

### Sprint 1: Pool Discovery (2 horas)
- [ ] Implementar `find_pool_by_mints()` usando `getProgramAccounts`.
- [ ] Cachear pools comunes en `pools_cache.json`.

### Sprint 2: Deserialización del Estado AMM (3 horas)
- [ ] Crear struct `AmmInfo` que mapea el layout de la cuenta.
- [ ] Implementar `deserialize_amm_account()`.
- [ ] Testear con un pool conocido (SOL/USDC).

### Sprint 3: Construcción de Swap (2 horas)
- [ ] Implementar `build_swap_instruction()`.
- [ ] Calcular `min_amount_out` con slippage configurable.
- [ ] Validar orden de cuentas.

### Sprint 4: Testing y Validación (2 horas)
- [ ] Ejecutar swap en **Devnet** primero.
- [ ] Validar en Mainnet con cantidad mínima (0.001 SOL).
- [ ] Comparar resultado con Jupiter (precio y fees).

---

## 📚 Recursos Técnicos

1.  **Raydium SDK (TypeScript):** [GitHub](https://github.com/raydium-io/raydium-sdk)
    - Estudiar `liquidity.ts` y `route.ts`.
2.  **Anchor Program IDL:** [Raydium AMM v4 IDL](https://github.com/raydium-io/raydium-contract-instructions)
    - Ver el layout exacto de las instrucciones.
3.  **Solana Program Library (SPL):** [Docs](https://spl.solana.com/)
    - Cómo gestionar cuentas de tokens asociadas.

---

## ⚠️ Puntos Críticos de Atención

1.  **Orden de Cuentas:** Un error aquí = transacción fallida.
2.  **Wrapped SOL (WSOL):** Cuando swapeas SOL nativo, necesitas crear una cuenta temporal de WSOL.
3.  **Fees de Serum:** Raydium usa Serum bajo el capó. Las cuentas de fee deben estar correctas.
4.  **Testing Exhaustivo:** Primero en Devnet, luego cantidades mínimas en Mainnet.

---

## 🎯 Criterio de Éxito

El módulo `raydium.rs` estará completo cuando:
- ✅ Podamos ejecutar un swap SOL → Memecoin sin llamar a Jupiter.
- ✅ La latencia total sea <500ms (RPC + construcción + envío).
- ✅ El slippage real sea ≤ slippage configurado +0.5%.
- ✅ El código no tenga `unwrap()` (manejo de errores completo).

---

**Estado Actual:** Esqueleto creado (`src/raydium.rs`).  
**Próximo Paso:** Pool Discovery + Deserialización.  
**Estimación Total:** 8-10 horas de ingeniería profunda.

**Filosofía:** "El que controla el pool, controla el juego." 🏎️
