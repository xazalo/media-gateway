#!/bin/bash
# Script de verificación post-deploy para Dokploy
# Uso: ./scripts/verify-deploy.sh <url-del-media-gateway>

set -e

if [ -z "$1" ]; then
    echo "❌ Uso: $0 <url-del-media-gateway>"
    echo "   Ejemplo: $0 http://localhost:3004"
    exit 1
fi

BASE_URL="$1"

echo "🔍 Verificando deployment en: $BASE_URL"
echo "========================================="
echo ""

# 1. Health check
echo "1️⃣  Health check..."
if curl -sf "$BASE_URL/health" > /dev/null; then
    echo "   ✅ Health endpoint OK"
else
    echo "   ❌ Health endpoint falló"
    exit 1
fi

# 2. Verificar conexión a DB (via admin endpoint)
echo ""
echo "2️⃣  Verificando conexión a PostgreSQL..."
if curl -sf "$BASE_URL/admin/traffic/stats" > /dev/null 2>&1; then
    echo "   ✅ PostgreSQL conectado"
else
    echo "   ⚠️  No se pudo verificar PostgreSQL (puede requerir auth)"
fi

# 3. Verificar endpoints principales
echo ""
echo "3️⃣  Verificando endpoints principales..."

# Upload (debería dar 401 sin auth)
UPLOAD_STATUS=$(curl -s -o /dev/null -w "%{http_code}" "$BASE_URL/media/upload" -X POST)
if [ "$UPLOAD_STATUS" = "401" ]; then
    echo "   ✅ /media/upload responde (401 sin auth, esperado)"
else
    echo "   ⚠️  /media/upload responde con status inesperado: $UPLOAD_STATUS"
fi

# Download (debería dar 404 sin key válida)
DOWNLOAD_STATUS=$(curl -s -o /dev/null -w "%{http_code}" "$BASE_URL/media/test.jpg")
if [ "$DOWNLOAD_STATUS" = "404" ]; then
    echo "   ✅ /media/{key} responde (404 sin archivo, esperado)"
else
    echo "   ⚠️  /media/{key} responde con status inesperado: $DOWNLOAD_STATUS"
fi

echo ""
echo "========================================="
echo "✅ Verificación completada"
echo ""
echo "📝 Próximos pasos:"
echo "   - Configura las replicas en Dokploy → Settings → Replicas"
echo "   - Verifica los logs en el panel de Dokploy"
echo "   - Prueba un upload real con curl o el frontend"
echo ""
echo "📚 Documentación completa: DEPLOYMENT.md"
