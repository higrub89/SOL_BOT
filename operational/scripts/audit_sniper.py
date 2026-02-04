#!/usr/bin/env python3
"""
AUDIT SNIPER - Auditoría Automática de Tokens en Solana
=========================================================
Reduce el tiempo de auditoría de 60 segundos a 3 segundos.
Consulta RugCheck + DexScreener en paralelo.
"""

import requests
import sys
import json
from datetime import datetime

class AuditSniper:
    def __init__(self):
        self.rugcheck_url = "https://api.rugcheck.xyz/v1/tokens"
        self.dexscreener_url = "https://api.dexscreener.com/latest/dex/tokens"
        
    def check_rugcheck(self, contract_address):
        """Consulta RugCheck para verificar seguridad del contrato"""
        try:
            url = f"{self.rugcheck_url}/{contract_address}/report"
            response = requests.get(url, timeout=10)
            
            if response.status_code == 200:
                data = response.json()
                return {
                    "score": data.get("score", "N/A"),
                    "risks": data.get("risks", []),
                    "top_holders": data.get("topHolders", [])[:5],
                    "lp_locked": data.get("markets", [{}])[0].get("lp", {}).get("lpLockedPct", 0) if data.get("markets") else 0
                }
            else:
                return {"error": f"HTTP {response.status_code}"}
        except Exception as e:
            return {"error": str(e)}
    
    def check_dexscreener(self, contract_address):
        """Consulta DexScreener para métricas de mercado"""
        try:
            url = f"{self.dexscreener_url}/{contract_address}"
            response = requests.get(url, timeout=10)
            
            if response.status_code == 200:
                data = response.json()
                pairs = data.get("pairs", [])
                
                if pairs:
                    pair = pairs[0]  # El par con más liquidez
                    return {
                        "name": pair.get("baseToken", {}).get("name", "Unknown"),
                        "symbol": pair.get("baseToken", {}).get("symbol", "???"),
                        "price_usd": pair.get("priceUsd", "0"),
                        "liquidity_usd": pair.get("liquidity", {}).get("usd", 0),
                        "volume_24h": pair.get("volume", {}).get("h24", 0),
                        "price_change_24h": pair.get("priceChange", {}).get("h24", 0),
                        "pair_created": pair.get("pairCreatedAt", 0),
                        "dex": pair.get("dexId", "unknown")
                    }
                else:
                    return {"error": "No pairs found"}
            else:
                return {"error": f"HTTP {response.status_code}"}
        except Exception as e:
            return {"error": str(e)}
    
    def calculate_verdict(self, rugcheck_data, dex_data):
        """Calcula el veredicto final basado en los datos"""
        red_flags = []
        green_flags = []
        
        # Análisis de RugCheck
        if "error" not in rugcheck_data:
            score = rugcheck_data.get("score", 0)
            if score == "Good" or (isinstance(score, (int, float)) and score < 1000):
                green_flags.append(f"RugCheck Score: {score}")
            else:
                red_flags.append(f"RugCheck Score: {score}")
            
            lp_locked = rugcheck_data.get("lp_locked", 0)
            if lp_locked >= 90:
                green_flags.append(f"LP Locked: {lp_locked}%")
            elif lp_locked < 50:
                red_flags.append(f"LP Locked: {lp_locked}% ⚠️")
                
            risks = rugcheck_data.get("risks", [])
            for risk in risks:
                if risk.get("level") == "danger":
                    red_flags.append(f"Risk: {risk.get('name')}")
        else:
            red_flags.append(f"RugCheck Error: {rugcheck_data['error']}")
        
        # Análisis de DexScreener
        if "error" not in dex_data:
            liquidity = dex_data.get("liquidity_usd", 0)
            if liquidity >= 5000:
                green_flags.append(f"Liquidez: ${liquidity:,.0f}")
            else:
                red_flags.append(f"Liquidez baja: ${liquidity:,.0f}")
            
            volume = dex_data.get("volume_24h", 0)
            if volume >= 10000:
                green_flags.append(f"Volumen 24h: ${volume:,.0f}")
        else:
            red_flags.append(f"DexScreener Error: {dex_data['error']}")
        
        # Veredicto Final
        if len(red_flags) == 0 and len(green_flags) >= 3:
            verdict = "🟢 SEGURO - Proceder con cautela"
        elif len(red_flags) <= 1 and len(green_flags) >= 2:
            verdict = "🟡 REVISAR - Evaluar riesgos manualmente"
        else:
            verdict = "🔴 PELIGRO - No operar"
        
        return verdict, green_flags, red_flags
    
    def audit(self, contract_address):
        """Ejecuta auditoría completa"""
        print("\n" + "="*60)
        print("     🔍 AUDIT SNIPER - Análisis de Contrato")
        print("="*60)
        print(f"\n📋 Contrato: {contract_address}")
        print(f"⏱️  Timestamp: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}")
        print("\nConsultando APIs...")
        
        # Consultas paralelas (en secuencia por simplicidad)
        rugcheck_data = self.check_rugcheck(contract_address)
        dex_data = self.check_dexscreener(contract_address)
        
        # Mostrar datos de mercado
        if "error" not in dex_data:
            print(f"\n📊 DATOS DE MERCADO ({dex_data.get('dex', 'DEX')})")
            print(f"   Nombre: {dex_data.get('name')} ({dex_data.get('symbol')})")
            print(f"   Precio: ${dex_data.get('price_usd')}")
            print(f"   Liquidez: ${dex_data.get('liquidity_usd', 0):,.0f}")
            print(f"   Volumen 24h: ${dex_data.get('volume_24h', 0):,.0f}")
            print(f"   Cambio 24h: {dex_data.get('price_change_24h', 0)}%")
        
        # Mostrar datos de seguridad
        if "error" not in rugcheck_data:
            print(f"\n🛡️ DATOS DE SEGURIDAD (RugCheck)")
            print(f"   Score: {rugcheck_data.get('score')}")
            print(f"   LP Locked: {rugcheck_data.get('lp_locked', 0)}%")
            if rugcheck_data.get("risks"):
                print(f"   Riesgos detectados: {len(rugcheck_data['risks'])}")
        
        # Calcular y mostrar veredicto
        verdict, green_flags, red_flags = self.calculate_verdict(rugcheck_data, dex_data)
        
        print(f"\n{'─'*60}")
        print(f"\n🎯 VEREDICTO: {verdict}")
        
        if green_flags:
            print(f"\n✅ Señales Positivas:")
            for flag in green_flags:
                print(f"   • {flag}")
        
        if red_flags:
            print(f"\n❌ Señales de Alerta:")
            for flag in red_flags:
                print(f"   • {flag}")
        
        print("\n" + "="*60 + "\n")
        
        return verdict


if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("Uso: python3 audit_sniper.py <CONTRACT_ADDRESS>")
        print("Ejemplo: python3 audit_sniper.py 2k8yZaJjf61unHriuqdmvbxe7CUhEYML5kVJDbcotKjU")
        sys.exit(1)
    
    contract = sys.argv[1]
    sniper = AuditSniper()
    sniper.audit(contract)
