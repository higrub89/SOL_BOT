# 🚀 Bot Trading - Sistema de Operaciones de Alta Frecuencia en Solana

**Autor:** Rubén  
**Entorno:** Ubuntu/Linux  
**Stack:** C/C++, Rust (futuro), Python (scripts operacionales)  
**Objetivo:** Operar el 5% de la cartera con disciplina militar para buscar retornos 10X en memecoins/tokens hyperagresivos.

---

## 📋 Filosofía del Proyecto

Este proyecto sigue la filosofía de **"Soberanía Técnica Progresiva"**:

1. **Fase Táctica (Hoy):** Operar con herramientas verificadas (Trojan Bot) manteniendo control total de wallets, logs y decisiones.
2. **Fase Estratégica (Paralelo):** Construir herramientas propias de monitorización y análisis en C++/Rust.
3. **Fase Soberana (Futuro):** Bot completamente autónomo, sin dependencias de terceros.

> *"No se trata de reinventar la rueda, sino de fabricar un chasis que nadie pueda sabotear."*

---

## 🏗️ Arquitectura del Sistema

```
bot_trading/
├── operational/          # Operativa diaria (HOY)
│   ├── scripts/         # Scripts de inicio, monitoreo, alertas
│   ├── logs/            # Registro de cada sesión de trading
│   ├── audits/          # Checklists de seguridad por token
│   └── wallets/         # Gestión de claves (NUNCA comittear)
│
├── core/                # Motor propio (DESARROLLO)
│   ├── src/            # Código fuente C++/Rust
│   ├── include/        # Headers
│   └── tests/          # Tests unitarios
│
├── intelligence/        # Análisis y detección (IA/ML)
│   ├── datasets/       # Datos históricos de tokens
│   ├── models/         # Modelos de predicción
│   └── scripts/        # Análisis de "Smart Money"
│
└── docs/               # Documentación técnica
```

---

## 🎯 Objetivos del 5% de Cartera

| Métrica | Objetivo |
|---------|----------|
| **Capital Asignado** | 5% de la cartera total |
| **Tamaño por Operación** | 0.25 - 0.5 SOL |
| **Take Profit 1** | 100% (2X) → Recuperar principal |
| **Moonshot Target** | 900% (10X) |
| **Stop Loss** | -30% sin tocar TP1 |
| **Ratio Riesgo/Recompensa** | 1:10 mínimo |

---

## 🛠️ Stack Tecnológico

### Operacional (Inmediato)
- **Bot de Ejecución:** Trojan on Solana ([@solana_trojanbot](https://t.me/solana_trojanbot))
- **RPC Privado:** Helius.dev (latencia <50ms)
- **Seguridad:** RugCheck.xyz, Sol Sniffer
- **Terminal:** Telegram Desktop (nativo Linux)

### Desarrollo (En Construcción)
- **Lenguaje Core:** C++ (tendiendo a Rust para paralelismo)
- **Conectividad:** gRPC (Yellowstone Geyser para Solana)
- **Testing:** Google Test / Catch2
- **Versionado:** Git + GitHub

---

## ⚙️ Configuración de Trojan Bot

### Parámetros de Precisión
```
Slippage:         20-30% (lanzamientos volátiles)
Priority Fee:     0.005 SOL
Jito Tip:         ON (0.001 SOL) - Anti-MEV
Auto-Buy:         OFF (inspección manual)
Confirmation:     OFF (velocidad crítica)
```

### Filtros de Seguridad (Obligatorios)
- ✅ LP Burned (100%)
- ✅ Mint Authority Disabled
- ✅ Top 10 Holders < 15%
- ✅ RugCheck Score > 85/100

---

## 🚀 Inicio Rápido

### 1. Preparar Entorno
```bash
cd /home/ruben/Automatitation/bot_trading
chmod +x operational/scripts/trading_session.sh
./operational/scripts/trading_session.sh
```

### 2. Configurar RPC Privado
1. Registrarse en [Helius.dev](https://www.helius.dev/)
2. Obtener API Key (Plan Free)
3. Configurar en Trojan: `/settings` → `RPC URL`

### 3. Checklist Pre-Operación
Antes de cada sesión, revisar:
- [ ] Wallet de trading fondeada (solo capital del día)
- [ ] RPC privado activo
- [ ] RugCheck.xyz abierto en navegador
- [ ] Log de sesión iniciado

---

## 📊 Protocolo de Ejecución

### Entrada
1. Detectar token en Dexscreener/GMGN
2. Copiar Contract Address (CA)
3. Auditar en RugCheck → Completar checklist
4. Si Score > 85 → Pegar CA en Trojan
5. Comprar 0.25-0.5 SOL

### Salida
1. **TP1 (100%):** Vender 50% → Recuperar principal
2. **TP2 (500%):** Vender 25% → Asegurar ganancia
3. **TP3 (1000%):** Vender resto → Moonshot

### Stop Loss
- Si cae -30% sin tocar TP1 → Liquidar posición completa

---

## 🔐 Seguridad y Soberanía

### Principios Irrenunciables
1. **Nunca** importar claves privadas en servicios no auditados
2. **Siempre** usar wallets "quemables" (burner wallets)
3. **Jamás** dejar más del 10% del capital diario en la wallet del bot
4. **Exportar** claves privadas a gestor de contraseñas (KeePassXC)

### Estructura de Wallets
```
Main Wallet (Cold):     95% de la cartera → Ledger/Hardware
Trading Wallet (Hot):   5% de la cartera → Phantom/Solflare
Burner Wallet (Bot):    10% del 5% → Generada por Trojan
```

---

## 📈 Roadmap de Desarrollo

### Fase 1: Operativa Inmediata (Semana 1)
- [x] Estructura de proyecto creada
- [ ] Configuración de Trojan completada
- [ ] Primera operación ejecutada con checklist
- [ ] Sistema de logs funcionando

### Fase 2: Herramientas de Monitoreo (Semanas 2-4)
- [ ] Script de monitoreo de wallet en terminal (Python)
- [ ] Alertas de Smart Money (copiar ballenas)
- [ ] Dashboard en terminal (ncurses)

### Fase 3: Motor Propio (Meses 2-3)
- [ ] Listener de Solana en C++/Rust
- [ ] Integración con Yellowstone gRPC
- [ ] Filtros de seguridad automáticos
- [ ] Ejecución de órdenes vía Jito Bundles

---

## 📚 Recursos Técnicos

### Documentación Oficial
- [Solana Docs](https://docs.solana.com/)
- [Trojan Official](https://trojanonsolana.com/)
- [Helius RPC](https://docs.helius.dev/)

### Comunidad y Análisis
- **X (Twitter):** @TrojanOnSolana, @heliuslabs
- **Telegram:** [@solana_trojanbot](https://t.me/solana_trojanbot)
- **Herramientas:** [RugCheck](https://rugcheck.xyz), [Dexscreener](https://dexscreener.com/solana)

---

## ⚠️ Disclaimers

> Este proyecto es para uso educacional y personal. El trading de criptomonedas conlleva riesgos significativos. No se garantiza ninguna ganancia. Opera solo con capital que puedas permitirte perder.

**Versión:** 0.1.0-alpha  
**Última Actualización:** 2026-02-04  
**Licencia:** MIT (Código propio) | Privado (Configuraciones)
