#!/bin/bash
# ============================================================================
# TRADING ENGINE - COMMAND HELPER
# Muestra todos los comandos disponibles del proyecto
# ============================================================================

CYAN='\033[0;36m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
MAGENTA='\033[0;35m'
NC='\033[0m'

clear

echo -e "${CYAN}"
cat << "EOF"
╔════════════════════════════════════════════════════════════════╗
║           🚀 BOT TRADING - COMMAND REFERENCE 🚀               ║
╚════════════════════════════════════════════════════════════════╝
EOF
echo -e "${NC}\n"

echo -e "${MAGENTA}▶ COMANDOS DE SESIÓN${NC}"
echo -e "${GREEN}./operational/scripts/trading_session.sh${NC}"
echo "   Inicia una nueva sesión de trading con todos los checks"
echo ""

echo -e "${MAGENTA}▶ MONITOREO Y SALUD DE RED${NC}"
echo -e "${GREEN}python3 operational/scripts/wallet_monitor.py <WALLET_ADDRESS> [RPC_URL]${NC}"
echo "   Monitorea el balance de tu wallet en tiempo real"
echo ""
echo -e "${GREEN}python3 operational/scripts/helius_engine.py${NC}"
echo "   Verifica la latencia de red (<150ms) y obtiene Priority Fees"
echo ""

echo -e "${MAGENTA}▶ GESTIÓN DE LOGS${NC}"
echo -e "${GREEN}ls -lht operational/logs/${NC}"
echo "   Lista todos los logs de sesiones (ordenados por fecha)"
echo ""
echo -e "${GREEN}cat operational/logs/session_YYYYMMDD_HHMMSS.log${NC}"
echo "   Ver un log específico"
echo ""
echo -e "${GREEN}tail -f operational/logs/session_YYYYMMDD_HHMMSS.log${NC}"
echo "   Seguir un log en tiempo real"
echo ""

echo -e "${MAGENTA}▶ AUDITORÍAS${NC}"
echo -e "${GREEN}ls operational/audits/${NC}"
echo "   Lista todos los templates de auditoría"
echo ""
echo -e "${GREEN}nano operational/audits/audit_template_YYYYMMDD.md${NC}"
echo "   Editar template de auditoría del día"
echo ""

echo -e "${MAGENTA}▶ CONFIGURACIÓN${NC}"
echo -e "${GREEN}echo 'TU_RPC_URL' > operational/.rpc_config${NC}"
echo "   Configurar RPC privado de Helius"
echo ""
echo -e "${GREEN}cat operational/.rpc_config${NC}"
echo "   Ver RPC configurado"
echo ""

echo -e "${MAGENTA}▶ DOCUMENTACIÓN${NC}"
echo -e "${GREEN}cat README.md${NC}"
echo "   Ver documentación principal del proyecto"
echo ""
echo -e "${GREEN}cat docs/QUICKSTART.md${NC}"
echo "   Ver guía de inicio rápido"
echo ""
echo -e "${GREEN}cat docs/TECHNICAL_ROADMAP.md${NC}"
echo "   Ver roadmap técnico"
echo ""
echo -e "${GREEN}cat PROJECT_STATUS.md${NC}"
echo "   Ver estado actual del proyecto"
echo ""

echo -e "${MAGENTA}▶ GIT${NC}"
echo -e "${GREEN}git status${NC}"
echo "   Ver cambios no commiteados"
echo ""
echo -e "${GREEN}git log --oneline${NC}"
echo "   Ver historial de commits"
echo ""
echo -e "${GREEN}git diff${NC}"
echo "   Ver diferencias en archivos modificados"
echo ""

echo -e "${MAGENTA}▶ HERRAMIENTAS WEB${NC}"
echo -e "${YELLOW}Trojan Bot:${NC} https://t.me/solana_trojanbot"
echo -e "${YELLOW}Helius RPC:${NC} https://www.helius.dev/"
echo -e "${YELLOW}RugCheck:${NC}   https://rugcheck.xyz"
echo -e "${YELLOW}Dexscreener:${NC} https://dexscreener.com/solana"
echo -e "${YELLOW}Solscan:${NC}    https://solscan.io/"
echo ""

echo -e "${MAGENTA}▶ ACCESOS RÁPIDOS${NC}"
echo -e "${GREEN}cd /home/ruben/Automatitation/bot_trading${NC}"
echo "   Ir al directorio del proyecto"
echo ""
echo -e "${GREEN}./operational/scripts/help.sh${NC}"
echo "   Mostrar esta ayuda de nuevo"
echo ""

echo -e "${CYAN}════════════════════════════════════════════════════════════════${NC}\n"
echo -e "${GREEN}💎 Recuerda las Reglas de Oro:${NC}"
echo "   1. SIEMPRE completar auditoría antes de comprar"
echo "   2. SIEMPRE vender 50% al 2X (recuperar principal)"
echo "   3. Stop Loss estricto al -30%"
echo "   4. NO dejar más de 2 SOL en burner wallet"
echo ""
echo -e "${YELLOW}¡Buena caza del 10X!${NC}\n"
