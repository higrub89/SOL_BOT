# 🏁 PLAN DE ARQUITECTURA "ZERO-CONFIG" Y HFT (DÍA 2)

## 🎯 Objetivo Principal
Convertir SOL_BOT (The Chassis) en un sistema de trading de Alta Frecuencia (HFT) "grado institucional".
Eliminar dependencias de archivos estáticos (`targets.json`) y migrar el 100% de la gobernanza a memoria dinámica (SQLite) y Telegram. Reducir la latencia de ventas concurrentes a menos de 50ms.

---

## 🛠️ Tareas y Fases de Implementación

### FASE 1: Migración "Zero-Config" (Database-First)
- [ ] Eliminar la carga forzada de tokens desde `targets.json`. Renombrar el archivo a `settings.json` para dejar solo parámetros globales (auto_execute, jito_tip, etc.).
- [ ] Refactorizar `lib.rs` para que el "Unified Target Map" se alimente **exclusivamente** de SQLite (`trading_state.db`).
- [ ] Implementar los nuevos Comandos de Telegram (`teloxide`):
  - 🟢 `/track <MINT> <SOL> <SL> <TP>` (Mejorar sintaxis).
  - 🔴 `/untrack <MINT>` (Elimina instantáneamente de SQLite y para el monitor).
  - ⚙️ `/update <MINT> sl=X tp=Y` (Hot-swap de parámetros sin reiniciar).
  - 📋 `/positions` (Markdown table nativo en Telegram con PnL real-time).

### FASE 2: Concurrencia Extrema (HFT Multihilo)
- [ ] Desacoplar el `while let Some(price_update) = price_rx.recv().await` en `lib.rs`.
- [ ] **Pool de Conexiones DB:** Implementar `r2d2` o `sqlx` en SQLite para evitar "Database Locked Errors" cuando 10 hilos intenten cerrar 10 posiciones al mismo tiempo.
- [ ] **Parallel Execution:** Envolver el disparador de ventas (`execute_emergency_sell`) dentro de un `tokio::spawn(async move { ... })`. Esto asegura que si 5 tokens colapsan, Solana reciba 5 transacciones independientes en el mismo milisegundo sin bloquear el loop principal.

### FASE 3: Inteligencia de Eventos & Jito Bundles
- [ ] **Wallet Event Monitor:** Sustituir la "venta fallida" por un `accountSubscribe` vía WebSocket de Helius.
  - *Lógica:* Si el bot detecta por WSS que el balance del token cayó a 0 (porque el usuario vendió en Trojan manualmente), SQLite marca `active: false` y apaga el monitor silenciosamente al milisegundo. Cero intentos fallidos, cero ruido de logs.
- [ ] **Jito Bundles Abstraction:** Refactorizar `jito_client` para permitir agrupar *Múltiples Ventas de Pánico* en 1 solo Bundle. Se paga una sola propina al minero, y los 3 tokens en SL se liquidan de forma atómica.

---

## 📝 Notas para el Despliegue
- La subida se hará a través del pipeline ya configurado (`CI GitHub Actions -> GHCR -> GCP Pull`).
- Al eliminar `targets.json`, ya no nos preocuparemos de *Zero-Builds* por agregar monedas, todo será en base de datos.
- Archivos clave a modificar mañana: `lib.rs`, `sqlite.rs` (state_manager), `telegram_commands.rs`, `executor_v2.rs`.
