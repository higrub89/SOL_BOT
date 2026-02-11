#!/bin/bash

# Script para actualizar la wallet del bot de forma segura
# Uso: ./update_wallet.sh <NUEVA_PRIVATE_KEY>

set -e

NEW_WALLET_ADDRESS="AY2zXdAiZaWU9RTruqgnLMBvVmseGv82sV9PMo2HT6tP"
NEW_PRIVATE_KEY="$1"

if [ -z "$NEW_PRIVATE_KEY" ]; then
    echo "❌ Error: Debes proporcionar la clave privada"
    echo "Uso: ./update_wallet.sh <PRIVATE_KEY>"
    exit 1
fi

echo "🔄 Actualizando wallet del bot..."
echo "📍 Nueva dirección: $NEW_WALLET_ADDRESS"

# Conectar al servidor y actualizar .env
ssh -i ~/.ssh/gcp_key higuitaruben@34.186.82.143 << EOF
cd ~/bot_trading

# Backup del .env actual
cp .env .env.backup.\$(date +%Y%m%d_%H%M%S)

# Actualizar wallet address
sed -i 's|^WALLET_ADDRESS=.*|WALLET_ADDRESS=$NEW_WALLET_ADDRESS|' .env

# Actualizar private key
sed -i 's|^WALLET_PRIVATE_KEY=.*|WALLET_PRIVATE_KEY=$NEW_PRIVATE_KEY|' .env

echo "✅ Configuración actualizada"
echo "🔄 Reiniciando bot..."

# Reiniciar el bot
docker-compose restart

echo "✅ Bot reiniciado con nueva wallet"
EOF

echo ""
echo "════════════════════════════════════════════════════════"
echo "✅ ACTUALIZACIÓN COMPLETA"
echo "════════════════════════════════════════════════════════"
echo ""
echo "📝 Siguiente paso:"
echo "   Deposita SOL a: $NEW_WALLET_ADDRESS"
echo "   Mínimo: 0.065 SOL (~\$10 USD)"
echo ""
echo "🔍 Verificar logs:"
echo "   ssh -i ~/.ssh/gcp_key higuitaruben@34.186.82.143 'cd ~/bot_trading && docker-compose logs -f'"
echo ""
