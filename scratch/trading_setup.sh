#!/bin/bash
# Script simple de inicio de sesión de trading para Ruben (Linux/42 style)

PROJECT_DIR="$HOME/trading_engine"
SESSION_LOG="$PROJECT_DIR/logs/session_$(date +%Y%m%d).log"

mkdir -p "$PROJECT_DIR/logs"
mkdir -p "$PROJECT_DIR/manual_audits"

echo "--- SESIÓN DE TRADING INICIADA: $(date) ---" >> "$SESSION_LOG"
echo "1. Verifica RPC en Helius..."
echo "2. Abre Telegram Desktop (Workspace 4)."
echo "3. Carga RugCheck.xyz en Brave/Librewolf."
echo "------------------------------------------"

# Abrir herramientas (opcional, ajusta según tus apps)
# telegram-desktop &
# brave "https://rugcheck.xyz" "https://dexscreener.com/solana" &

echo "Entorno listo. ¡Buena caza del 10X!"
