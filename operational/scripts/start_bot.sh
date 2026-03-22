#!/bin/bash
# ═══════════════════════════════════════════════════════════════
#  🏎️ THE CHASSIS - Solana Trading Bot Launcher
#  Versión: 1.1.0-luxury | Auto-Buy & Auto-Sell Ready
# ═══════════════════════════════════════════════════════════════

CHASSIS_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

# Colores
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

echo ""
echo -e "${CYAN}╔════════════════════════════════════════════════════════════╗${NC}"
echo -e "${CYAN}║         🏎️  THE CHASSIS - Solana Trading Engine          ║${NC}"
echo -e "${CYAN}║           v1.1.0 - Full Automation Ready                   ║${NC}"
echo -e "${CYAN}╚════════════════════════════════════════════════════════════╝${NC}"
echo ""

# Menú de opciones
echo -e "${YELLOW}Selecciona el modo de operación:${NC}"
echo ""
echo "  1) 🛡️  MONITOR   - Vigilancia 24/7 con Trailing Stop-Loss"
echo "  2) 📡 SCAN      - Scanner de eventos Pump.fun en tiempo real"
echo "  3) 💰 BUY       - Compra directa desde terminal"
echo "  4) 🔧 BUILD     - Compilar el proyecto"
echo ""
read -p "Opción [1-4]: " choice

cd "$CHASSIS_DIR"

case $choice in
    1)
        echo -e "\n${GREEN}🛡️ Iniciando modo MONITOR...${NC}\n"
        cargo run -p the_chassis
        ;;
    2)
        echo -e "\n${GREEN}📡 Iniciando modo SCAN...${NC}\n"
        cargo run -p the_chassis -- scan
        ;;
    3)
        echo -e "\n${YELLOW}💰 Modo COMPRA DIRECTA${NC}"
        read -p "Mint Address: " mint
        read -p "Cantidad SOL: " sol
        echo -e "\n${GREEN}🚀 Ejecutando compra...${NC}\n"
        cargo run -p the_chassis -- buy --mint "$mint" --sol "$sol"
        ;;
    4)
        echo -e "\n${GREEN}🔧 Compilando The Chassis...${NC}\n"
        cargo build --release --workspace
        echo -e "\n${GREEN}✅ Compilación completada${NC}"
        ;;
    *)
        echo "Opción no válida"
        exit 1
        ;;
esac
