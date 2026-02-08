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

# Verificar que existe targets.json
if [ ! -f "targets.json" ]; then
    echo "❌ ERROR: No se encuentra targets.json"
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
ACTIVE_TARGETS=$(grep -o '"active": true' targets.json | wc -l)
echo "   • Targets activos: $ACTIVE_TARGETS"

# Verificar auto_execute
if grep -q '"auto_execute": true' targets.json; then
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
    cargo run --release
else
    cargo run
fi
