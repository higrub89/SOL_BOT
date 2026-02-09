#!/usr/bin/env python3
"""
INTELLIGENCE AUTO-AUDIT v1.0
============================
Automated token analysis with report generation.
Fase 1: Aumento de Auditoría Manual.
"""

import requests
import sys
import json
import os
from datetime import datetime

class AutoAudit:
    def __init__(self, audit_dir="operational/audits"):
        self.rugcheck_url = "https://api.rugcheck.xyz/v1/tokens"
        self.dexscreener_url = "https://api.dexscreener.com/latest/dex/tokens"
        self.audit_dir = audit_dir
        
        if not os.path.exists(self.audit_dir):
            os.makedirs(self.audit_dir)
            
    def check_rugcheck(self, contract_address):
        try:
            url = f"{self.rugcheck_url}/{contract_address}/report"
            response = requests.get(url, timeout=10)
            if response.status_code == 200:
                data = response.json()
                return {
                    "score": data.get("score", "N/A"),
                    "risks": data.get("risks", []),
                    "top_holders": data.get("topHolders", [])[:5],
                    "lp_locked": data.get("markets", [{}])[0].get("lp", {}).get("lpLockedPct", 0) if data.get("markets") else 0,
                    "mint_authority": data.get("mintAuthority"),
                    "freeze_authority": data.get("freezeAuthority")
                }
            return {"error": f"HTTP {response.status_code}"}
        except Exception as e:
            return {"error": str(e)}
    
    def check_dexscreener(self, contract_address):
        try:
            url = f"{self.dexscreener_url}/{contract_address}"
            response = requests.get(url, timeout=10)
            if response.status_code == 200:
                data = response.json()
                pairs = data.get("pairs", [])
                if pairs:
                    pair = pairs[0]
                    return {
                        "name": pair.get("baseToken", {}).get("name", "Unknown"),
                        "symbol": pair.get("baseToken", {}).get("symbol", "???"),
                        "price_usd": pair.get("priceUsd", "0"),
                        "liquidity_usd": pair.get("liquidity", {}).get("usd", 0),
                        "volume_24h": pair.get("volume", {}).get("h24", 0),
                        "fdv": pair.get("fdv", 0),
                        "pair_created": pair.get("pairCreatedAt", 0)
                    }
            return {"error": "No pairs found"}
        except Exception as e:
            return {"error": str(e)}

    def generate_report(self, contract, rug_data, dex_data):
        timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
        symbol = dex_data.get("symbol", "UNKNOWN").replace("$", "")
        filename = f"audit_{symbol}_{timestamp}.md"
        filepath = os.path.join(self.audit_dir, filename)
        
        score = rug_data.get("score", 0)
        verdict = "🔴 RECHAZADO"
        if score == "Good" or (isinstance(score, (int, float)) and score < 500):
            verdict = "🟢 APROBADO"
        elif isinstance(score, (int, float)) and score < 1500:
            verdict = "🟡 RIESGO MEDIO"

        report_content = f"""# 🔍 Informe de Auditoría Inteligente: {symbol}
        
**Fecha:** {datetime.now().strftime("%Y-%m-%d %H:%M:%S")}
**Contrato:** `{contract}`
**Veredicto:** {verdict}

---

## 📊 Métricas de Mercado
- **Nombre:** {dex_data.get('name')}
- **Precio:** ${dex_data.get('price_usd')}
- **Liquidez:** ${dex_data.get('liquidity_usd', 0):,.0f}
- **Volumen 24h:** ${dex_data.get('volume_24h', 0):,.0f}
- **FDV:** ${dex_data.get('fdv', 0):,.0f}

## 🛡️ Análisis de Seguridad (RugCheck)
- **Score:** {score}
- **LP Bloqueada:** {rug_data.get('lp_locked', 0)}%
- **Mint Authority:** {"⚠️ ACTIVA" if rug_data.get('mint_authority') else "✅ RENUNCIADA"}
- **Freeze Authority:** {"⚠️ ACTIVA" if rug_data.get('freeze_authority') else "✅ RENUNCIADA"}

### Riesgos Detectados:
"""
        for risk in rug_data.get("risks", []):
            level = "🚨" if risk.get("level") == "danger" else "⚠️"
            report_content += f"- {level} {risk.get('name')}: {risk.get('description', 'Sin descripción')}\n"
            
        report_content += f"""
---
*Informe generado automáticamente por The Chassis Intelligence*
"""
        
        with open(filepath, "w") as f:
            f.write(report_content)
            
        return filepath, verdict

    def run(self, contract):
        print(f" iniciando Auditoría para: {contract}...")
        rug_data = self.check_rugcheck(contract)
        dex_data = self.check_dexscreener(contract)
        
        if "error" in rug_data and "error" in dex_data:
            print(f"❌ Error en auditoría: {rug_data.get('error')} | {dex_data.get('error')}")
            return
            
        filepath, verdict = self.generate_report(contract, rug_data, dex_data)
        
        print(f"\n✅ Auditoría completada!")
        print(f"🎯 Veredicto: {verdict}")
        print(f"📑 Reporte guardado en: {filepath}")
        
        # Mostrar resumen en terminal
        print("\n--- Resumen ---")
        print(f"Token: {dex_data.get('name')} ({dex_data.get('symbol')})")
        print(f"Score: {rug_data.get('score')}")
        print(f"Liq:   ${dex_data.get('liquidity_usd', 0):,.0f}")
        print(f"----------------\n")

if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("Uso: python3 auto_audit.py <CONTRACT_ADDRESS>")
        sys.exit(1)
    
    contract = sys.argv[1]
    audit = AutoAudit()
    audit.run(contract)
