#!/bin/bash

# Configuración del Servidor
SERVER_USER="higuitaruben"
SERVER_IP="34.186.82.143"
REMOTE_DIR="~/bot_trading"
IDENTITY_FILE="~/.ssh/gcp_key"

# Colores para output bonito
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "\n${YELLOW}🚀 Iniciando despliegue de THE CHASSIS a $SERVER_IP ($SERVER_USER)...${NC}"

# 1. Verificar si tenemos rsync instalado
if ! command -v rsync &> /dev/null; then
    echo -e "${RED}❌ rsync no encontrado. Por favor instálalo (sudo apt install rsync)${NC}"
    exit 1
fi

# 2. Sincronizar archivos (Excluyendo basura)
echo -e "\n${GREEN}📦 Sincronizando código fuente...${NC}"

rsync -avz --progress --delete -e "ssh -i $IDENTITY_FILE" \
  --exclude 'target' \
  --exclude '.git' \
  --exclude '.env' \
  --exclude '.env.example' \
  --exclude 'logs/*.log' \
  --exclude 'bot_trading_pack.zip' \
  --exclude '.DS_Store' \
  --exclude 'node_modules' \
  ./ "$SERVER_USER@$SERVER_IP:$REMOTE_DIR"

if [ $? -ne 0 ]; then
    echo -e "\n${RED}❌ Error al sincronizar. Verifica tu clave SSH en Google Cloud.${NC}"
    exit 1
fi

# 3. Reiniciar contenedores remotamente
echo -e "\n${GREEN}🔄 Reiniciando Bot en el servidor...${NC}"
ssh -i "$IDENTITY_FILE" "$SERVER_USER@$SERVER_IP" "cd $REMOTE_DIR && \
  docker-compose down && \
  docker-compose up -d --build --remove-orphans && \
  docker image prune -f"

if [ $? -eq 0 ]; then
    echo -e "\n${GREEN}✅ ¡Despliegue EXITOSO! El bot está reiniciándose.${NC}"
    echo -e "${YELLOW}Monitoriza los logs con: ssh -i $IDENTITY_FILE $SERVER_USER@$SERVER_IP 'cd $REMOTE_DIR && docker-compose logs -f'${NC}\n"
else
    echo -e "\n${RED}❌ Error al reiniciar el bot.${NC}"
fi
