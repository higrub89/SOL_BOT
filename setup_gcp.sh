#!/bin/bash
# ╔═══════════════════════════════════════════════════════════════╗
# ║       SETUP INICIAL — Servidor GCP The Chassis Bot           ║
# ║   Ejecutar UNA SOLA VEZ: bash setup_gcp.sh                   ║
# ╚═══════════════════════════════════════════════════════════════╝
# Uso:
#   1. Copia este script al servidor GCP
#   2. chmod +x setup_gcp.sh && bash setup_gcp.sh
#   3. Editar ~/.bot_trading/.env con tus claves reales

set -e  # Salir si cualquier comando falla

echo ""
echo "╔══════════════════════════════════════════════════════════╗"
echo "║     🚀 THE CHASSIS — Setup Servidor GCP                ║"
echo "╚══════════════════════════════════════════════════════════╝"
echo ""

# ─── 1. Actualizar sistema ───────────────────────────────────────
echo "📦 [1/6] Actualizando sistema..."
sudo apt-get update -qq
sudo apt-get install -y -qq \
    curl \
    git \
    ca-certificates \
    gnupg \
    lsb-release \
    rsync \
    ufw

# ─── 2. Instalar Docker ──────────────────────────────────────────
echo "🐳 [2/6] Instalando Docker..."
if ! command -v docker &> /dev/null; then
    curl -fsSL https://get.docker.com | bash
    sudo usermod -aG docker $USER
    echo "   ✅ Docker instalado. Puede ser necesario re-loguear para usar sin sudo."
else
    echo "   ✅ Docker ya instalado: $(docker --version)"
fi

# Docker Compose V2 (plugin)
if ! docker compose version &> /dev/null; then
    sudo apt-get install -y docker-compose-plugin
fi
echo "   ✅ Docker Compose: $(docker compose version)"

# ─── 3. Crear estructura de directorios ──────────────────────────
echo "📁 [3/6] Creando estructura de directorios..."
mkdir -p ~/bot_trading/logs
mkdir -p ~/bot_trading/operational/logs

# Crear trading_state.db vacío si no existe (volumen Docker)
touch ~/bot_trading/trading_state.db
touch ~/bot_trading/pools_cache.json

# ─── 4. Crear .env en el servidor (con valores placeholder) ──────
echo "🔐 [4/6] Configurando variables de entorno..."
if [ ! -f ~/bot_trading/.env ]; then
    cat > ~/bot_trading/.env << 'EOF'
# =============================================
# THE CHASSIS — Variables de Entorno
# IMPORTANTE: Reemplaza los valores con los reales
# =============================================

# Helius RPC (Obtener en https://dev.helius.xyz)
HELIUS_API_KEY=PON_TU_API_KEY_AQUI

# Wallet del bot (dirección pública)
WALLET_ADDRESS=PON_TU_WALLET_PUBLICA_AQUI

# Clave privada de la wallet (NUNCA compartir)
# Formato: base58 o array JSON de bytes
WALLET_PRIVATE_KEY=PON_TU_CLAVE_PRIVADA_AQUI

# Jupiter API (para swaps optimizados)
JUPITER_API_KEY=PON_TU_JUPITER_KEY_AQUI

# Telegram Notifications
TELEGRAM_BOT_TOKEN=PON_TU_TOKEN_TELEGRAM
TELEGRAM_CHAT_ID=PON_TU_CHAT_ID

# Latencia máxima permitida
MAX_LATENCY_MS=150

# Runtime
RUST_LOG=info
EOF
    chmod 600 ~/bot_trading/.env
    echo "   ⚠️  Archivo .env creado. EDITA con tus claves reales:"
    echo "       nano ~/bot_trading/.env"
else
    echo "   ✅ .env ya existe, no se sobreescribe."
fi

# ─── 5. Configurar Firewall ──────────────────────────────────────
echo "🛡️  [5/6] Configurando Firewall..."
sudo ufw allow 22/tcp   > /dev/null 2>&1  # SSH
sudo ufw --force enable > /dev/null 2>&1
echo "   ✅ Firewall activado (SSH permitido)"

# ─── 6. Habilitar Docker en arranque ─────────────────────────────
echo "⚙️  [6/6] Habilitando Docker al arranque..."
sudo systemctl enable docker > /dev/null 2>&1
sudo systemctl start docker  > /dev/null 2>&1

# ─── Resumen Final ───────────────────────────────────────────────
echo ""
echo "╔══════════════════════════════════════════════════════════╗"
echo "║                  ✅ SETUP COMPLETADO                    ║"
echo "╚══════════════════════════════════════════════════════════╝"
echo ""
echo "📋 PRÓXIMOS PASOS:"
echo ""
echo "  1️⃣  Editar el archivo .env con tus claves REALES:"
echo "       nano ~/bot_trading/.env"
echo ""
echo "  2️⃣  Configurar GitHub Secrets en tu repo:"
echo "       GCP_SERVER_IP  → $(curl -s ifconfig.me 2>/dev/null || echo 'tu-ip-aqui')"
echo "       GCP_USER       → $USER"
echo "       GCP_SSH_KEY    → Contenido de tu clave privada SSH"
echo "       TELEGRAM_BOT_TOKEN → Token de @BotFather"
echo "       TELEGRAM_CHAT_ID   → Tu Chat ID"
echo ""
echo "  3️⃣  Hacer git push a 'main' para activar el deploy automático"
echo ""
echo "  4️⃣  Verificar el bot:"
echo "       cd ~/bot_trading && docker compose logs -f"
echo ""
