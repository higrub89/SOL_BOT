# 📱 Guía Termius iPhone - Configuración Segura del Bot

## Nueva Wallet
```
Dirección: 82upJikbczYevdT79WSfcnoBRnxvrbcyLJqjK84d1ut2
```

## Configuración de Termius

### Host:
- **Hostname:** `34.186.82.143`
- **Port:** `22`
- **Username:** `higuitaruben`
- **Key:** Importar desde `~/.ssh/gcp_key`

## Comandos para Ejecutar en Termius

### 1. Ir a la carpeta del bot:
```bash
cd ~/bot_trading
```

### 2. Editar configuración:
```bash
nano .env
```

### 3. Cambiar estas líneas:
```bash
WALLET_ADDRESS=82upJikbczYevdT79WSfcnoBRnxvrbcyLJqjK84d1ut2
WALLET_PRIVATE_KEY=<TU_CLAVE_PRIVADA_DE_PHANTOM>
```

Para obtener la clave privada:
- Phantom → Settings → Security & Privacy → Export Private Key
- Copiar la clave
- Pegarla en el .env

### 4. Guardar cambios en nano:
- `Ctrl+O` (Write Out)
- `Enter` (confirmar)
- `Ctrl+X` (Exit)

### 5. Reiniciar el bot:
```bash
docker-compose restart
```

### 6. Verificar logs:
```bash
docker-compose logs --tail=50
```

Busca esta línea:
```
✅ Keypair cargado correctamente para 82upJikbczYevdT79WSfcnoBRnxvrbcyLJqjK84d1ut2
```

### 7. Monitorear en tiempo real:
```bash
docker-compose logs -f
```

(Para salir: `Ctrl+C`)

## Después de Configurar

### Depositar SOL:
```
Enviar a: 82upJikbczYevdT79WSfcnoBRnxvrbcyLJqjK84d1ut2
Cantidad mínima: 0.065-0.07 SOL
Recomendado: 0.10 SOL
```

### Tokens Configurados:
- **WIF:** 0.025 SOL por operación
- **POPCAT:** 0.025 SOL por operación

### Verificar Balance:
https://solscan.io/account/82upJikbczYevdT79WSfcnoBRnxvrbcyLJqjK84d1ut2

## Solución de Problemas

### Si el bot no inicia:
```bash
docker-compose down
docker-compose up -d
docker-compose logs -f
```

### Si hay error de permisos en .env:
```bash
chmod 600 .env
```

### Ver estado del contenedor:
```bash
docker-compose ps
```

## Seguridad ✅

- ✅ Clave privada NUNCA tocó tu PC
- ✅ Solo existe en iPhone → Servidor
- ✅ Máxima seguridad
- ✅ Sin riesgo de malware en PC
