---
name: solana-jito-mev
description: Empaquetado de transacciones atómicas con protección frente a MEV (Jito Labs).
auto-invokes:
  - Call Solana RPC, send transactions, parse slots
  - Cambios en módulos de ejecución de Jupiter/Raydium
---
# Solana Jito MEV Integration

## 1. Construcción de Bundles
- Toda orden de mercado vulnerable al frontrunning (Swaps de más de 0.5 SOL de liquidez) debe enrutarse por Jito Block Engine.
- Empaquetar la transacción destino junto con una transferencia puente al account de Tipping de Jito.

## 2. Dynamic Tipping
- La propina de Jito debe ser calculada dinámicamente en base al fee-tracker local (evitar pagar propinas estáticas que perjudican el PnL).

## 3. Manejo de Fallos (HTTP 429)
- En caso de latencia o `429 Too Many Requests` provenientes del Jito Block Engine, el fallback directo **DEBE ESTAR DESACTIVADO** para trades direccionales grandes, o caer al RPC público de Helius asumiendo riesgo de sándwich (limitando slippage estricto).
