#!/bin/bash

# Script para inicializar la base de datos del Media Gateway

set -e

DB_NAME="media_gateway"
DB_USER="clubify"
DB_HOST="localhost"
DB_PORT="5432"

echo "🗄️  Inicializando base de datos para Media Gateway..."

# Verificar que PostgreSQL esté corriendo
if ! pg_isready -h $DB_HOST -p $DB_PORT > /dev/null 2>&1; then
    echo "❌ PostgreSQL no está corriendo en $DB_HOST:$DB_PORT"
    echo "Iniciando con docker-compose del backend..."
    cd ../backend && docker compose up -d
    cd ../media-gateway
    sleep 3
fi

# Crear base de datos si no existe
echo "📊 Creando base de datos '$DB_NAME'..."
docker compose exec postgres psql -U $DB_USER -tc "SELECT 1 FROM pg_database WHERE datname = '$DB_NAME'" | grep -q 1 || \
    docker compose exec postgres psql -U $DB_USER -c "CREATE DATABASE $DB_NAME"

# Aplicar migraciones
echo "🔧 Aplicando migraciones..."
export DATABASE_URL="postgres://$DB_USER:password@$DB_HOST:$DB_PORT/$DB_NAME"

# Ejecutar migraciones usando cargo run (el servicio las aplica automáticamente)
# O podemos ejecutar el SQL directamente
docker compose exec postgres psql -U $DB_USER -d $DB_NAME -f sql/001_traffic_tracking.sql

echo "✅ Base de datos inicializada correctamente"
echo "📝 DATABASE_URL=$DATABASE_URL"
