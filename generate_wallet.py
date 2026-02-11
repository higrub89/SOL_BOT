#!/usr/bin/env python3
"""
Generador seguro de wallet Solana para el bot
La clave privada se genera directamente en el servidor
"""

import json
import secrets

def generate_solana_wallet():
    """Genera una wallet Solana compatible"""
    try:
        from nacl.signing import SigningKey
        from base58 import b58encode
        
        # Generar keypair
        signing_key = SigningKey.generate()
        private_key = signing_key.encode()
        public_key = signing_key.verify_key.encode()
        
        # Base58 encoding
        private_key_b58 = b58encode(private_key + public_key).decode('utf-8')
        public_key_b58 = b58encode(public_key).decode('utf-8')
        
        # Formato JSON de Solana
        wallet_array = list(private_key + public_key)
        
        return {
            'public_key': public_key_b58,
            'private_key_b58': private_key_b58,
            'wallet_json': wallet_array
        }
        
    except ImportError:
        print("ERROR: Faltan dependencias. Instala con:")
        print("pip3 install pynacl base58")
        return None

if __name__ == "__main__":
    print("🔐 Generando wallet Solana segura...")
    print()
    
    wallet = generate_solana_wallet()
    
    if wallet:
        print("✅ Wallet generada exitosamente!")
        print()
        print("=" * 60)
        print("DIRECCIÓN PÚBLICA (para depositar):")
        print(wallet['public_key'])
        print("=" * 60)
        print()
        print("⚠️  CLAVE PRIVADA (MANTENER SECRETA):")
        print(wallet['private_key_b58'])
        print("=" * 60)
        print()
        
        # Guardar JSON
        with open('/tmp/bot-wallet.json', 'w') as f:
            json.dump(wallet['wallet_json'], f)
        
        print("📁 Archivo guardado en: /tmp/bot-wallet.json")
        print()
        print("🚀 Próximos pasos:")
        print("1. Copia el archivo al servidor")
        print("2. Actualiza WALLET_PRIVATE_KEY en .env")
        print("3. Deposita SOL en la dirección pública")
