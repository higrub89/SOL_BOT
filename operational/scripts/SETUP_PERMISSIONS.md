# Configuración de Permisos en GCP - The Chassis

Para que el bot pueda acceder a los secretos en Secret Manager desde la instancia GCE, debes configurar correctamente la Cuenta de Servicio.

## 1. Identificar la Cuenta de Servicio
Por defecto, las instancias GCE usan la cuenta:
`compute@developer.gserviceaccount.com` (o la cuenta de servicio predeterminada de Compute Engine).

## 2. Asignar el Rol de Accessor
Ejecuta el siguiente comando en tu terminal local para dar permiso de lectura de secretos a esa cuenta:

```bash
PROJECT_ID="project-828d4ae0-6385-40d2-aa6"
# Obtener el número del proyecto automáticamente
PROJECT_NUMBER=$(gcloud projects describe ${PROJECT_ID} --format="value(projectNumber)")
SERVICE_ACCOUNT="${PROJECT_NUMBER}-compute@developer.gserviceaccount.com"

# Asignar el rol de Secret Manager Secret Accessor
gcloud projects add-iam-policy-binding ${PROJECT_ID} \
    --member="serviceAccount:${SERVICE_ACCOUNT}" \
    --role="roles/secretmanager.secretAccessor"
```

## 3. Verificar los Nombres de los Secretos
Asegúrate de que los nombres de los secretos en Secret Manager coincidan con los esperados por el bot:

- `WALLET_PRIVATE_KEY` (Obligatorio - Clave privada)
- `HELIUS_API_KEY` (Obligatorio)
- `TELEGRAM_BOT_TOKEN` (Obligatorio)
- `TELEGRAM_CHAT_ID` (Obligatorio)

*Nota: El bot prioriza las variables de entorno si están presentes. Si no lo están, intentará recuperarlas de Secret Manager usando el nombre de la variable como nombre del secreto.*
