#!/bin/bash
# ╔═══════════════════════════════════════════════════════════════════════╗
# ║        THE CHASSIS — GCP GCE DEPLOYMENT SCRIPT (HFT Optimized)        ║
# ╚═══════════════════════════════════════════════════════════════════════╝
set -euo pipefail

PROJECT_ID="project-828d4ae0-6385-40d2-aa6"
INSTANCE_NAME="solana-bot-v1"
ZONE="us-east4-b"
IMAGE_NAME="bot"
GCR_PATH="europe-west1-docker.pkg.dev/${PROJECT_ID}/the-chassis-repo/${IMAGE_NAME}:latest"

echo "🚀 Preparando despliegue de The Chassis en GCP..."

# 1. Build y Push
echo "📦 Construyendo imagen Docker..."
docker build -t ${GCR_PATH} .

echo "⬆️ Subiendo imagen a Artifact Registry..."
docker push ${GCR_PATH}

# 2. Definir Startup Script (Seguro y Optimizado)
STARTUP_SCRIPT=$(cat <<EOF
#!/bin/bash
# THE CHASSIS - Startup Script v2.2
set -euo pipefail

# Redirigir toda la salida a un log local para diagnóstico
exec > >(tee /var/log/chassis-startup.log|logger -t startup-script -s 2>/dev/console) 2>&1

echo "🚀 Iniciando setup del entorno Chassis..."

# Instalación de Docker si no existe (Ubuntu 24.04)
if ! command -v docker &> /dev/null; then
  apt-get update && apt-get install -y docker.io
  systemctl start docker
  systemctl enable docker
fi

# Configurar autenticación para Artifact Registry
gcloud auth configure-docker europe-west1-docker.pkg.dev --quiet

# Preparar archivo de entorno seguro
ENV_FILE="/root/.chassis.env"
touch \$ENV_FILE
chmod 600 \$ENV_FILE

echo "📥 Recuperando secretos de Secret Manager..."

fetch_secret() {
    local secret_name=\$1
    local val
    val=\$(gcloud secrets versions access latest --secret="\$secret_name" --project="${PROJECT_ID}" --quiet 2>/dev/null)
    if [ -z "\$val" ]; then
        echo "❌ Error: No se pudo recuperar el secreto \$secret_name" >&2
        exit 1
    fi
    echo "\$val"
}

# Asignar a variables locales primero para evitar contaminar el archivo .env si falla
H_KEY=\$(fetch_secret HELIUS_API_KEY)
J_KEY=\$(fetch_secret JUPITER_API_KEY)
W_KEY=\$(fetch_secret WALLET_PRIVATE_KEY)
W_ADDR=\$(fetch_secret WALLET_ADDRESS)
T_TOKEN=\$(fetch_secret TELEGRAM_BOT_TOKEN)
T_ID=\$(fetch_secret TELEGRAM_CHAT_ID)

cat <<ENV_EOF > \$ENV_FILE
HELIUS_API_KEY=\$H_KEY
JUPITER_API_KEY=\$J_KEY
WALLET_PRIVATE_KEY=\$W_KEY
WALLET_ADDRESS=\$W_ADDR
TELEGRAM_BOT_TOKEN=\$T_TOKEN
TELEGRAM_CHAT_ID=\$T_ID
SOLANA_WS_URL=wss://bold-dry-friday.solana-mainnet.quiknode.pro/8717ec45daa16137b672fe894c3655061ab521bd/
HUNTER_MODE=devnet
RUST_LOG=info
ENV_EOF

echo "✅ Secretos recuperados y validados correctamente en \$ENV_FILE."

# Pull y Run
docker pull ${GCR_PATH}
docker stop solana_trading_bot || true
docker rm solana_trading_bot || true

echo "⚡ Iniciando contenedor con SCHED_FIFO support..."
docker run -d \
  --name solana_trading_bot \
  --restart always \
  --cap-add=SYS_NICE \
  --ulimit rtprio=99 \
  --env-file \$ENV_FILE \
  ${GCR_PATH} \
  ./the_chassis_app scan

# Limpiar historial
history -c && history -w
EOF
)

# 3. Actualizar o Crear Instancia
echo "🖥️  Actualizando GCE ${INSTANCE_NAME} con nueva configuración..."

# Verificamos si la instancia existe
if gcloud compute instances describe ${INSTANCE_NAME} --zone=${ZONE} --project=${PROJECT_ID} &>/dev/null; then
    echo "♻️  Instancia detectada. Actualizando metadata..."
    gcloud compute instances add-metadata ${INSTANCE_NAME} \
        --zone=${ZONE} \
        --project=${PROJECT_ID} \
        --metadata=startup-script="$STARTUP_SCRIPT"
    
    echo "🔄 Reiniciando instancia para aplicar cambios..."
    gcloud compute instances reset ${INSTANCE_NAME} --zone=${ZONE} --project=${PROJECT_ID}
else
    echo "🆕 Creando nueva instancia..."
    gcloud compute instances create ${INSTANCE_NAME} \
        --project=${PROJECT_ID} \
        --zone=${ZONE} \
        --machine-type=e2-micro \
        --image-family=ubuntu-2404-lts \
        --image-project=ubuntu-os-cloud \
        --metadata=startup-script="$STARTUP_SCRIPT" \
        --scopes=https://www.googleapis.com/auth/cloud-platform \
        --tags=chassis-bot
fi

echo "✅ Proceso completado."
echo "----------------------------------------------------------------"
echo "🔍 Monitoreo de logs:"
echo "   gcloud compute ssh ${INSTANCE_NAME} --zone=${ZONE} -- 'docker logs -f solana_trading_bot'"
echo "----------------------------------------------------------------"
