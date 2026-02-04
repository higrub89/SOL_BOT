# 🔐 GESTIÓN DE WALLETS - PROTOCOLO DE SEGURIDAD

## ⚠️ ADVERTENCIA CRÍTICA

**NUNCA** comitees este directorio a Git. 
**NUNCA** compartas estas claves con nadie.
**NUNCA** importes claves en servicios no verificados.

---

## 📋 Estructura de Wallets

### 1. Main Wallet (Cold Storage)
- **Propósito:** 95% de la cartera total
- **Tipo:** Hardware Wallet (Ledger/Trezor) o Cold Wallet
- **Contenido:** BTC (60%), ETH (20%), SOL (20%)
- **Acceso:** Solo para rebalanceos mensuales

### 2. Trading Wallet (Hot Wallet)
- **Propósito:** 5% de la cartera (capital de riesgo)
- **Tipo:** Phantom/Solflare
- **Contenido:** SOL para operaciones rápidas
- **Acceso:** Semanal (fondeo de burner wallets)

### 3. Burner Wallets (Bot Wallets)
- **Propósito:** 10% del Trading Wallet (operativa diaria)
- **Tipo:** Generada por Trojan Bot
- **Contenido:** Solo el capital del día (máx 1-2 SOL)
- **Acceso:** Diario (crear nueva cada semana)

---

## 🛡️ Protocolo de Exportación de Claves

Cuando generes una wallet en Trojan:

1. Ve a `/settings` → `Wallets` → `Export Private Key`
2. Copia la clave privada
3. Ábrela en **KeePassXC** (gestor de contraseñas offline)
4. Guarda como: `burner_wallet_YYYYMMDD`
5. **BORRA** el mensaje de Telegram inmediatamente

---

## 📝 Registro de Wallets

Mantén un registro (NUNCA comitear a Git):

```
Wallet_1_Burner_20260204:
  - Address: [DIRECCIÓN PÚBLICA]
  - Created: 2026-02-04
  - Purpose: Trading session week 5
  - Status: Active
  - Balance: 1.5 SOL
  
Wallet_2_Burner_20260211:
  - Address: [DIRECCIÓN PÚBLICA]
  - Created: 2026-02-11
  - Purpose: Trading session week 6
  - Status: Pending
  - Balance: 0 SOL
```

---

## 🔄 Rotación de Wallets

**Cada 7 días:**
1. Transfiere ganancias de la burner wallet a la Trading Wallet
2. Genera una nueva burner wallet en Trojan
3. Exporta la clave privada
4. Archiva la wallet antigua (opcional: mantenerla para auditoría)

---

## 🚨 En Caso de Compromiso

Si sospechas que una wallet fue comprometida:

1. **INMEDIATAMENTE** transfiere todos los fondos a una wallet limpia
2. Genera una nueva burner wallet
3. Cambia todas las contraseñas relacionadas
4. Revisa el historial de transacciones en Solscan

---

## ✅ Checklist de Seguridad

Antes de cada sesión:
- [ ] Verificar que la burner wallet solo tiene el capital del día
- [ ] Confirmar que la clave privada está en KeePassXC
- [ ] Revisar que no hay transacciones no autorizadas
- [ ] Asegurar que 2FA está activo en Telegram

---

**Última Actualización:** 2026-02-04  
**Responsable:** Ruben
