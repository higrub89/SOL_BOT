---
name: wallet-ops
description: >
  Use this skill for any wallet, keypair, or vault operation. Covers key
  generation, rotation, encrypted storage, and the operational/wallets/
  directory conventions.
metadata:
  scope:
    - operational/wallets/
  auto_invoke:
    - "Generate a keypair"
    - "Rotate a wallet"
    - "Access a private key"
    - "Manage vault"
---

# Skill: Wallet Operations

## Security Rules (Absolute)

1. **Private keys are NEVER hardcoded** — not in source, not in configs, not in tests
2. **Never commit** `operational/wallets/` to git — confirm `.gitignore` covers `*.json` and `*.key`
3. All vault access requires environment variable: `CHASSIS_VAULT_PASSWORD`
4. Production keypairs use hardware-backed storage when possible

## Key Generation

```bash
# Generate a new Solana keypair
solana-keygen new --outfile operational/wallets/trader_$(date +%Y%m%d).json

# Verify
solana-keygen pubkey operational/wallets/trader_YYYYMMDD.json
```

## Vault Directory Structure

```
operational/wallets/
├── .gitignore          # ← must contain: *.json, *.key, *.pem
├── README.md           # Documents wallet purposes (no keys)
└── {name}_{date}.json  # Encrypted keypair files
```

## Environment Loading

```rust
// Load keypair from env path — never inline
let keypair_path = std::env::var("CHASSIS_KEYPAIR_PATH")
    .expect("CHASSIS_KEYPAIR_PATH must be set");
let keypair = read_keypair_file(&keypair_path)
    .expect("Invalid keypair file");
```

## Rotation Checklist

- [ ] Generate new keypair
- [ ] Transfer minimal SOL for gas
- [ ] Update `CHASSIS_KEYPAIR_PATH` in environment
- [ ] Log rotation in `operational/audits/` with date and pubkey (no private key)
- [ ] Revoke old keypair access from any APIs
