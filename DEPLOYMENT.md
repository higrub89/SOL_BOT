# 🚀 DEPLOYMENT GUIDE - The Chassis Production Server

**Última Actualización:** 2026-02-09  
**Versión del Bot:** v1.1.0-luxury

---

## 🎯 Objetivo
Desplegar **The Chassis** en un servidor VPS para operación 24/7 sin depender de tu laptop personal.

---

## 📋 Requisitos del Servidor

### Especificaciones Mínimas:
- **CPU:** 2 vCores (para WebSocket + Monitor simultáneo)
- **RAM:** 2GB (4GB recomendado para compilación en servidor)
- **Disco:** 20GB SSD
- **OS:** Ubuntu 22.04 LTS o Debian 12
- **Red:** Conexión estable con latencia <200ms a `mainnet.helius-rpc.com`

### Proveedores Recomendados:
1.  **Hetzner Cloud CX21** - €4.51/mes (Alemania) - [Enlace](https://www.hetzner.com/cloud)
2.  **DigitalOcean Droplet** - $6/mes (NYC/SF) - [Enlace](https://www.digitalocean.com/pricing/droplets)
3.  **AWS Lightsail** - $5/mes (us-east-1) - Ultra latencia - [Enlace](https://aws.amazon.com/lightsail/)

---

## 🛠️ Configuración Inicial del Servidor

### Paso 1: Conectar al Servidor
```bash
ssh root@TU_IP_SERVIDOR
```

### Paso 2: Actualizar el Sistema
```bash
apt update && apt upgrade -y
apt install -y build-essential git curl pkg-config libssl-dev
```

### Paso 3: Instalar Rust
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
rustc --version  # Verificar instalación
```

### Paso 4: Instalar Python y Dependencias
```bash
apt install -y python3 python3-pip python3-venv
pip3 install requests solana solders
```

### Paso 5: Configurar Git y Clonar el Proyecto
```bash
git config --global user.name "Tu Nombre"
git config --global user.email "tu@email.com"

# Clonar el repositorio (ajusta la URL según tu setup)
cd /opt
git clone https://github.com/TU_USUARIO/SOL_BOT.git
cd SOL_BOT/core/the_chassis
```

---

## 🔐 Configuración de Variables de Entorno

### Crear el archivo `.env` en el servidor:
```bash
nano /opt/SOL_BOT/core/the_chassis/.env
```

### Contenido mínimo requerido:
```env
HELIUS_API_KEY=tu_api_key_de_helius
WALLET_ADDRESS=tu_wallet_publica
WALLET_PRIVATE_KEY=tu_clave_privada_base58
TELEGRAM_BOT_TOKEN=tu_token_de_telegram
TELEGRAM_CHAT_ID=tu_chat_id
MAX_LATENCY_MS=150
```

**⚠️ SEGURIDAD:** Este archivo contiene tu clave privada. Asegúrate de:
```bash
chmod 600 .env  # Solo el usuario root puede leerlo
```

---

## 🏗️ Compilación en el Servidor

### Compilar en Modo Release (Optimizado):
```bash
cd /opt/SOL_BOT/core/the_chassis
cargo build --release
```

Esto puede tardar 5-10 minutos la primera vez.

### Verificar la Compilación:
```bash
./target/release/the_chassis --help
```

---

## 🚀 Arranque del Bot

### Opción A: Ejecución Manual (Para Testing)
```bash
# Monitor Mode (Vigilancia 24/7)
./target/release/the_chassis

# Scan Mode (Sensor de Telemetría)
./target/release/the_chassis scan
```

### Opción B: Arranque Automático con `systemd` (Recomendado)

#### 1. Crear el Servicio:
```bash
nano /etc/systemd/system/the-chassis.service
```

#### 2. Contenido del Servicio:
```ini
[Unit]
Description=The Chassis - Solana Trading Bot
After=network.target

[Service]
Type=simple
User=root
WorkingDirectory=/opt/SOL_BOT/core/the_chassis
ExecStart=/opt/SOL_BOT/core/the_chassis/target/release/the_chassis
Restart=always
RestartSec=10
StandardOutput=append:/var/log/the-chassis.log
StandardError=append:/var/log/the-chassis-error.log

[Install]
WantedBy=multi-user.target
```

#### 3. Activar el Servicio:
```bash
systemctl daemon-reload
systemctl enable the-chassis
systemctl start the-chassis
```

#### 4. Verificar Estado:
```bash
systemctl status the-chassis
tail -f /var/log/the-chassis.log  # Ver logs en tiempo real
```

---

## 📊 Monitoreo del Bot en Producción

### Ver Logs en Tiempo Real:
```bash
tail -f /var/log/the-chassis.log
```

### Reiniciar el Bot:
```bash
systemctl restart the-chassis
```

### Detener el Bot:
```bash
systemctl stop the-chassis
```

---

## 🔄 Actualización del Bot (Deploy de Nuevas Versiones)

### Script de Actualización Rápida:
```bash
#!/bin/bash
# update_bot.sh

cd /opt/SOL_BOT
git pull origin main
cd core/the_chassis
cargo build --release
systemctl restart the-chassis
echo "✅ Bot actualizado y reiniciado"
```

### Uso:
```bash
chmod +x update_bot.sh
./update_bot.sh
```

---

## 🛡️ Seguridad Crítica

1.  **Firewall:**
    ```bash
    ufw allow 22/tcp  # SSH
    ufw enable
    ```

2.  **Cambiar Puerto SSH (Opcional):**
    ```bash
    nano /etc/ssh/sshd_config
    # Cambiar "Port 22" a "Port 2222"
    systemctl restart ssh
    ```

3.  **Backup de `.env`:**
    ```bash
    cp .env .env.backup
    # Guárdalo en un lugar seguro fuera del servidor
    ```

---

## 🎯 Checklist de Deployment

- [ ] Servidor creado en proveedor (Hetzner/DO/AWS)
- [ ] Rust instalado
- [ ] Python y librerías instaladas
- [ ] Proyecto clonado en `/opt/SOL_BOT`
- [ ] Archivo `.env` configurado con claves reales
- [ ] Compilación en release exitosa
- [ ] Servicio `systemd` creado y habilitado
- [ ] Bot arrancado y verificado en logs
- [ ] Telegram recibiendo notificaciones de prueba

---

## 📞 Soporte y Troubleshooting

### El bot no arranca:
```bash
journalctl -u the-chassis -n 50  # Ver últimos 50 mensajes
```

### Errores de compilación:
```bash
cargo clean
cargo build --release 2>&1 | tee build.log
```

### Problemas de DNS (como en tu laptop):
```bash
echo "nameserver 8.8.8.8" >> /etc/resolv.conf
ping quote-api.jup.ag
```

---

**Desarrollado con ⚡ por Rubén | MV Agusta Mindset**
