# 🚀 GUÍA DE INICIO RÁPIDO

## Paso 1: Preparar tu Entorno (5 minutos)

### A. Ejecutar el script de inicialización
```bash
cd /home/ruben/Automatitation/bot_trading
./operational/scripts/trading_session.sh
```

Este script:
- ✅ Verifica la estructura de directorios
- ✅ Crea el log de la sesión
- ✅ Genera el template de auditoría
- ✅ Muestra el checklist pre-operación

---

## Paso 2: Configurar RPC Privado (10 minutos)

### A. Registrarse en Helius
1. Ve a: https://www.helius.dev/
2. Crea una cuenta (Plan Free es suficiente para empezar)
3. En el dashboard, copia tu **HTTPS RPC URL**

### B. Guardar el RPC en el proyecto
```bash
echo 'TU_RPC_URL_AQUI' > /home/ruben/Automatitation/bot_trading/operational/.rpc_config
```

**Ejemplo:**
```bash
echo 'https://mainnet.helius-rpc.com/?api-key=abc123xyz' > /home/ruben/Automatitation/bot_trading/operational/.rpc_config
```

---

## Paso 3: Configurar Trojan Bot (15 minutos)

### A. Acceder al bot oficial
1. Abre **Telegram Desktop** en Linux
2. Accede SOLO desde este enlace verificado: https://t.me/solana_trojanbot
3. Envía `/start`

### B. Generar tu Burner Wallet
1. El bot generará automáticamente una wallet
2. Envía `/settings` → `Wallets` → `Export Private Key`
3. **IMPORTANTE:** Copia la clave y guárdala en **KeePassXC**
4. **BORRA** el mensaje de Telegram inmediatamente

### C. Configurar parámetros de trading
En `/settings`, ajusta:

| Parámetro | Valor | Razón |
|-----------|-------|-------|
| **Slippage** | 25% | Equilibrio entre entrada exitosa y precio |
| **Priority Fee** | 0.005 SOL | Superar a traders manuales |
| **Jito Tip** | ON (0.001 SOL) | Protección anti-MEV |
| **Auto-Buy** | OFF | Control manual de cada entrada |
| **Confirmation** | OFF | Velocidad crítica |

### D. Configurar el RPC en Trojan
1. Ve a `/settings` → `RPC Settings`
2. Pega tu URL de Helius
3. El bot confirmará la conexión

---

## Paso 4: Fondear tu Burner Wallet (5 minutos)

### A. Obtener la dirección pública
En el chat de Trojan, envía `/wallet` o `/balance`

### B. Enviar SOL desde tu Trading Wallet
1. Desde Phantom/Solflare, envía **solo 1-2 SOL**
2. **NUNCA** envíes todo tu capital de una vez

### C. Verificar el balance
En tu terminal:
```bash
cd /home/ruben/Automatitation/bot_trading
python3 operational/scripts/wallet_monitor.py TU_DIRECCION_DE_WALLET
```

---

## Paso 5: Hacer tu Primera Operación (20 minutos)

### A. Preparar el entorno
1. Abre **RugCheck.xyz** en tu navegador: https://rugcheck.xyz
2. Abre **Dexscreener**: https://dexscreener.com/solana
    ```bash
    nano operational/audits/audit_template_YYYYMMDD.md
    ```
4. **Verificar Salud de Red (Quirúrgico):**
   ```bash
   python3 operational/scripts/helius_engine.py
   ```
   *   Si la latencia es **> 150ms**, aborta el sniping. El terreno no es óptimo para competir.

### B. Buscar un token candidato
En Dexscreener, filtra por:
- **Liquidez:** > $10,000
- **Creado:** < 1 hora
- **Volume 5m:** Creciente

### C. Auditar el contrato
1. Copia el **Contract Address (CA)**
2. Pégalo en RugCheck.xyz
3. Completa el checklist de auditoría:
   - [ ] LP Burned (100%): ✅
   - [ ] Mint Authority Disabled: ✅
   - [ ] Top 10 Holders < 15%: ✅
   - [ ] RugCheck Score > 85: ✅

### D. Ejecutar la compra
Si **todos** los checks pasan:

1. En Trojan, pega el CA
2. El bot te mostrará:
   - Precio actual
   - Liquidez
   - Holders
3. Selecciona **Buy SOL Amount** → `0.5 SOL` (o menos para tu primera operación)
4. Confirma la transacción

### E. Configurar Take Profits
1. Inmediatamente después de comprar, ve a `/positions`
2. Selecciona el token
3. Configura:
   - **TP1 (100%):** Vender 50% al 2X
   - **TP2 (500%):** Vender 25% al 5X
   - **SL (-30%):** Vender todo si cae 30%

---

## Paso 6: Monitorear y Cerrar la Sesión

### A. Durante la operación
- Revisa `/positions` cada 10-15 minutos
- Monitorea con `wallet_monitor.py`
- Mantén RugCheck abierto para verificar nuevos holders

### B. Al finalizar el día
1. Cierra todas las posiciones abiertas
2. Transfiere las ganancias a tu Trading Wallet
3. Deja **solo 0.1-0.2 SOL** en la burner wallet
4. Actualiza el log de sesión:
   ```bash
   nano operational/logs/session_YYYYMMDD_HHMMSS.log
   ```
5. Completa la sección "Resultado Final" en el template de auditoría

---

## ⚠️ Reglas de Oro (NUNCA romper)

1. **NUNCA** operes sin completar la auditoría completa
2. **SIEMPRE** vende el 50% al 2X (recuperar principal)
3. **NUNCA** dejes más de 2 SOL en la burner wallet
4. **SIEMPRE** usa Stop Loss al -30%
5. **NUNCA** persigas un token que ya hizo 5X+
6. **SIEMPRE** exporta y guarda las claves privadas

---

## 🆘 Troubleshooting

### "Transaction failed"
- **Causa:** Slippage muy bajo o priority fee insuficiente
- **Solución:** Aumenta slippage a 30-40% y priority fee a 0.01 SOL

### "Insufficient SOL for rent"
- **Causa:** Balance muy bajo
- **Solución:** Mantén siempre al menos 0.05 SOL extra para fees

### "Token not showing in /positions"
- **Causa:** La transacción está pendiente
- **Solución:** Espera 30 segundos y revisa en Solscan

---

## 📚 Recursos Rápidos

- **Trojan Bot:** https://t.me/solana_trojanbot
- **RugCheck:** https://rugcheck.xyz
- **Dexscreener:** https://dexscreener.com/solana
- **Helius Dashboard:** https://dashboard.helius.dev/
- **Solscan:** https://solscan.io/

---

## ✅ Checklist Final

Antes de comenzar, asegúrate:
- [ ] Script de sesión ejecutado
- [ ] RPC privado configurado
- [ ] Trojan Bot configurado con los parámetros correctos
- [ ] Burner wallet fondeada (1-2 SOL)
- [ ] RugCheck y Dexscreener abiertos
- [ ] Template de auditoría listo
- [ ] Salud de red verificada (< 150ms)
- [ ] KeePassXC instalado y configurado

---

**¡Estás listo para cazar tu primer 10X!** 🚀

Recuerda: La disciplina es más importante que la suerte. Sigue el protocolo y protege tu capital.

---

**Última Actualización:** 2026-02-04  
**Versión:** 1.0
