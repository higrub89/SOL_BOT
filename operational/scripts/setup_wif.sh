#!/bin/bash
# ╔═══════════════════════════════════════════════════════════════╗
# ║        SETUP WIF — Workload Identity Federation               ║
# ║        Configuración para GitHub Actions -> GCP               ║
# ╚═══════════════════════════════════════════════════════════════╝

set -e

PROJECT_ID="project-828d4ae0-6385-40d2-aa6"
POOL_NAME="github-pool"
PROVIDER_NAME="github-provider"
REPO="higrub89/SOL_BOT" # Ajustar si el repo es otro
SA_NAME="github-actions-deploy"
SA_EMAIL="${SA_NAME}@${PROJECT_ID}.iam.gserviceaccount.com"

echo "🚀 Iniciando configuración de Workload Identity Federation..."

# 1. Crear el Pool de Identidad si no existe
if ! gcloud iam workload-identity-pools describe "$POOL_NAME" --location="global" --project="$PROJECT_ID" &>/dev/null; then
    gcloud iam workload-identity-pools create "$POOL_NAME" \
        --project="$PROJECT_ID" \
        --location="global" \
        --display-name="GitHub Actions Pool"
    echo "✅ Pool creado: $POOL_NAME"
else
    echo "ℹ️  Pool ya existe: $POOL_NAME"
fi

# 2. Crear el Provider si no existe
if ! gcloud iam workload-identity-pools providers describe "$PROVIDER_NAME" \
    --location="global" --workload-identity-pool="$POOL_NAME" --project="$PROJECT_ID" &>/dev/null; then
    
    gcloud iam workload-identity-pools providers create-oidc "$PROVIDER_NAME" \
        --project="$PROJECT_ID" \
        --location="global" \
        --workload-identity-pool="$POOL_NAME" \
        --display-name="GitHub Actions Provider" \
        --attribute-mapping="google.subject=assertion.sub,attribute.actor=assertion.actor,attribute.repository=assertion.repository" \
        --issuer-uri="https://token.actions.githubusercontent.com"
    echo "✅ Provider creado: $PROVIDER_NAME"
else
    echo "ℹ️  Provider ya existe: $PROVIDER_NAME"
fi

# 3. Vincular el SA con el Provider
echo "🔗 Vinculando Service Account con el repo GitHub..."
gcloud iam service-accounts add-iam-policy-binding "$SA_EMAIL" \
    --project="$PROJECT_ID" \
    --role="roles/iam.workloadIdentityUser" \
    --member="principalSet://iam.googleapis.com/projects/$(gcloud projects describe $PROJECT_ID --format='value(projectNumber)')/locations/global/workloadIdentityPools/$POOL_NAME/attribute.repository/$REPO"

# 4. Obtener el Provider Name para GitHub Secrets
WIF_FULL_NAME=$(gcloud iam workload-identity-pools providers describe "$PROVIDER_NAME" \
    --location="global" --workload-identity-pool="$POOL_NAME" \
    --project="$PROJECT_ID" --format='value(name)')

echo ""
echo "╔══════════════════════════════════════════════════════════╗"
echo "║          ✅ CONFIGURACIÓN COMPLETADA                     ║"
echo "╚══════════════════════════════════════════════════════════╝"
echo ""
echo "Añade estos secretos a tu repositorio GitHub ($REPO):"
echo ""
echo "1. WIF_PROVIDER_NAME: $WIF_FULL_NAME"
echo "2. GCP_SA_EMAIL:      $SA_EMAIL"
echo ""
echo "Configuración lista para despliegue automático. ⚡"
