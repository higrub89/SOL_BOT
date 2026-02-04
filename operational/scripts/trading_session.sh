#!/bin/bash
# ============================================================================
# TRADING SESSION MANAGER
# Autor: Ruben
# Descripción: Script de inicialización de sesión de trading con estándares
#              de ingeniería de alta precisión (42 Madrid style)
# ============================================================================

set -e  # Exit on error

# Colores para output (Luxury Terminal Aesthetics)
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
MAGENTA='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Directorios del proyecto
PROJECT_ROOT="/home/ruben/Automatitation/bot_trading"
LOGS_DIR="$PROJECT_ROOT/operational/logs"
AUDITS_DIR="$PROJECT_ROOT/operational/audits"
SESSION_DATE=$(date +%Y%m%d)
SESSION_TIME=$(date +%H%M%S)
SESSION_LOG="$LOGS_DIR/session_${SESSION_DATE}_${SESSION_TIME}.log"

# ============================================================================
# FUNCIONES
# ============================================================================

print_header() {
    echo -e "${CYAN}"
    echo "╔════════════════════════════════════════════════════════════════╗"
    echo "║         🚀 SOLANA TRADING ENGINE - SESSION MANAGER 🚀         ║"
    echo "║                    Ruben's Trading Station                     ║"
    echo "╚════════════════════════════════════════════════════════════════╝"
    echo -e "${NC}"
}

print_section() {
    echo -e "\n${MAGENTA}▶ $1${NC}"
}

print_success() {
    echo -e "${GREEN}✓ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠ $1${NC}"
}

print_error() {
    echo -e "${RED}✗ $1${NC}"
}

check_directory_structure() {
    print_section "Verificando Estructura de Directorios..."
    
    local dirs=(
        "$LOGS_DIR"
        "$AUDITS_DIR"
        "$PROJECT_ROOT/operational/scripts"
        "$PROJECT_ROOT/operational/wallets"
    )
    
    for dir in "${dirs[@]}"; do
        if [ -d "$dir" ]; then
            print_success "Directorio OK: $dir"
        else
            print_warning "Creando: $dir"
            mkdir -p "$dir"
        fi
    done
}

initialize_session_log() {
    print_section "Inicializando Log de Sesión..."
    
    cat > "$SESSION_LOG" << EOF
================================================================================
SESIÓN DE TRADING - $(date)
================================================================================
Usuario: $USER
Hostname: $(hostname)
Directorio: $PROJECT_ROOT
Session ID: ${SESSION_DATE}_${SESSION_TIME}
================================================================================

EOF
    
    print_success "Log creado: $SESSION_LOG"
}

check_network_connectivity() {
    print_section "Verificando Conectividad de Red..."
    
    if ping -c 1 -W 2 8.8.8.8 &> /dev/null; then
        print_success "Conectividad a Internet: OK"
        echo "[$(date)] Network check: PASSED" >> "$SESSION_LOG"
    else
        print_error "Sin conexión a Internet"
        echo "[$(date)] Network check: FAILED" >> "$SESSION_LOG"
        exit 1
    fi
}

check_rpc_endpoint() {
    print_section "Verificando RPC Endpoint..."
    
    # Verificar si existe archivo de configuración de RPC
    RPC_CONFIG="$PROJECT_ROOT/operational/.rpc_config"
    
    if [ -f "$RPC_CONFIG" ]; then
        RPC_URL=$(cat "$RPC_CONFIG")
        print_success "RPC configurado: ${RPC_URL:0:30}..."
        echo "[$(date)] RPC configured: YES" >> "$SESSION_LOG"
    else
        print_warning "RPC no configurado. Usando nodo público (NO RECOMENDADO)"
        print_warning "Para configurar RPC privado:"
        echo -e "  ${CYAN}1. Registrate en https://helius.dev/${NC}"
        echo -e "  ${CYAN}2. Copia tu API URL${NC}"
        echo -e "  ${CYAN}3. Ejecuta: echo 'TU_RPC_URL' > $RPC_CONFIG${NC}"
        echo "[$(date)] RPC configured: NO - Using public nodes" >> "$SESSION_LOG"
    fi
}

