# Deployment con Dokploy (Dockerfile puro)

Este proyecto está preparado para desplegar directamente con Dokploy usando el Dockerfile, sin necesidad de docker-compose. Esto permite aprovechar el escalado horizontal nativo de Dokploy.

## Configuración en Dokploy

### 1. Crear aplicación

En el panel de Dokploy:

1. Ve a tu proyecto
2. Click en "New" → "Application"
3. Selecciona "Dockerfile" como método de deploy
4. Conecta el repositorio `media-gateway`

### 2. Variables de entorno

Configura estas variables en el panel de Dokploy (sección "Environment"):

#### Requeridas

```bash
# Conexión a servicios externos (red clubify-internal)
DATABASE_URL=postgres://clubify:***:5432/media_gateway
REDIS_URL=redis://redis:6379
JWT_SECRET=***

# Hetzner Object Storage
S3_ENDPOINT=***
S3_BUCKET=***
S3_REGION=***
S3_ACCESS_KEY=***
S3_SECRET_KEY=***

# Bunny CDN
BUNNY_CDN_URL=***
```

#### Opcionales (con defaults)

```bash
# Servidor
HOST=0.0.0.0
PORT=3004
RUST_LOG=info

# Límites de tráfico (defaults recomendados)
TRAFFIC_MAX_DAILY_BYTES=107374182400      # 100GB/día
TRAFFIC_MAX_MONTHLY_BYTES=2199023255552   # 2TB/mes
TRAFFIC_MAX_DAILY_DOWNLOADS=50000
TRAFFIC_MAX_USER_DAILY_BYTES=10737418240  # 10GB/día/usuario
TRAFFIC_MAX_USER_DAILY_DOWNLOADS=500
TRAFFIC_MAX_REQUESTS_PER_HOUR=5000

# Video processing (pendiente de implementar)
VIDEO_PROCESSING_ENABLED=false
VIDEO_MAX_UPLOAD_SIZE=524288000
VIDEO_QUALITIES=360p,720p,1080p
```

### 3. Network

Importante: El media-gateway debe estar en la red `clubify-internal` para acceder a:
- `postgres-media:5432` (base de datos dedicada)
- `redis:6379` (Redis compartido)

En Dokploy:
1. Ve a "Settings" → "Network"
2. Añade la red externa `clubify-internal`

### 4. Escalado horizontal

Para escalar:

1. Ve a "Settings" → "Replicas"
2. Activa "Enable Replicas"
3. Configura el número mínimo y máximo de instancias
4. Configura auto-scaling basado en CPU/RAM si lo deseas

Ejemplo recomendado:
- Min replicas: 2
- Max replicas: 5
- CPU threshold: 70%
- Memory threshold: 80%

### 5. Healthcheck

El Dockerfile incluye un healthcheck en `/health`. Dokploy lo usará para:
- Detectar instancias no saludables
- Reemplazarlas automáticamente
- Balancear carga solo entre instancias sanas

## Arquitectura multi-instancia

```
                    ┌─────────────────┐
                    │   Load Balancer │
                    │   (Dokploy)     │
                    └────────┬────────┘
                             │
            ┌────────────────┼────────────────┐
            │                │                │
            ▼                ▼                ▼
    ┌──────────────┐ ┌──────────────┐ ┌──────────────┐
    │ media-gw #1  │ │ media-gw #2  │ │ media-gw #3  │
    │  (stateless) │ │  (stateless) │ │  (stateless) │
    └──────┬───────┘ └──────┬───────┘ └──────┬───────┘
           │                │                │
           └────────────────┼────────────────┘
                            │
            ┌───────────────┼───────────────┐
            │               │               │
            ▼               ▼               ▼
    ┌──────────────┐ ┌──────────────┐ ┌──────────────┐
    │  PostgreSQL  │ │    Redis     │ │     S3       │
    │  (compartido)│ │  (compartido)│ │  (compartido)│
    └──────────────┘ └──────────────┘ └──────────────┘
```

Todos los componentes son **compartidos y stateless** a nivel de aplicación:
- PostgreSQL: tracking persistente compartido
- Redis: tracking en tiempo real compartido
- S3: storage compartido

## Métricas y logs

Con múltiples instancias, los logs se agregan automáticamente en Dokploy. Para métricas:

```bash
# Ver tráfico total (suma de todas las instancias)
GET /admin/traffic/stats

# Las métricas están centralizadas en PostgreSQL/Redis
```

## Consideraciones importantes

### Rate limiting distribuido

El rate limiting actual es **por instancia**. Con N instancias, el límite efectivo es N × límite por instancia. Si necesitas rate limiting global:

1. Usa un API Gateway externo (Kong, Traefik)
2. Implementa rate limiting en Redis con claves compartidas
3. Configura rate limiting en el load balancer de Dokploy

### Uploads concurrentes

Si un usuario sube el mismo archivo desde múltiples tabs/instancias:
- Cada upload genera una key única (timestamp + UUID)
- No hay colisiones
- Todos los uploads son válidos

### Bunny CDN

Bunny CDN cachea automáticamente los archivos. Para invalidar caché:
- Usa nombres de archivo únicos (ya lo hace el sistema)
- Configura TTL apropiado en Bunny

## Migración desde docker-compose

Si ya tenías desplegado con `docker-compose.yml`:

1. Crea la aplicación en Dokploy (Dockerfile)
2. Configura las mismas variables de entorno
3. Conecta a la misma red `clubify-internal`
4. Deploy
5. (Opcional) Desactiva el servicio docker-compose antiguo

La base de datos, Redis y S3 son los mismos, así que no hay migración de datos necesaria.

## Troubleshooting

### Error: "Connection refused" a postgres/redis

- Verifica que la red `clubify-internal` está conectada
- Verifica que los nombres de host sean correctos: `postgres-media`, `redis`

### Error: "401 invalid authentication token"

- Verifica que `JWT_SECRET` sea idéntico al del backend principal

### Instancias no escalan

- Verifica que el healthcheck pasa: `curl http://localhost:3004/health`
- Revisa los logs en Dokploy
- Verifica que no haya errores de conexión a DB/Redis/S3

## Monitoreo recomendado

Para producción con múltiples instancias, considera:

1. **Health endpoint**: Ya implementado (`/health`)
2. **Metrics endpoint**: Añadir Prometheus metrics (pendiente)
3. **Centralized logging**: Dokploy lo provee automáticamente
4. **Alertas**: Configurar alertas en Dokploy para:
   - Instancias caídas
   - Uso de CPU/RAM alto
   - Errores 5xx frecuentes
