# 📱 Configuración de Notificaciones Telegram para The Chassis

Este documento explica cómo configurar las notificaciones de Telegram para recibir alertas automáticas cuando se active el stop-loss.

## ¿Por qué Telegram?

- ⚡ **Notificaciones instantáneas**: Recibes alertas en tu móvil al segundo
- 🔗 **Links directos**: Click para abrir Jupiter y ejecutar la venta
- 📊 **Información completa**: Precio, drawdown, y estado del mercado
- 🔒 **Privado y seguro**: Solo tú recibes las notificaciones

## Paso 1: Crear un Bot de Telegram

1. Abre Telegram y busca el bot **@BotFather**
2. Envía el comando `/newbot`
3. Sigue las instrucciones:
   - Elige un nombre para tu bot (ej: "The Chassis Alerts")
   - Elige un username (debe terminar en "bot", ej: "chassis_trading_bot")
4. BotFather te dará un **token** como este:
   ```
   1234567890:ABCdefGHIjklMNOpqrsTUVwxyz1234567890
   ```
5. **¡GUARDA ESTE TOKEN!** Lo necesitarás en el paso 3.

## Paso 2: Obtener tu Chat ID

### Opción A: Usando el bot GetIDs (Más Fácil)

1. Busca el bot **@getidsbot** en Telegram
2. Inicia una conversación con `/start`
3. El bot te enviará tu **Chat ID** (un número como `123456789`)

### Opción B: Manualmente

1. Envía un mensaje a tu bot recién creado (el que hiciste en Paso 1)
2. Abre esta URL en tu navegador (reemplaza `YOUR_BOT_TOKEN` con el token del Paso 1):
   ```
   https://api.telegram.org/botYOUR_BOT_TOKEN/getUpdates
   ```
3. Busca el campo `"chat":{"id":123456789}`
4. El número es tu **Chat ID**

## Paso 3: Configurar el .env

Edita el archivo `.env` en el directorio del proyecto:

```bash
HELIUS_API_KEY=1d8b1813-084e-41ed-8e93-87a503c496c6
WALLET_ADDRESS=6EJeiMFoBgQrUfbpt8jjXZdc5nASe2Kc8qzfVSyGrPQv
MAX_LATENCY_MS=150

# Telegram Notifications
TELEGRAM_BOT_TOKEN=1234567890:ABCdefGHIjklMNOpqrsTUVwxyz1234567890
TELEGRAM_CHAT_ID=123456789
```

## Paso 4: Probar la Configuración

1. Guarda el archivo `.env`
2. Ejecuta el bot:
   ```bash
   cargo run
   ```
3. Deberías ver en la consola:
   ```
   📱 Telegram Notifier: ACTIVADO
      • Chat ID: 123456789
   ```

## Tipos de Notificaciones

El bot enviará notificaciones en estos casos:

### 🚨 Stop-Loss Activado
```
🚨 ALERTA DE STOP-LOSS 🚨

🪙 Token: ICEBEAR
📉 Precio Actual: $0.00028435
📊 Precio Entrada: $0.00056870
📉 Drawdown: -50.02%
🛑 Límite SL: -50.0%

⚡ ACCIÓN REQUERIDA
👉 [Abrir Jupiter para vender](https://jup.ag/swap/...)

⏰ 2026-02-08 10:30:45 UTC
```

### ✅ Venta Automática Ejecutada
```
✅ VENTA AUTOMÁTICA EJECUTADA

🪙 Token: ICEBEAR
💰 Precio: $0.00028435
💵 Cantidad: ~0.051 SOL

⏰ 2026-02-08 10:30:45 UTC
```

### ❌ Error Crítico
```
❌ ERROR CRÍTICO

Error obteniendo precio de ICEBEAR: Network timeout

⏰ 2026-02-08 10:30:45 UTC
```

## Solución de Problemas

### ❌ "Telegram Notifier: DESACTIVADO"
- Verifica que hayas añadido `TELEGRAM_BOT_TOKEN` y `TELEGRAM_CHAT_ID` al `.env`
- Asegúrate de que no haya espacios extra en el archivo `.env`

### ❌ "Error enviando mensaje a Telegram"
- Verifica que el token sea correcto (cópialo nuevamente de BotFather)
- Asegúrate de haber enviado al menos un mensaje a tu bot antes

### ❌ "Chat not found"
- El Chat ID debe ser correcto
- Debes iniciar una conversación con el bot (enviar `/start`)

## Desactivar Notificaciones

Si quieres desactivar temporalmente las notificaciones, simplemente deja vacíos los campos en `.env`:

```bash
TELEGRAM_BOT_TOKEN=
TELEGRAM_CHAT_ID=
```

El sistema funcionará normalmente, pero sin enviar notificaciones.

## 🔒 Seguridad

- **NUNCA** compartas tu token de bot
- **NUNCA** hagas commit del archivo `.env` a GitHub
- El archivo `.gitignore` ya está configurado para ignorar `.env`

## Próximos Pasos

Una vez configurado Telegram, podrás:
1. Ver el bot en acción monitoreando ICEBEAR
2. Añadir más tokens al archivo `targets.json`
3. Ajustar los límites de stop-loss dinámicamente

---

**¿Necesitas ayuda?** Revisa los logs del bot para ver mensajes de error detallados.
