#!/usr/bin/env bash
#
# Script de migración de archivos al Media Gateway.
#
# Este script mueve archivos existentes del bucket S3 antiguo al formato
# esperado por el Media Gateway, y actualiza las URLs en la base de datos
# principal.
#
# Uso:
#   ./migrate.sh --dry-run    # Solo muestra lo que haría
#   ./migrate.sh              # Ejecuta la migración
#
# Requisitos:
#   - aws-cli configurado con las credenciales del bucket antiguo
#   - psql con acceso a la base de datos principal
#   - Las variables de entorno del Media Gateway configuradas
#

set -euo pipefail

DRY_RUN=false
if [[ "${1:-}" == "--dry-run" ]]; then
    DRY_RUN=true
    echo "🔍 DRY RUN — no se harán cambios reales"
fi

source .env 2>/dev/null || { echo "❌ Necesitas un .env configurado"; exit 1; }

if [ -z "${S3_ENDPOINT:-}" ] || [ -z "${S3_BUCKET:-}" ]; then
    echo "❌ Variables S3_ENDPOINT / S3_BUCKET no configuradas en .env"
    exit 1
fi

OLD_BUCKET="${OLD_S3_BUCKET:-${S3_BUCKET}}"
NEW_BUCKET="${S3_BUCKET}"
DB_URL="${DATABASE_URL:-postgres://clubify:password@localhost:5432/clubify}"

echo "📦 Migrando de bucket '$OLD_BUCKET' a '$NEW_BUCKET'"
echo "🗄️  Base de datos: $DB_URL"
echo ""

echo "📋 Listando objetos en el bucket antiguo..."
aws s3 ls "s3://${OLD_BUCKET}/" --recursive --endpoint-url "${S3_ENDPOINT}" > /tmp/objects.txt
TOTAL=$(wc -l < /tmp/objects.txt)
echo "   Encontrados $TOTAL objetos"
echo ""

if [ "$DRY_RUN" = true ]; then
    echo "Primeros 10 objetos:"
    head -10 /tmp/objects.txt
    echo ""
    echo "✅ DRY RUN completado. Ejecuta sin --dry-run para migrar."
    exit 0
fi

echo "🔄 Migrando archivos..."
COUNT=0
while IFS= read -r line; do
    SIZE=$(echo "$line" | awk '{print $3}')
    KEY=$(echo "$line" | awk '{print $4}')

    [ -z "$KEY" ] && continue

    aws s3 cp \
        "s3://${OLD_BUCKET}/${KEY}" \
        "s3://${NEW_BUCKET}/${KEY}" \
        --endpoint-url "${S3_ENDPOINT}" \
        --no-progress 2>/dev/null

    COUNT=$((COUNT + 1))
    if (( COUNT % 100 == 0 )); then
        echo "   Migrados $COUNT / $TOTAL archivos..."
    fi
done < /tmp/objects.txt

echo ""
echo "✅ Migración completada: $COUNT archivos"
echo ""
echo "🗄️  Actualizando URLs en la base de datos..."
echo "   (Esto requiere el helper toCdnUrl del frontend o un UPDATE SQL manual)"
echo ""
echo "   Ejemplo SQL:"
echo "   UPDATE media SET url = REPLACE(url, 'old-bucket-url', 'new-cdn-url')"
echo "   WHERE url LIKE '%old-bucket-url%';"
echo ""
echo "✅ Listo. Verifica que las URLs funcionen antes de eliminar el bucket antiguo."
