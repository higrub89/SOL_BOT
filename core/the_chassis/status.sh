#!/bin/bash

# 🏎️ The Chassis - Estado del Sistema
# Muestra un resumen visual del estado actual del bot

clear

echo "╔════════════════════════════════════════════════════════════╗"
echo "║         🏎️  THE CHASSIS - Estado del Sistema             ║"
echo "║                    v0.9.0                                  ║"
echo "╚════════════════════════════════════════════════════════════╝"
echo ""

# Verificar archivos críticos
echo "📁 ARCHIVOS DE CONFIGURACIÓN:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

if [ -f ".env" ]; then
    echo "   ✅ .env encontrado"
    if grep -q "HELIUS_API_KEY=" .env; then
        if grep -q "HELIUS_API_KEY=$" .env; then
            echo "      ⚠️  HELIUS_API_KEY no configurado"
        else
            echo "      ✅ HELIUS_API_KEY configurado"
        fi
    fi
    if grep -q "TELEGRAM_BOT_TOKEN=" .env; then
        if grep -q "TELEGRAM_BOT_TOKEN=$" .env; then
            echo "      ⚠️  Telegram NO configurado"
        else
            echo "      ✅ Telegram configurado"
        fi
    fi
else
    echo "   ❌ .env NO encontrado"
fi

if [ -f "targets.json" ]; then
    echo "   ✅ targets.json encontrado"
else
    echo "   ❌ targets.json NO encontrado"
fi

echo ""
echo "📊 TARGETS CONFIGURADOS:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

if [ -f "targets.json" ]; then
    # Contar targets totales
    TOTAL_TARGETS=$(grep -c '"symbol"' targets.json)
    # Contar targets activos
    ACTIVE_TARGETS=$(grep -c '"active": true' targets.json)
    
    echo "   • Total de targets: $TOTAL_TARGETS"
    echo "   • Targets activos:  $ACTIVE_TARGETS"
    
    # Mostrar símbolos activos
    if [ $ACTIVE_TARGETS -gt 0 ]; then
        echo ""
        echo "   🎯 Tokens en monitoreo:"
        # Extraer símbolos de targets activos (simple parsing)
        grep -B 5 '"active": true' targets.json | grep '"symbol"' | sed 's/.*"symbol": "\(.*\)".*/      • \1/'
    fi
    
    # Mostrar configuración global
    echo ""
    echo "   ⚙️  Configuración global:"
    AUTO_EXEC=$(grep '"auto_execute"' targets.json | grep -o 'true\|false')
    INTERVAL=$(grep '"monitor_interval_sec"' targets.json | grep -o '[0-9]*')
    
    if [ "$AUTO_EXEC" == "true" ]; then
        echo "      • Auto-Execute:  🔴 ACTIVADO"
    else
        echo "      • Auto-Execute:  🟡 DESACTIVADO"
    fi
    echo "      • Intervalo:     ${INTERVAL}s"
fi

echo ""
echo "🔧 ESTADO DE COMPILACIÓN:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

if [ -f "target/debug/the_chassis" ]; then
    BINARY_DATE=$(stat -c %y "target/debug/the_chassis" 2>/dev/null | cut -d' ' -f1)
    echo "   ✅ Binario debug compilado (${BINARY_DATE})"
else
    echo "   ⚠️  No hay binario debug compilado"
fi

if [ -f "target/release/the_chassis" ]; then
    BINARY_DATE=$(stat -c %y "target/release/the_chassis" 2>/dev/null | cut -d' ' -f1)
    echo "   ✅ Binario release compilado (${BINARY_DATE})"
else
    echo "   ⚠️  No hay binario release compilado"
fi

echo ""
echo "📚 DOCUMENTACIÓN:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

FILES=("README.md" "TELEGRAM_SETUP.md" "IMPLEMENTATION_SUMMARY.md")
for file in "${FILES[@]}"; do
    if [ -f "$file" ]; then
        echo "   ✅ $file"
    else
        echo "   ❌ $file (faltante)"
    fi
done

echo ""
echo "🚀 COMANDOS DISPONIBLES:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "   • ./start.sh          - Iniciar el bot (modo interactivo)"
echo "   • cargo run           - Iniciar en modo debug"
echo "   • cargo run --release - Iniciar en modo release (optimizado)"
echo "   • ./status.sh         - Ver este estado (este script)"
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Resumen final
echo ""
if [ -f ".env" ] && [ -f "targets.json" ]; then
    ACTIVE_COUNT=$(grep -c '"active": true' targets.json 2>/dev/null || echo "0")
    if [ "$ACTIVE_COUNT" -gt 0 ]; then
        echo "✅ Sistema LISTO para ejecutar"
        echo ""
        echo "Para iniciar el bot, ejecuta:"
        echo "   ./start.sh"
    else
        echo "⚠️  Sistema configurado pero sin targets activos"
        echo ""
        echo "Activa un target en targets.json cambiando 'active' a true"
    fi
else
    echo "⚠️  Configuración incompleta"
    echo ""
    echo "Completa los archivos .env y targets.json antes de ejecutar"
fi

echo ""
