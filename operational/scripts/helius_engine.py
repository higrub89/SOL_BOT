#!/usr/bin/env python3
import requests
import json
import os
import time
from datetime import datetime

class HeliusEngine:
    def __init__(self):
        self.log_dir = '/home/ruben/Automatitation/bot_trading/operational/logs/'
        try:
            with open('/home/ruben/Automatitation/bot_trading/operational/.rpc_config', 'r') as f:
                self.rpc_url = f.read().strip()
        except FileNotFoundError:
            print("❌ Error: RPC no configurado. Ejecuta: echo 'URL' > operational/.rpc_config")
            exit(1)
        
        # Check latencia inicial
        self.check_network_health()

    def _log_alert(self, message):
        """Escribe una alerta en el log de sesión más reciente si existe"""
        print(message)
        try:
            logs = [f for f in os.listdir(self.log_dir) if f.startswith('session_')]
            if logs:
                latest_log = os.path.join(self.log_dir, sorted(logs)[-1])
                with open(latest_log, 'a') as f:
                    f.write(f"[{datetime.now().strftime('%H:%M:%S')}] ALERT: {message}\n")
        except Exception:
            pass

    def check_network_health(self):
        """Verifica que la latencia sea óptima para el sniping (< 150ms)"""
        start_time = time.time()
        try:
            # Peticion ligera para medir latencia
            payload = {"jsonrpc": "2.0", "id": 1, "method": "getHealth"}
            requests.post(self.rpc_url, json=payload, timeout=2)
            latency = (time.time() - start_time) * 1000
            
            if latency > 150:
                self._log_alert(f"⚠️ LATENCIA CRÍTICA: {latency:.2f}ms. Terreno no óptimo para sniping.")
            else:
                print(f"✅ Latencia óptima: {latency:.2f}ms")
            return latency
        except Exception as e:
            self._log_alert(f"❌ Error de conexión al nodo: {e}")
            return None

    def get_priority_fee(self):
        """Obtiene la propina necesaria para entrar en el bloque 0"""
        payload = {
            "jsonrpc": "2.0",
            "id": 1,
            "method": "getPriorityFeeEstimate",
            "params": [{"options": {"priorityLevel": "VeryHigh"}}]
        }
        start_time = time.time()
        res = requests.post(self.rpc_url, json=payload)
        latency = (time.time() - start_time) * 1000
        
        if latency > 150:
            self._log_alert(f"⚠️ Latencia de red elevada en Priority Fee API: {latency:.2f}ms")
            
        fee = res.json().get('result', {}).get('priorityFeeEstimate', 0)
        self._log_alert(f"⛽ Priority Fee Calc: {fee} microLamports | Latency: {latency:.2f}ms")
        return fee

    def check_token_safety(self, mint_address):
        """Usa DAS API para ver info rápida del token"""
        payload = {
            "jsonrpc": "2.0",
            "id": "v1",
            "method": "getAsset",
            "params": {"id": mint_address}
        }
        res = requests.post(self.rpc_url, json=payload)
        return res.json().get('result', {})

if __name__ == "__main__":
    engine = HeliusEngine()
    fee = engine.get_priority_fee()
    print(f"🚀 Priority Fee Recomendada: {fee} microLamports")
