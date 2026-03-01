import os
import requests
import json
import time

# Colors for output
GREEN = "\033[92m"
RED = "\033[91m"
YELLOW = "\033[93m"
BLUE = "\033[94m"
RESET = "\033[0m"

def load_env(filepath):
    env_vars = {}
    if not os.path.exists(filepath):
        return env_vars
    with open(filepath, 'r') as f:
        for line in f:
            line = line.strip()
            if not line or line.startswith('#'):
                continue
            if '=' in line:
                key, value = line.split('=', 1)
                env_vars[key.strip()] = value.strip()
    return env_vars

def check_helius(api_key):
    print(f"{BLUE}[1/5] Checking Helius RPC ...{RESET}")
    url = f"https://mainnet.helius-rpc.com/?api-key={api_key}"
    payload = {
        "jsonrpc": "2.0",
        "id": 1,
        "method": "getHealth"
    }
    try:
        response = requests.post(url, json=payload, timeout=10)
        if response.status_code == 200:
            result = response.json()
            if result.get("result") == "ok":
                print(f"{GREEN}  ✅ Helius RPC is UP and API Key is valid.{RESET}")
                return True
            else:
                print(f"{YELLOW}  ⚠️ Helius returned unexpected result: {result}{RESET}")
        else:
            print(f"{RED}  ❌ Helius RPC failed with status {response.status_code}{RESET}")
            print(f"     Response: {response.text}")
    except Exception as e:
        print(f"{RED}  ❌ Error connecting to Helius: {e}{RESET}")
    return False

def check_jupiter(api_key):
    print(f"{BLUE}[2/5] Checking Jupiter API ...{RESET}")
    # Jupiter V6 Quote API doesn't strictly require API key for public endpoints but use it if provided
    # Using the same endpoint as in jupiter.rs: https://api.jup.ag/swap/v1/quote
    url = "https://api.jup.ag/swap/v1/quote"
    params = {
        "inputMint": "So11111111111111111111111111111111111111112", # SOL
        "outputMint": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v", # USDC
        "amount": "100000000", # 0.1 SOL
        "slippageBps": 50
    }
    headers = {}
    if api_key:
        headers["x-api-key"] = api_key
    
    try:
        response = requests.get(url, params=params, headers=headers, timeout=10)
        if response.status_code == 200:
            print(f"{GREEN}  ✅ Jupiter API is UP.{RESET}")
            return True
        else:
            print(f"{RED}  ❌ Jupiter API failed with status {response.status_code}{RESET}")
            print(f"     Response: {response.text}")
    except Exception as e:
        print(f"{RED}  ❌ Error connecting to Jupiter: {e}{RESET}")
    return False

def check_telegram(token, chat_id):
    print(f"{BLUE}[3/5] Checking Telegram Bot ...{RESET}")
    if not token or not chat_id:
        print(f"{YELLOW}  ⚠️ Telegram config missing (TOKEN or CHAT_ID). Skipping.{RESET}")
        return False
    
    url = f"https://api.telegram.org/bot{token}/getMe"
    try:
        response = requests.get(url, timeout=10)
        if response.status_code == 200:
            bot_info = response.json().get("result", {})
            print(f"{GREEN}  ✅ Telegram Bot is VALID: @{bot_info.get('username')}{RESET}")
            return True
        else:
            print(f"{RED}  ❌ Telegram API failed with status {response.status_code}{RESET}")
            print(f"     Response: {response.text}")
    except Exception as e:
        print(f"{RED}  ❌ Error connecting to Telegram: {e}{RESET}")
    return False

def check_dexscreener():
    print(f"{BLUE}[4/5] Checking DexScreener API ...{RESET}")
    # Sol-USDC pair on Raydium
    url = "https://api.dexscreener.com/latest/dex/pairs/solana/58o8xfneax38qyhjwptmcztofrgodvzyit73fdwcv9hu"
    try:
        response = requests.get(url, timeout=10)
        if response.status_code == 200:
            print(f"{GREEN}  ✅ DexScreener API is UP.{RESET}")
            return True
        else:
            print(f"{RED}  ❌ DexScreener API failed with status {response.status_code}{RESET}")
    except Exception as e:
        print(f"{RED}  ❌ Error connecting to DexScreener: {e}{RESET}")
    return False

def check_jito():
    print(f"{BLUE}[5/5] Checking Jito Block Engine reachable ...{RESET}")
    url = "https://amsterdam.mainnet.block-engine.jito.wtf/api/v1/bundles"
    # Jito usually returns 405 Method Not Allowed for GET, which is fine for accessibility check
    # or we can send an empty JSON-RPC request
    payload = {
        "jsonrpc": "2.0",
        "id": 1,
        "method": "getTipAccounts",
        "params": []
    }
    try:
        response = requests.post(url, json=payload, timeout=10)
        if response.status_code == 200:
            print(f"{GREEN}  ✅ Jito Block Engine is UP (Amsterdam).{RESET}")
            return True
        else:
            print(f"{RED}  ❌ Jito check failed with status {response.status_code}{RESET}")
    except Exception as e:
        print(f"{RED}  ❌ Error connecting to Jito: {e}{RESET}")
    return False

def main():
    print(f"{YELLOW}=== SOL_BOT API HEALTH CHECK ==={RESET}\n")
    
    env = load_env(".env")
    
    helius_key = env.get("HELIUS_API_KEY")
    jupiter_key = env.get("JUPITER_API_KEY")
    tg_token = env.get("TELEGRAM_BOT_TOKEN")
    tg_chat = env.get("TELEGRAM_CHAT_ID")
    
    status = []
    status.append(check_helius(helius_key))
    status.append(check_jupiter(jupiter_key))
    status.append(check_telegram(tg_token, tg_chat))
    status.append(check_dexscreener())
    status.append(check_jito())
    
    print(f"\n{YELLOW}=== SUMMARY ==={RESET}")
    total = len(status)
    success = sum(1 for s in status if s)
    
    if success == total:
        print(f"{GREEN}All systems operational ({success}/{total}){RESET}")
    else:
        print(f"{RED}Some systems failed ({success}/{total}). Check the logs above.{RESET}")

if __name__ == "__main__":
    main()
