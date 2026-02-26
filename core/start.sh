#!/bin/bash

# 🏎️ The Chassis - Script de Inicio Rápido
# Este script facilita el inicio del bot de trading

echo "╔════════════════════════════════════════════════════════════╗"
echo "║         🏎️  THE CHASSIS - Quick Start Script             ║"
echo "╚════════════════════════════════════════════════════════════╝"
echo ""

# Verificar que existe el .env
if [ ! -f ".env" ]; then
    echo "❌ ERROR: No se encuentra el archivo .env"
    echo "   Por favor, copia .env.example a .env y configura tus credenciales"
    exit 1
fi

# Verificar que existe settings.json
if [ ! -f "settings.json" ]; then
    echo "❌ ERROR: No se encuentra settings.json"
    echo "   Este archivo es necesario para configurar los tokens a monitorear"
    exit 1
fi

# Verificar configuración de Telegram (opcional)
if grep -q "TELEGRAM_BOT_TOKEN=$" .env || grep -q "TELEGRAM_CHAT_ID=$" .env; then
    echo "⚠️  AVISO: Telegram no está configurado"
    echo "   El bot funcionará sin notificaciones de Telegram"
    echo "   Lee TELEGRAM_SETUP.md para configurarlo"
    echo ""
fi

# Mostrar configuración actual
echo "📋 CONFIGURACIÓN ACTUAL:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Contar targets activos
if command -v sqlite3 &> /dev/null && [ -f "trading_state.db" ]; then
    ACTIVE_TARGETS=$(sqlite3 trading_state.db "SELECT count(*) FROM positions WHERE active = 1;")
else
    ACTIVE_TARGETS=0
fi
echo "   • Targets activos: $ACTIVE_TARGETS"

# Check si Auto Execute esta habilitado
if grep -q '"auto_execute": true' settings.json; then
    echo "   • Auto-Execute:    🔴 ACTIVADO (abrirá Jupiter automáticamente)"
else
    echo "   • Auto-Execute:    🟡 DESACTIVADO (requiere acción manual)"
fi

# Verificar Telegram
if ! grep -q "TELEGRAM_BOT_TOKEN=$" .env && ! grep -q "TELEGRAM_CHAT_ID=$" .env; then
    echo "   • Telegram:        ✅ CONFIGURADO"
else
    echo "   • Telegram:        ⚠️  NO CONFIGURADO"
fi

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Preguntar si quiere compilar en modo release o debug
echo "🔧 Modo de ejecución:"
echo "   1) Debug (más rápido de compilar, más lento de ejecutar)"
echo "   2) Release (más lento de compilar, más rápido de ejecutar) - RECOMENDADO"
echo ""
read -p "Selecciona una opción (1 o 2): " MODE

echo ""
echo "🚀 Iniciando The Chassis..."
echo ""

if [ "$MODE" == "2" ]; then
    cargo run --release --bin the_chassis_app
else
    cargo run --bin the_chassis_app
fi
