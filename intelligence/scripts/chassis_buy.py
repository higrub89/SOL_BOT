import os
import sys
import base64
import requests
from solana.rpc.api import Client
from solders.keypair import Keypair
from solders.transaction import VersionedTransaction
from solders.message import MessageV0

def load_env():
    env_vars = {}
    # Resolver .env relativo a la raíz del repo (intelligence/../.. -> repo root)
    base_dir = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
    env_path = os.path.join(base_dir, ".env")
    if os.path.exists(env_path):
        with open(env_path, "r") as f:
            for line in f:
                line = line.strip()
                if not line or line.startswith('#'):
                    continue
                if '=' in line:
                    key, value = line.split('=', 1)
                    env_vars[key.strip()] = value.strip()
    return env_vars

def buy_token(mint, amount_sol):
    print(f"🏎️ [CHASSIS EXECUTION] Iniciando motor de compra...")
    env = load_env()
    
    # 1. Configuración
    api_key = env.get("HELIUS_API_KEY")
    priv_key = env.get("WALLET_PRIVATE_KEY")
    
    if not api_key or not priv_key:
        print("❌ Error: Faltan API_KEY o WALLET_PRIVATE_KEY en .env")
        return

    print(f"🔌 Conectando a Helius RPC...")
    rpc_url = f"https://mainnet.helius-rpc.com/?api-key={api_key}"
    client = Client(rpc_url)
    keypair = Keypair.from_base58_string(priv_key)
    
    print(f"🔍 Consultando quote de Jupiter para {amount_sol} SOL...")
    amount_lamports = int(amount_sol * 1_000_000_000)
    
    try:
        quote_url = f"https://quote-api.jup.ag/v6/quote?inputMint=So11111111111111111111111111111111111111112&outputMint={mint}&amount={amount_lamports}&slippageBps=100"
        print(f"🌐 GET {quote_url}")
        res = requests.get(quote_url, timeout=10)
        res.raise_for_status()
        quote_res = res.json()
        
        if "outAmount" not in quote_res:
            print(f"❌ Error en Quote: {quote_res}")
            return

        # 3. Obtener Transacción
        swap_url = "https://quote-api.jup.ag/v6/swap"
        swap_data = {
            "quoteResponse": quote_res,
            "userPublicKey": str(keypair.pubkey()),
            "wrapAndUnwrapSol": True
        }
        swap_res = requests.post(swap_url, json=swap_data).json()
        
        # 4. Firmar y Enviar
        raw_tx = base64.b64decode(swap_res["swapTransaction"])
        tx = VersionedTransaction.from_bytes(raw_tx)
        
        # Firmar
        signature = keypair.sign_message(tx.message)
        signed_tx = VersionedTransaction.populate(tx.message, [signature])
        
        # Enviar
        res = client.send_raw_transaction(bytes(signed_tx))
        print(f"✅ COMPRA EJECUTADA: {res.value}")
        print(f"🔗 https://solscan.io/tx/{res.value}")
        
    except Exception as e:
        print(f"❌ Error crítico: {e}")

if __name__ == "__main__":
    if len(sys.argv) < 3:
        print("Uso: python3 chassis_buy.py <MINT> <SOL>")
    else:
        buy_token(sys.argv[1], float(sys.argv[2]))
