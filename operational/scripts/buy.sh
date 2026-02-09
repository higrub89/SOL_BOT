#!/usr/bin/env bash
# Script helper para ejecutar compras rápidas desde la terminal

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$SCRIPT_DIR/../core/the_chassis"

if [ $# -lt 2 ]; then
    echo "Uso: ./buy.sh <TOKEN_MINT> <SOL_AMOUNT>"
    echo "Ejemplo: ./buy.sh 5TATk16oMrt4vsMR8WwQ9AtiPeosdJhXFkp2UhGJpump 0.05"
    exit 1
fi

TOKEN_MINT=$1
SOL_AMOUNT=$2

echo "╔════════════════════════════════════════════════════════════╗"
echo "║        🚀 THE CHASSIS - QUICK BUY COMMAND 🚀              ║"
echo "╚════════════════════════════════════════════════════════════╝"
echo ""
echo "Token:  $TOKEN_MINT"
echo "Amount: $SOL_AMOUNT SOL"
echo ""
echo "Ejecutando compra..."
echo ""

# Crear un targets.json temporal para la compra
cat > buy_temp.json << EOF
{
  "mode": "buy",
  "token_mint": "$TOKEN_MINT",
  "amount_sol": $SOL_AMOUNT
}
EOF

# Compilar y ejecutar
cargo run --release -- buy "$TOKEN_MINT" "$SOL_AMOUNT"

# Limpiar
rm -f buy_temp.json

echo ""
echo "✅ Proceso completado."
echo "💡 Tip: Para activar el monitor de protección, añade este token a targets.json"
