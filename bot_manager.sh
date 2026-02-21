#!/bin/bash
# ═══════════════════════════════════════════════════════════════
#  🏎️ THE CHASSIS - Bot Manager
#  Controla el bot en background (persiste aunque cierres Termius)
# ═══════════════════════════════════════════════════════════════

BOT_DIR="/home/ruben/Automatitation/bot_trading"
BINARY="$BOT_DIR/target/release/the_chassis_app"
LOG="$BOT_DIR/logs/bot.log"
PID_FILE="$BOT_DIR/bot.pid"

GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

is_running() {
    if [ -f "$PID_FILE" ]; then
        PID=$(cat "$PID_FILE")
        if kill -0 "$PID" 2>/dev/null; then
            return 0
        fi
    fi
    return 1
}

case "$1" in
    start)
        if is_running; then
            echo -e "${YELLOW}⚠️  El bot ya está corriendo (PID: $(cat $PID_FILE))${NC}"
        else
            mkdir -p "$BOT_DIR/logs"
            cd "$BOT_DIR/core/the_chassis"
            nohup "$BINARY" monitor > "$LOG" 2>&1 &
            echo $! > "$PID_FILE"
            echo -e "${GREEN}✅ Bot arrancado en background (PID: $(cat $PID_FILE))${NC}"
            echo -e "${CYAN}   Logs: tail -f $LOG${NC}"
            sleep 3
            tail -5 "$LOG"
        fi
        ;;
    stop)
        if is_running; then
            PID=$(cat "$PID_FILE")
            kill "$PID"
            rm -f "$PID_FILE"
            echo -e "${RED}🛑 Bot detenido (PID: $PID)${NC}"
        else
            echo -e "${YELLOW}⚠️  El bot no está corriendo${NC}"
        fi
        ;;
    restart)
        $0 stop
        sleep 2
        $0 start
        ;;
    status)
        if is_running; then
            PID=$(cat "$PID_FILE")
            echo -e "${GREEN}🟢 Bot ACTIVO (PID: $PID)${NC}"
            echo ""
            echo "--- Último estado ---"
            tail -15 "$LOG" 2>/dev/null
        else
            echo -e "${RED}🔴 Bot DETENIDO${NC}"
        fi
        ;;
    logs)
        echo -e "${CYAN}📋 Siguiendo logs en tiempo real (Ctrl+C para salir)...${NC}"
        tail -f "$LOG"
        ;;
    *)
        echo ""
        echo -e "${CYAN}🏎️  THE CHASSIS - Bot Manager${NC}"
        echo "════════════════════════════════"
        echo "  ./bot_manager.sh start    — Arranca el bot en background"
        echo "  ./bot_manager.sh stop     — Detiene el bot"
        echo "  ./bot_manager.sh restart  — Reinicia el bot"
        echo "  ./bot_manager.sh status   — Ver estado y últimos logs"
        echo "  ./bot_manager.sh logs     — Seguir logs en tiempo real"
        echo ""
        ;;
esac