display_trading_checklist() {
    print_section "Checklist Pre-Operación"
    
    echo -e "${YELLOW}"
    cat << "EOF"
    □ Wallet de trading fondeada (solo capital del día)
    □ Telegram Desktop abierto (@solana_trojanbot)
    □ RugCheck.xyz en navegador (https://rugcheck.xyz)
    □ Dexscreener en navegador (https://dexscreener.com/solana)
    □ Configuración de Trojan verificada:
        ├─ Slippage: 20-30%
        ├─ Priority Fee: 0.005 SOL
        ├─ Jito Tip: ON (0.001 SOL)
        └─ Auto-Buy: OFF
EOF
    echo -e "${NC}"
}

create_audit_template() {
    print_section "Preparando Template de Auditoría..."
    
    AUDIT_FILE="$AUDITS_DIR/audit_template_${SESSION_DATE}.md"
    
    cat > "$AUDIT_FILE" << 'EOF'
# CHECKLIST DE AUDITORÍA QUIRÚRGICA - SOLANA MEMES

## 1. Datos Básicos
- Token CA (Contract Address): 
- Token Symbol: 
- Narrativa (IA, Cultura, Meme): 
- Liquidez Inicial: 
- Fecha/Hora: 

## 2. Telemetría de Seguridad (RugCheck.xyz)
- [ ] LP Burned (100%): ☐ SI ☐ NO
- [ ] Mint Authority Disabled: ☐ SI ☐ NO
- [ ] Top 10 Holders < 15%: ☐ SI ☐ NO (%_____)
- [ ] RugCheck Score: ___/100

## 3. Análisis de Distribución
- Total Holders: 
- Top 5 Wallets (%): 
- Dev Wallet Identificada: ☐ SI ☐ NO

## 4. Decisión de Entrada
- [ ] APROBADO para entrada: ☐ SI ☐ NO
- Tamaño de Posición: ___ SOL
- Precio de Entrada: $ ___

## 5. Estrategia de Salida
- [ ] TP 1 (2X - Recuperar Principal): $ ___
- [ ] TP 2 (5X - Ganancia Parcial): $ ___
- [ ] TP 3 (10X - Moonshot): $ ___
- [ ] Stop Loss (-30%): $ ___

## 6. Resultado Final (Completar al cerrar posición)
- Precio de Salida: $ ___
- ROI: ___% 
- Ganancia/Pérdida: ___ SOL
- Lecciones Aprendidas:

EOF
    
    print_success "Template creado: $AUDIT_FILE"
    echo "[$(date)] Audit template created: $AUDIT_FILE" >> "$SESSION_LOG"
}

open_tools() {
    print_section "Abriendo Herramientas..."
    
    # Abrir Telegram Desktop (si está instalado)
    if command -v telegram-desktop &> /dev/null; then
        telegram-desktop &> /dev/null &
        print_success "Telegram Desktop iniciado"
    else
        print_warning "Telegram Desktop no encontrado. Ábrelo manualmente."
    fi
    
    # Abrir navegador con herramientas (opcional, descomenta si quieres)
    # if command -v brave &> /dev/null; then
    #     brave "https://rugcheck.xyz" "https://dexscreener.com/solana" &> /dev/null &
    #     print_success "Brave abierto con RugCheck y Dexscreener"
    # fi
}

show_session_summary() {
    print_section "Resumen de Sesión"
    
    echo -e "${CYAN}"
    cat << EOF
    Session ID:    ${SESSION_DATE}_${SESSION_TIME}
    Log File:      $SESSION_LOG
    Audit Dir:     $AUDITS_DIR
    
    🎯 REGLAS DE ORO:
    1. Nunca operar sin completar checklist de auditoría
    2. SIEMPRE vender el 50% al 2X (recuperar principal)
    3. Stop Loss estricto al -30% si no toca TP1
    4. NO dejar fondos en burner wallet al final del día
    
    💎 Buena caza del 10X. Operamos con precisión suiza.
EOF
    echo -e "${NC}"
}

# ============================================================================
# EJECUCIÓN PRINCIPAL
# ============================================================================

main() {
    clear
    print_header
    
    check_directory_structure
    initialize_session_log
    check_network_connectivity
    check_rpc_endpoint
    create_audit_template
    display_trading_checklist
    # open_tools  # Descomenta si quieres que abra apps automáticamente
    show_session_summary
    
    echo -e "\n${GREEN}Sistema listo. ¡Que comience la operación!${NC}\n"
    echo "[$(date)] Session initialized successfully" >> "$SESSION_LOG"
}

# Ejecutar
main
