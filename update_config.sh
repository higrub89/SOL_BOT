#!/bin/bash

# Configuración del Servidor (Copiada de deploy.sh)
SERVER_USER="higuitaruben"
SERVER_IP="34.186.82.143"
REMOTE_DIR="~/bot_trading"
IDENTITY_FILE="~/.ssh/gcp_key"

# Colores
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${YELLOW}🔄 Actualizando configuración sin recompilar...${NC}"

# 1. Subir el archivo de configuración forzando el reemplazo (scp es más confiable que rsync para esto)
echo -e "${GREEN}📦 Subiendo targets.json al servidor...${NC}"

scp -i "$IDENTITY_FILE" \
  ./core/the_chassis/targets.json \
  "$SERVER_USER@$SERVER_IP:$REMOTE_DIR/core/the_chassis/targets.json"

if [ $? -ne 0 ]; then
    echo -e "${RED}❌ Error al sincronizar targets.json${NC}"
    exit 1
fi

# 2. Reiniciar SOLO el contenedor del bot (es instantáneo)
echo -e "${GREEN}⚡ Reiniciando el contenedor para aplicar cambios...${NC}"
ssh -i "$IDENTITY_FILE" "$SERVER_USER@$SERVER_IP" "cd $REMOTE_DIR && docker-compose restart bot"

if [ $? -eq 0 ]; then
    echo -e "${GREEN}✅ ¡Configuración actualizada con éxito!${NC}"
    echo -e "${YELLOW}Monitoriza los logs para confirmar el nuevo token: ssh -i $IDENTITY_FILE $SERVER_USER@$SERVER_IP 'cd $REMOTE_DIR && docker-compose logs -f'${NC}"
else
    echo -e "${RED}❌ Error al reiniciar el contenedor.${NC}"
fi
