# 🚀 Bot Trading - Sistema de Operaciones de Alta Frecuencia en Solana

**Autor:** Rubén  
**Entorno:** Ubuntu/Linux  
**Stack Principal:** Rust (Core Engine), Python (Analytics), Telegram Bot API (Control)  
**Objetivo:** Operar el 5% de la cartera con disciplina militar para buscar retornos 10X en memecoins/tokens hyperagresivos.

---

## 📋 Filosofía del Proyecto

Este proyecto sigue la filosofía de **"Soberanía Técnica Progresiva"**:

1. **Fase Táctica:** Operar con herramientas de ejecución rápida (Trojan Bot) manteniendo control total de logs y auditoría manual.
2. **Fase Estratégica (HOY):** Uso de **"The Chassis"**, nuestro motor propio en Rust para monitoreo 24/7, trailing stop-loss y alertas de liquidez proactivas.
3. **Fase Soberana (Próximamente):** Ejecución directa on-chain (auto-sell/buy) sin dependencias de interfaces de terceros.

> *"No se trata de reinventar la rueda, sino de fabricar un chasis que nadie pueda sabotear."*

---

## 🏗️ Arquitectura del Sistema

```
bot_trading/
├── core/                # 🏎️ MOTOR PRINCIPAL (The Chassis)
│   ├── the_chassis/     # Lógica en Rust v1.0.0 (Precios, SL, Telegram)
│   ├── src/             # Código fuente (Trailing SL, Liquidity Monitor)
│   └── targets.json     # Configuración dinámica de posiciones
│
├── operational/         # Operativa diaria
│   ├── scripts/         # Automatización de entorno
│   └── wallets/         # Seguridad y gestión de claves
│
├── intelligence/        # Análisis y detección (En desarrollo)
│   └── models/          # Detección de Smart Money / Rug Pulls
│
└── docs/                # Documentación técnica y setups
```

---

## 🎯 Objetivos del 5% de Cartera

| Métrica | Objetivo |
|---------|----------|
| **Capital Asignado** | 5% de la cartera total |
| **Tamaño por Operación** | 0.25 - 0.5 SOL |
| **Take Profit 1** | 100% (2X) → Recuperar principal |
| **Moonshot Target** | 900% (10X) |
| **Trailing Stop Loss** | Dinámico (ajustado por The Chassis) |
| **Ratio Riesgo/Recompensa** | 1:10 mínimo |

---

## 🛠️ Stack Tecnológico Actual (v1.0.0)

### Motor de Control & Monitoreo (The Chassis)
- **Lenguaje:** Rust (Alta eficiencia y seguridad de memoria)
- **Control Remoto:** Telegram Bot API (Comandos interactivos `/status`, `/balance`)
- **Gestión de Riesgo:** Trailing Stop-Loss inteligente y Monitor de Liquidez en tiempo real.
- **RPC:** Helius RPC (Latencia optimizada)

### Ejecución & Seguridad
- **Ejecución:** Trojan on Solana + Jupiter Aggregator (vía The Chassis)
- **Auditoría:** RugCheck.xyz (Integrado en protocolo), Sol Sniffer
- **Infraestructura:** Ubuntu Linux + Jito Bundles (Anti-MEV)

---

## ⚙️ Configuración del Sistema

### Capa de Protección (The Chassis)
```json
// Ejemplo de configuración en targets.json
{
  "trailing_enabled": true,
  "trailing_distance_percent": 30.0,
  "trailing_activation_threshold": 50.0,
  "liquidity_check": true
}
```

### Capa de Ejecución (Trojan/Jito)
```
Slippage:         20-30%
Priority Fee:     0.005 SOL
Jito Tip:         ON (0.001 SOL)
```

---

## 🚀 Inicio Rápido

### 1. Arrancar el Motor Core
```bash
cd core/the_chassis
./target/release/the_chassis
```

### 2. Control desde Telegram
Busca a `@solbotruben` (o tu bot configurado) y usa:
- `/status` - Revisar todas las posiciones y drawdowns.
- `/balance` - Consultar SOL disponible.
- `/targets` - Ver configuración activa de tokens.

---

## 📊 Protocolo de Ejecución Actualizado

### Entrada (Manual/Asistida)
1. Detectar CA en Dexscreener/GMGN.
2. Auditoría rápida en RugCheck (Score > 85 obligatorio).
3. Compra vía Trojan (0.25-0.5 SOL).
4. **Alta en The Chassis:** Añadir a `targets.json` para protección automática.

### Salida (Protegida por Trailing SL)
1. **Fase de Crecimiento:** El bot monitorea el precio 24/7.
2. **Activación:** Al superar el `activation_threshold` (ej. +50%), el Trailing SL se activa.
3. **Protección:** Si el precio cae la distancia configurada (ej. -30% desde el pico), el bot lanza alerta inmediata con link de ejecución en Jupiter.

---

## 📈 Roadmap de Desarrollo

### ✅ Fase 1: Cimientos (Completado)
- [x] Estructura de proyecto y entorno Linux.
- [x] Conectividad RPC Helius optimizada.
- [x] Integración de notificaciones Telegram.

### ✅ Fase 2: El Chasis v1.0.0 (Completado)
- [x] Motor de monitoreo multithread en Rust.
- [x] **Comandos Interactivos de Telegram.**
- [x] **Sistema de Trailing Stop-Loss.**
- [x] **Detector de Liquidez y Rug Pulls.**

### 🚧 Fase 3: Automatización Total (Siguiente Paso)
- [ ] **Ejecución On-Chain Directa:** Venta automática sin pasar por navegador.
- [ ] **Auto-Buy:** Compra automática basada en filtros de seguridad.
- [ ] **Integración Yellowstone gRPC:** Monitoreo a nivel de slot (latencia <10ms).

---

## ⚠️ Disclaimers

> Este proyecto es para uso educacional y personal. El trading de criptomonedas conlleva riesgos significativos. No se garantiza ninguna ganancia. Opera solo con capital que puedas permitirte perder.

**Versión:** 1.0.0 (v1.0.0-release)  
**Última Actualización:** 2026-02-08  
**Licencia:** Privada (Configuraciones) | MIT (Componentes Core)

---  
Desarrollado con ⚡ por Ruben | 2026
