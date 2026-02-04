#!/usr/bin/env python3
"""
Wallet Monitor - Terminal-based Solana wallet tracker
Autor: Ruben
Descripción: Monitorea el balance de tu burner wallet en tiempo real
"""

import sys
import json
import requests
from datetime import datetime
from typing import Optional, Dict

# Colores para terminal
class Colors:
    HEADER = '\033[95m'
    BLUE = '\033[94m'
    CYAN = '\033[96m'
    GREEN = '\033[92m'
    YELLOW = '\033[93m'
    RED = '\033[91m'
    ENDC = '\033[0m'
    BOLD = '\033[1m'

def get_sol_balance(wallet_address: str, rpc_url: str = "https://api.mainnet-beta.solana.com") -> Optional[float]:
    """
    Obtiene el balance de SOL de una wallet
    
    Args:
        wallet_address: Dirección pública de la wallet
        rpc_url: URL del RPC endpoint
    
    Returns:
        Balance en SOL o None si hay error
    """
    payload = {
        "jsonrpc": "2.0",
        "id": 1,
        "method": "getBalance",
        "params": [wallet_address]
    }
    
    try:
        response = requests.post(rpc_url, json=payload, timeout=10)
        response.raise_for_status()
        data = response.json()
        
        if "result" in data:
            # Convertir de lamports a SOL (1 SOL = 1,000,000,000 lamports)
            lamports = data["result"]["value"]
            sol = lamports / 1_000_000_000
            return sol
        else:
            print(f"{Colors.RED}Error en respuesta: {data.get('error', 'Unknown error')}{Colors.ENDC}")
            return None
            
    except requests.exceptions.RequestException as e:
        print(f"{Colors.RED}Error de conexión: {e}{Colors.ENDC}")
        return None
    except Exception as e:
        print(f"{Colors.RED}Error inesperado: {e}{Colors.ENDC}")
        return None

def get_sol_price_usd() -> Optional[float]:
    """
    Obtiene el precio actual de SOL en USD desde CoinGecko
    
    Returns:
        Precio en USD o None si hay error
    """
    try:
        response = requests.get(
            "https://api.coingecko.com/api/v3/simple/price?ids=solana&vs_currencies=usd",
            timeout=10
        )
        response.raise_for_status()
        data = response.json()
        return data["solana"]["usd"]
    except:
        return None

def print_wallet_status(wallet_address: str, rpc_url: str = "https://api.mainnet-beta.solana.com"):
    """
    Imprime el estado actual de la wallet con formato profesional
    """
    print(f"\n{Colors.CYAN}{'='*70}{Colors.ENDC}")
    print(f"{Colors.BOLD}{Colors.HEADER}         🔍 WALLET MONITOR - SOLANA TRADING ENGINE 🔍{Colors.ENDC}")
    print(f"{Colors.CYAN}{'='*70}{Colors.ENDC}\n")
    
    # Información básica
    print(f"{Colors.BLUE}Wallet Address:{Colors.ENDC} {wallet_address[:8]}...{wallet_address[-8:]}")
    print(f"{Colors.BLUE}Timestamp:{Colors.ENDC}      {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}")
    print(f"{Colors.BLUE}RPC Endpoint:{Colors.ENDC}   {rpc_url[:40]}...")
    print(f"\n{Colors.YELLOW}Consultando balance...{Colors.ENDC}")
    
    # Obtener balance
    balance = get_sol_balance(wallet_address, rpc_url)
    
    if balance is not None:
        print(f"\n{Colors.GREEN}✓ Balance obtenido exitosamente{Colors.ENDC}\n")
        print(f"{Colors.BOLD}{Colors.CYAN}Balance:{Colors.ENDC} {Colors.BOLD}{balance:.4f} SOL{Colors.ENDC}")
        
        # Obtener precio en USD
        sol_price = get_sol_price_usd()
        if sol_price:
            usd_value = balance * sol_price
            print(f"{Colors.CYAN}Precio SOL:{Colors.ENDC} ${sol_price:.2f}")
            print(f"{Colors.CYAN}Valor Total:{Colors.ENDC} {Colors.BOLD}${usd_value:.2f} USD{Colors.ENDC}")
        
        # Alertas de seguridad
        print(f"\n{Colors.YELLOW}{'─'*70}{Colors.ENDC}")
        if balance > 5.0:
            print(f"{Colors.RED}⚠️  ALERTA: Balance alto detectado (>{5.0} SOL){Colors.ENDC}")
            print(f"{Colors.RED}   Considera mover fondos a tu Trading Wallet principal{Colors.ENDC}")
        elif balance < 0.1:
            print(f"{Colors.YELLOW}⚠️  Advertencia: Balance bajo (<0.1 SOL){Colors.ENDC}")
            print(f"{Colors.YELLOW}   Puede que necesites fondear para continuar operando{Colors.ENDC}")
        else:
            print(f"{Colors.GREEN}✓ Balance dentro del rango operativo normal{Colors.ENDC}")
    else:
        print(f"\n{Colors.RED}✗ No se pudo obtener el balance{Colors.ENDC}")
    
    print(f"\n{Colors.CYAN}{'='*70}{Colors.ENDC}\n")

def main():
    """
    Función principal
    """
    print(f"{Colors.HEADER}Wallet Monitor v1.0{Colors.ENDC}\n")
    
    if len(sys.argv) < 2:
        print(f"{Colors.YELLOW}Uso:{Colors.ENDC} python3 wallet_monitor.py <WALLET_ADDRESS> [RPC_URL]")
        print(f"\nEjemplo:")
        print(f"  python3 wallet_monitor.py 7xKXtg2CW87d97TXJSDpbD5jBkheTqA83TZRuJosgAsU")
        print(f"  python3 wallet_monitor.py 7xKXtg2CW87d97TXJSDpbD5jBkheTqA83TZRuJosgAsU https://mainnet.helius-rpc.com/?api-key=YOUR_KEY")
        sys.exit(1)
    
    wallet_address = sys.argv[1]
    rpc_url = sys.argv[2] if len(sys.argv) > 2 else "https://api.mainnet-beta.solana.com"
    
    # Validación básica de dirección Solana (32-44 caracteres base58)
    if not (32 <= len(wallet_address) <= 44):
        print(f"{Colors.RED}Error: La dirección de wallet no parece válida{Colors.ENDC}")
        sys.exit(1)
    
    print_wallet_status(wallet_address, rpc_url)

if __name__ == "__main__":
    main()
