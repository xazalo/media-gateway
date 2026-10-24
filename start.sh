#!/bin/bash

# Script de inicio rápido para Media Gateway

set -e

echo "🚀 Iniciando Media Gateway..."

# Verificar que existe .env
if [ ! -f .env ]; then
    echo "❌ No se encontró .env"
    echo "Copiando .env.example a .env..."
    cp .env.example .env
    echo "⚠️  Edita .env con tus credenciales reales"
    exit 1
fi

# Verificar que PostgreSQL esté corriendo
echo "📊 Verificando PostgreSQL..."
if ! pg_isready -h localhost -p 5432 > /dev/null 2>&1; then
    echo "⚠️  PostgreSQL no está corriendo"
    echo "Iniciando con docker-compose del backend principal..."
    cd ../backend && docker compose up -d
    cd ../media-gateway
    sleep 3
fi

# Verificar que Redis esté corriendo
echo "🔴 Verificando Redis..."
if ! redis-cli ping > /dev/null 2>&1; then
    echo "⚠️  Redis no está corriendo"
    echo "Iniciando con docker-compose del backend principal..."
    cd ../backend && docker compose up -d
    cd ../media-gateway
    sleep 2
fi

# Compilar y ejecutar
echo "🔨 Compilando..."
cargo build

echo "✅ Iniciando servicio en http://localhost:3003"
cargo run
