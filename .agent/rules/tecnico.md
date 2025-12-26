---
trigger: always_on
---

RT-001: Arquitectura y Diseño

Patrón de Arquitectura:

- Microservicio independiente
- Cap'n Proto
- Separación clara entre dominio, aplicación e infraestructura

Stack Tecnológico Recomendado:

- Lenguaje: Rust

RT-002: Performance y Velocidad (Ultra-rápido)

Objetivo: Latencia p99 < 5ms, p50 < 1ms
Estrategias:

Caché Multi-nivel:

- L1: Caché en memoria local - TTL: 5min
- L2: Redis - TTL: 30min
- Caché de perfiles impositivos completos
- Caché de reglas de cálculo precalculadas

Optimización de Acceso a Datos:

- Pool de conexiones optimizado
- Queries precompiladas y optimizadas
- Índices en: jurisdiccion, producto, categoria_iva, categoria_ganancias
- Read replicas para queries de solo lectura

Computación:

- Cálculos en memoria sin I/O
- Algoritmos O(1) o O(log n)
- Evitar serializaciones innecesarias
- Uso de estructuras de datos eficientes

RT-003: Tolerancia a Fallos

Circuit Breaker:

- Configuración: 50% error rate, ventana de 10 requests
- Fallback a valores default o caché desactualizada

Retry Policies:

- Retry exponencial: 3 intentos (2ms, 3ms, 4ms)
- Solo para errores transitorios (timeouts)

Timeout Management:

- Timeout de request: 5ms
- Timeout de DB: 5ms
- Timeout de caché externo: 2ms

Degradación Graciosa:

- Si falla caché L2, usar solo L1
- Si falla BD, usar última configuración cacheada
- Respuesta parcial antes que error total

Health Checks:

- Chequeos de dependencias: BD, Redis, servicios externos

RT-004: Escalabilidad

- Stateless: Sin estado en memoria compartido entre instancias
- Horizontal Scaling: Auto-scaling basado en CPU (>70%) y latencia (>3ms p99)
- Load Balancing: Round-robin con health checks
- Capacidad: Mínimo 10,000 RPS por instancia

RT-005: Disponibilidad

- SLA: 99.95% uptime
- Despliegue: Blue-Green
- Disaster Recovery: RPO < 5min, RTO < 15min

RT-006: Observabilidad

Métricas (Prometheus/Micrometer):

- Latencias: p50, p95, p99, p999
- Throughput: RPS
- Error rate por tipo de impuesto
- Cache hit ratio (L1, L2)
- Circuit breaker state

Logs (Estructurados):

- Request ID de trazabilidad
- Nivel INFO para requests exitosos
- ERROR para fallos con stack traces
- Log sampling en alta carga

Tracing (OpenTelemetry/Jaeger):

- Traces completos de request end-to-end
- Spans por: validación, caché, BD, cálculo
- Correlación con logs via trace-id

Dashboards:

- Grafana con alertas en Prometheus
- Alertas: latencia p99 > 5ms, error rate > 1%, disponibilidad < 99.9%

RT-007: Testing

- Unit Tests: >80% cobertura
- Integration Tests: Con testcontainers
- Performance Tests: Goose, 10K RPS
- Chaos Engineering: Simular fallos de dependencias

RT-008: Configuración

- Feature Flags
- Secrets Management: Vault / AWS Secrets Manager
- Environment: dev, staging, prod con configs específicas

Tecnologías Recomendadas

- Rust
- PostgreSQL 
- Redis 
- Prometheus 

RT-009: Documentacion

- Markdown
- Mermaid Chart
- Modelo C4

Consideraciones Adicionales

Compliance: Registrar todos los cálculos para auditoría fiscal
Versionado: Mantener histórico de cambios en reglas impositivas
Documentación: Markdown
CI/CD: Pipeline automatizado con quality gates
Ambiente: Kubernetes para orquestación y auto-scaling