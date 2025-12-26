# Diagrama de Componentes: Tax Engine

Este documento detalla la estructura interna del **Tax Engine** (C4 - Nivel 3), describiendo los componentes lógicos que residen dentro del microservicio y cómo colaboran para procesar los cálculos de impuestos.

```mermaid
C4Component
    title Diagrama de Componentes - Tax Engine (C4 - Nivel 3)

    Container_Boundary(tax_engine, "Tax Engine") {
        Component(api, "Interface Cap'n Proto")
        Component(validator, "Validador de Integridad")
        Component(orchestrator, "Orquestador")
        Component(profile_resolver, "Gestor de Perfiles")
        Component(exemption_engine, "Motor de Exenciones")
        
        Container_Boundary(calculators, "Calculadores Específicos") {
            Component(iva_logic, "Lógica IVA")
            Component(sellos_logic, "Lógica Sellos")
            Component(iibb_logic, "Lógica IIBB")
            Component(ganancias_logic, "Lógica Ganancias")
        }
        
    }

    ContainerDb(cache, "Cache")
    ContainerDb(db, "Base de Datos")
    System_Ext(people, "People Center")

    Rel(api, validator, "")
    Rel(validator, orchestrator, "")
    Rel(orchestrator, profile_resolver, "")
    Rel(profile_resolver, cache, "")
    Rel(profile_resolver, db, "")
    Rel(profile_resolver, people, "")
    
    Rel(orchestrator, exemption_engine, "")
    Rel(orchestrator, iva_logic, "")
    Rel(orchestrator, sellos_logic, "")
    Rel(orchestrator, iibb_logic, "")
    Rel(orchestrator, ganancias_logic, "")
    
    
    UpdateLayoutConfig($c4ShapeInRow="4", $c4BoundaryInRow="2")
```

## Componentes Principales

### 1. Interface Cap'n Proto

Es la frontera de comunicación del microservicio. Implementa el esquema de datos definido para las transacciones bancarias, asegurando una comunicación extremadamente eficiente y con tipado fuerte.

### 2. Validador de Integridad

Asegura que antes de iniciar cualquier cálculo, los datos de la transacción sean consistentes y que las configuraciones requeridas existan en el sistema.

### 3. Gestor de Perfiles

Centraliza la lógica de obtención de la configuración fiscal del cliente. Implementa la estrategia de cache (Redis).

### 4. Orquestador de Cálculo

Es el cerebro del microservicio. Define el orden de ejecución:

1. Obtención del perfil.
2. Identificación de impuestos aplicables según el Módulo Impositivo.
3. Aplicación de exenciones.
4. Ejecución de los calculadores específicos.

### 5. Calculadores Específicos (Domain Logic)

Contienen las reglas de negocio puras para cada tipo de gravamen

- **IVA**: Diferenciación entre general, intereses y comisiones.
- **Sellos**: Lógica basada en la matriz Jurisdicción-Producto.
- **IIBB**: Aplicación de alícuotas por zona geográfica.
- **Ganancias**: Basado en categorías de ganancias.

### 6. Motor de Exenciones

Evalúa certificados de exención cargados en el perfil o en la transacción y determina si el monto imponible debe ser ajustado.


## Diagrama de Secuencia

```mermaid
sequenceDiagram
    title Diagrama de Secuencia de Cálculo de Impuestos

    participant C as Consumidor
    participant API as Interface
    participant VAL as Validador
    participant ORQ as Orquestador
    participant PR as Perfiles
    participant EX as Exenciones
    participant CALC as Calculadores...
    participant DB as Base de Datos

    C->>API: Solicitud Cálculo
    API->>VAL: Validar integridad datos
    VAL-->>API: OK
    API->>ORQ: Iniciar Proceso de Cálculo
    
    ORQ->>PR: Obtener Perfil Impositivo
    PR->>PR: Consultar Cache
    alt Cache Miss
        PR->>DB: Leer Perfil Impositivo
        PR->>PR: Actualizar Cache
    end
    PR-->>ORQ: Perfil Impositivo 
    
    ORQ->>EX: Evaluar Exenciones
    EX-->>ORQ: Lista de Exenciones
    
    loop por cada impuesto
        ORQ->>CALC: Calcular Monto
        CALC-->>ORQ: Resultado Parcial
    end
    
    ORQ-->>API: Respuesta Consolidada
    API-->>C: Respuesta Detallada
```

