#!/bin/bash
# ╔═══════════════════════════════════════════════════════════════╗
# ║        SETUP INICIAL — Servidor GCP The Chassis Bot           ║
# ║        EDICIÓN ESPECIAL: HIGH PERFORMANCE & REAL-TIME         ║
# ╚═══════════════════════════════════════════════════════════════╝

set -e  # Salir si cualquier comando falla

echo ""
echo "╔══════════════════════════════════════════════════════════╗"
echo "║      🚀 THE CHASSIS — High Performance Setup GCP         ║"
echo "╚══════════════════════════════════════════════════════════╝"
echo ""

# ─── 1. Actualizar sistema ───────────────────────────────────────
echo "📦 [1/7] Actualizando sistema..."
sudo apt-get update -qq
sudo apt-get install -y -qq \
    curl \
    git \
    ca-certificates \
    gnupg \
    lsb-release \
    rsync \
    ufw \
    chrony  # Instalación inmediata de Chrony para el tiempo

# ─── 2. Ingeniería de Rendimiento (The Chassis Tuning) ───────────
echo "🏎️  [2/7] Aplicando Hardening de Kernel y Red..."

# Configurar límites de recursos (Prioridad RT y Memoria Bloqueada)
sudo bash -c "cat > /etc/security/limits.d/99-realtime.conf << EOF
* soft    rtprio          99
* hard    rtprio          99
* soft    memlock         unlimited
* hard    memlock         unlimited
* soft    nofile          65535
* hard    nofile          65535
EOF"

# Tuning del Stack de Red para Solana (Buffers de 16MB)
sudo bash -c "cat > /etc/sysctl.d/10-trading-performance.conf << EOF
net.core.rmem_max = 16777216
net.core.wmem_max = 16777216
net.core.rmem_default = 16777216
net.core.wmem_default = 16777216
net.ipv4.tcp_fastopen = 3
net.ipv4.tcp_low_latency = 1
net.ipv4.tcp_slow_start_after_idle = 0
net.core.netdev_max_backlog = 5000
EOF"
sudo sysctl --system

# Sincronización de Tiempo con Google Metadata (Baja latencia)
sudo bash -c "cat > /etc/chrony/chrony.conf << EOF
server metadata.google.internal prefer iburst
driftfile /var/lib/chrony/drift
makestep 1.0 3
rtcsync
EOF"
sudo systemctl restart chrony

# ─── 3. Instalar Docker ──────────────────────────────────────────
echo "🐳 [3/7] Instalando Docker..."
if ! command -v docker &> /dev/null; then
    curl -fsSL https://get.docker.com | bash
    sudo usermod -aG docker $USER
    echo "    ✅ Docker instalado."
else
    echo "    ✅ Docker ya instalado: $(docker --version)"
fi

if ! docker compose version &> /dev/null; then
    sudo apt-get install -y docker-compose-plugin
fi

# ─── 4. Estructura de directorios ────────────────────────────────
echo "📁 [4/7] Creando estructura de directorios..."
mkdir -p ~/bot_trading/logs
mkdir -p ~/bot_trading/operational/logs
touch ~/bot_trading/trading_state.db
touch ~/bot_trading/pools_cache.json
touch ~/bot_trading/settings.json

# ─── 5. Variables de Entorno ─────────────────────────────────────
echo "🔐 [5/7] Configurando variables de entorno..."
if [ ! -f ~/bot_trading/.env ]; then
    cat > ~/bot_trading/.env << 'EOF'
# =============================================
# THE CHASSIS — Variables de Entorno
# =============================================
HELIUS_API_KEY=PON_TU_API_KEY_AQUI
WALLET_ADDRESS=PON_TU_WALLET_PUBLICA_AQUI
WALLET_PRIVATE_KEY=PON_TU_CLAVE_PRIVADA_AQUI
JUPITER_API_KEY=PON_TU_JUPITER_KEY_AQUI
TELEGRAM_BOT_TOKEN=PON_TU_TOKEN_TELEGRAM
TELEGRAM_CHAT_ID=PON_TU_CHAT_ID
MAX_LATENCY_MS=150
RUST_LOG=info
EOF
    chmod 600 ~/bot_trading/.env
    echo "    ⚠️  Archivo .env creado."
else
    echo "    ✅ .env ya existe."
fi

# ─── 6. Firewall ─────────────────────────────────────────────────
echo "🛡️  [6/7] Configurando Firewall..."
sudo ufw allow 22/tcp > /dev/null 2>&1
sudo ufw --force enable > /dev/null 2>&1

# ─── 7. Habilitar Docker ─────────────────────────────────────────
echo "⚙️  [7/7] Habilitando Docker..."
sudo systemctl enable docker > /dev/null 2>&1
sudo systemctl start docker  > /dev/null 2>&1

echo ""
echo "╔══════════════════════════════════════════════════════════╗"
echo "║          ✅ SETUP DE ALTO RENDIMIENTO COMPLETADO         ║"
echo "╚══════════════════════════════════════════════════════════╝"
echo ""
echo "🔥 IMPORTANTE: Debes reiniciar el servidor para aplicar los cambios de Kernel:"
echo "   sudo reboot"
echo ""
echo "Después del reinicio, verifica los límites con: ulimit -r -l"
