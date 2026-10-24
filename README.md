# Media Gateway

Servicio ligero en Rust para gestionar uploads/downloads de medios con control de tráfico integrado.

## Características

- ✅ Upload/download de imágenes y videos
- ✅ Integración con Hetzner Object Storage (S3-compatible)
- ✅ Control de tráfico con límites configurables (daily, monthly, per-user)
- ✅ Tracking en PostgreSQL + Redis
- ✅ URLs de Bunny.net CDN
- ✅ Autenticación JWT compatible con backend principal
- ✅ Rate limiting por IP
- ✅ Estadísticas de uso

## Estructura

```
media-gateway/
├── src/
│   ├── main.rs              # Entry point
│   ├── lib.rs               # Librería para tests
│   ├── auth.rs              # JWT auth (compatible con backend)
│   ├── config.rs            # Configuración desde .env
│   ├── db.rs                # PostgreSQL (tracking persistente)
│   ├── redis.rs             # Redis (tracking en tiempo real)
│   ├── middleware.rs         # Rate limiting + client IP extraction
│   ├── storage.rs           # Cliente S3 (Hetzner Object Storage)
│   ├── traffic.rs           # Sistema de límites de tráfico
│   └── routes/
│       ├── mod.rs
│       ├── upload.rs        # POST /media/upload (auth required)
│       ├── download.rs      # GET /media/{key} (optional auth)
│       └── admin.rs         # GET /admin/traffic/stats
├── sql/
│   └── 001_traffic_tracking.sql  # Migraciones
├── tests/
│   └── config_test.rs       # Tests de configuración
├── Cargo.toml
├── Dockerfile
├── docker-compose.yml
├── .env.example
├── init-db.sh               # Inicializa DB separada
├── migrate.sh               # Migra archivos existentes
└── README.md
```

## Configuración

### Base de Datos Separada

El Media Gateway usa una base de datos **separada** (`media_gateway`) para no cargar la base de datos principal del backend.

```bash
./init-db.sh
```

### Variables de Entorno

Copia `.env.example` a `.env` y configura:

```bash
# Servidor
HOST=0.0.0.0
PORT=3003

# PostgreSQL (DB separada)
DATABASE_URL=postgres://clubify:password@localhost:5432/media_gateway

# Redis
REDIS_URL=redis://localhost:6379

# JWT (DEBE coincidir con el backend principal)
JWT_SECRET=***

# Hetzner Object Storage
S3_ENDPOINT=***
S3_BUCKET=***
S3_REGION=***
S3_ACCESS_KEY=***
S3_SECRET_KEY=***

# Bunny.net CDN
BUNNY_CDN_URL=***

# Límites de tráfico
TRAFFIC_MAX_DAILY_BYTES=107374182400      # 100GB
TRAFFIC_MAX_MONTHLY_BYTES=2199023255552   # 2TB
TRAFFIC_MAX_DAILY_DOWNLOADS=50000
TRAFFIC_MAX_USER_DAILY_BYTES=10737418240  # 10GB/usuario
TRAFFIC_MAX_USER_DAILY_DOWNLOADS=500
TRAFFIC_MAX_REQUESTS_PER_HOUR=5000
```

### Frontend

En el frontend (`frontend/web/.env`), añade:

```bash
NUXT_MEDIA_GATEWAY_URL=http://localhost:3003
NUXT_PUBLIC_BUNNY_CDN_URL=***
```

## Ejecutar

### Desarrollo

```bash
# 1. Inicializar base de datos
./init-db.sh

# 2. Configurar .env
cp .env.example .env
# Editar .env con credenciales reales

# 3. Compilar y ejecutar
cargo run
```

### Producción (Docker)

```bash
# Construir imagen
docker build -t media-gateway .

# Ejecutar
docker compose up -d
```

## API

### Upload (requiere auth)

```bash
curl -X POST http://localhost:3003/media/upload \
  -H "Authorization: Bearer <jwt-token>" \
  -F "file=@image.jpg"
```

**Respuesta:**
```json
{
  "url": "https://cdn.b-cdn.net/uploads/123/uuid-image.jpg",
  "cdn_url": "https://cdn.b-cdn.net/uploads/123/uuid-image.jpg",
  "key": "uploads/123/uuid-image.jpg",
  "size": 123456,
  "mime_type": "image/jpeg",
  "filename": "image.jpg"
}
```

### Download (auth opcional)

```bash
curl http://localhost:3003/media/uploads/123/uuid-image.jpg
```

### Info

```bash
curl http://localhost:3003/media/uploads/123/uuid-image.jpg/info
```

### Admin Stats

```bash
curl http://localhost:3003/admin/traffic/stats
```

## Sistema de Límites

| Umbral | Acción | Descripción |
|--------|--------|-------------|
| < 75%  | Allow  | Todo normal |
| 75%    | Warning | Notificar admin |
| 90%    | Throttle | Reducir velocidad |
| 95%    | BlockNew | Bloquear nuevas descargas |
| 100%   | BlockAll | Bloquear todo |

## Flujo de Subida

```
Browser → Nitro (/api/media/upload)
       → Media Gateway (/media/upload) [con JWT]
       → Object Storage
       → URL de Bunny CDN devuelta
       → Browser registra URL en backend Rust (/me/media)
```

## Tests

```bash
cargo test
```

## Migración de Archivos Existentes

```bash
# Ver qué haría
./migrate.sh --dry-run

# Ejecutar migración
./migrate.sh
```

## Endpoints

| Método | Ruta | Auth | Descripción |
|--------|------|------|-------------|
| GET | /health | No | Health check |
| POST | /media/upload | Sí (JWT) | Subir archivo |
| GET | /media/{key} | No | Descargar archivo |
| GET | /media/{key}/info | No | Info de archivo |
| GET | /admin/traffic/stats | No | Estadísticas de tráfico |
| POST | /admin/traffic/reset | No | Reset contadores (test) |

## License

Privado - Clubify 2026
