# ⚠️ ADVERTENCIA: Rate Limiting con Múltiples Instancias

## ⚠️ IMPORTANTE

**El rate limiting actual es POR INSTANCIA, no global.**

Cuando ejecutas múltiples replicas/instancias del media-gateway, cada instancia tiene su propio contador de rate limiting independiente.

---

## ¿Qué significa esto?

### Comportamiento actual

Si configuras `TRAFFIC_MAX_REQUESTS_PER_HOUR=5000` y tienes:

| Instancias | Límite efectivo total |
|------------|----------------------|
| 1 | 5,000 requests/hora |
| 2 | 10,000 requests/hora |
| 3 | 15,000 requests/hora |
| 5 | 25,000 requests/hora |

### Ejemplo práctico

Un usuario malintencionado puede:
- Enviar 5,000 requests a la **instancia 1** (bloqueado)
- Enviar 5,000 requests a la **instancia 2** (bloqueado)
- Enviar 5,000 requests a la **instancia 3** (bloqueado)
- **Total: 15,000 requests con 3 instancias**

El load balancer distribuye las peticiones, así que un usuario podría superar el límite si sus requests se distribuyen entre múltiples instancias.

---

## Lo mismo aplica para otros límites

| Límite | Comportamiento |
|--------|----------------|
| `TRAFFIC_MAX_DAILY_BYTES` | Por instancia |
| `TRAFFIC_MAX_DAILY_DOWNLOADS` | Por instancia |
| `TRAFFIC_MAX_USER_DAILY_BYTES` | Por instancia |
| `TRAFFIC_MAX_USER_DAILY_DOWNLOADS` | Por instancia |
| `TRAFFIC_MAX_REQUESTS_PER_HOUR` | Por instancia |

---

## ¿Cómo solucionarlo?

### Opción 1: Rate Limiting en Redis (Recomendado)

Implementar contadores compartidos en Redis usando claves globales. Ya tienes Redis compartido, solo falta implementar la lógica distribuida.

**Ventajas:**
- Rate limit global real
- Todas las instancias comparten el mismo contador
- Ya tienes la infraestructura (Redis compartido)

**Desventajas:**
- Requiere modificar el código
- Latencia adicional (consulta a Redis en cada request)

### Opción 2: API Gateway / Load Balancer

Usar un proxy reverso con rate limiting integrado:
- **Kong Gateway**
- **Traefik** (integrado con Dokploy)
- **NGINX Plus**
- **Cloudflare** (si está delante)

**Ventajas:**
- No requiere cambios en el código
- Rate limit antes de llegar a la aplicación
- Métricas y logs centralizados

**Desventajas:**
- Componente adicional que gestionar
- Configuración separada

### Opción 3: Ajustar límites manualmente

Dividir los límites entre el número de instancias:

```bash
# Si quieres 15,000 requests/hora total con 3 instancias:
TRAFFIC_MAX_REQUESTS_PER_HOUR=5000  # 5000 × 3 = 15,000
```

**Ventajas:**
- Sin cambios de código
- Simple

**Desventajas:**
- No es preciso (el load balancer no distribuye perfectamente)
- Hay que recalcular si cambias el número de replicas
- No funciona bien con auto-scaling

---

## ¿Qué hacer ahora?

### Para desarrollo/testing
El rate limiting por instancia es suficiente. No hacer nada.

### Para producción con tráfico moderado
Considerar **Opción 2** (API Gateway) si ya tienes uno en tu infraestructura.

### Para producción con tráfico alto o crítico
Implementar **Opción 1** (Redis) o **Opción 2** (API Gateway) antes de escalar a múltiples instancias.

---

## Estado actual del código

El rate limiting está implementado en:
- `src/middleware.rs` → función `rate_limit`
- `src/redis.rs` → contadores en Redis (pero por instancia, no global)

Para implementar rate limiting distribuido, habría que:
1. Usar claves Redis globales (sin ID de instancia)
2. Implementar lógica de locking o atomicidad
3. Considerar cache local para reducir latencia

---

## Contacto

Si necesitas implementar rate limiting distribuido, consulta con el equipo de desarrollo antes de escalar a múltiples instancias en producción.

---

**Última actualización:** 2026-11-12  
**Versión del media-gateway:** 1.0  
**Estado:** Rate limiting por instancia (no global)
