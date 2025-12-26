---
trigger: always_on
---

RF-001: Cálculo de Impuestos

El microservicio debe calcular impuestos (SELLOS, IVA, IVA_PERCEPCION, IIBB_PERCEPCION, GANANCIAS) basándose en:

- Parámetros del Perfil Impositivo (PI)
- Reglas del Módulo Impositivo (MI)
- Datos de la transacción (jurisdicción, producto, categorías, exenciones)


RF-002: Gestión de Perfiles Impositivos (PI)

- Consultar configuración impositiva por cliente/entidad
- Cachear perfiles para acceso ultrarrápido
- Soportar versionado de perfiles para auditoría

RF-003: Aplicación de Reglas por Tipo de Impuesto

- SELLOS: Cálculo por jurisdicción y tipo de producto
- IVA: Aplicar alícuotas según categoría_iva y exenciones
- IVA_PERCEPCION: Calcular percepciones según categoría_iva, considerando general, comisiones/gastos e intereses
-IIBB_PERCEPCION: Aplicar percepciones por jurisdicción
- GANANCIAS: Calcular según categoría_ganancias y exenciones aplicables

RF-004: Gestión de Exenciones

- Validar y aplicar exenciones por tipo de impuesto
- Soportar exenciones totales y parciales
- Registrar exenciones aplicadas para auditoría

RF-005: Respuesta del Cálculo

-Retornar desglose detallado por cada impuesto
-Incluir base imponible, alícuota aplicada, monto calculado
-Proporcionar identificadores de trazabilidad

RF-006: Validaciones

- Validar integridad de datos de entrada
- Verificar existencia de configuraciones requeridas
- Validar rangos y límites según normativa