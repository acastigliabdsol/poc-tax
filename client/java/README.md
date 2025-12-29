# Cliente Java

Este directorio contiene el cliente Java para la aplicación y un submódulo con las utilidades de Cap'n Proto para Java.

## Estructura del directorio

- `mvnw`, `pom.xml` : wrapper y descriptor Maven para construir los artefactos Java.
- `capnproto-java-rpc/` : submódulo que provee el runtime y herramientas Java para Cap'n Proto RPC.
- `tax-client/` : módulo del cliente Java que contiene el código fuente del cliente y scripts de ayuda.
- `run-client.sh` : script auxiliar para ejecutar el cliente (ubicado en la raíz de `client/java`).

## Arquitectura del cliente Java

El cliente Java está dividido en módulos Maven. El módulo `tax-client` contiene la lógica de negocio y las clases generadas a partir de los esquemas Cap'n Proto. El submódulo `capnproto-java-rpc` implementa las utilidades necesarias para usar Cap'n Proto en Java (compilador `capnpc-java`, bindings y runtime para RPC).

Flujo típico:

1. Generación de código Java a partir de los esquemas `.capnp`.
2. Compilación del submódulo `capnproto-java-rpc` (si es necesario) para disponer del `capnpc-java` y runtime.
3. Compilación del módulo `tax-client` que incluye las clases generadas y la lógica del cliente.
4. Ejecución del cliente usando el script `run-client.sh` o ejecutando el JAR generado.

## Requisitos

- Java JDK 11+ (se recomienda JDK 17 o superior).
- Maven (se usa el wrapper `./mvnw`, pero tener Maven instalado puede ser útil).
- Git (para inicializar submódulos si es necesario).
- Opcional: si no se usa el `capnpc-java` incluido en el submódulo, instale el compilador Cap'n Proto (`capnp`) en el sistema.

## Preparar el repositorio (submódulos)

Si el submódulo aún no está inicializado, ejecutar desde la raíz del repositorio:

```bash
git submodule update --init --recursive
```

Esto descargará `capnproto-java-rpc` y otros submódulos necesarios.

## Generar código Cap'n Proto

Si el proyecto ya incluye un script para generación (por ejemplo en `tax-client/generate-capnp.sh`), úselo. Desde `client/java` o desde `tax-client`:

```bash
cd client/java/tax-client
./generate-capnp.sh
```

## Compilar el submódulo `capnproto-java-rpc`

Algunos entornos contienen el compilado del runtime y herramientas dentro del submódulo. Para compilarlo desde `client/java` usando el wrapper Maven:

```bash
cd client/java
./mvnw -pl capnproto-java-rpc -am clean install
```

Esto compila `capnproto-java-rpc` y lo instala en el repositorio local de Maven para que el resto de módulos lo consuman.

## Compilar el cliente Java

Desde `client/java`:

```bash
./mvnw -pl tax-client -am clean package
```

O para compilar todos los módulos Java del directorio:

```bash
./mvnw clean package
```

Al finalizar, el JAR del cliente suele encontrarse en `tax-client/target/`.

## Ejecutar el cliente

La forma recomendada es usar el script auxiliar `run-client.sh` en `client/java`:

```bash
cd client/java
./run-client.sh
```

Alternativamente puede ejecutar directamente el JAR producido por Maven:

```bash
java -jar tax-client/target/tax-client-<version>-jar-with-dependencies.jar
```

O usando Maven:

```bash
cd client/java/tax-client
../mvnw exec:java -Dexec.mainClass=tax.Client
```

## Notas y soluciones de problemas

- Si la generación Cap'n Proto falla por no encontrar `capnpc-java`, asegúrese de que el submódulo `capnproto-java-rpc` esté compilado o que `capnpc-java` esté en su `PATH`.
- Si hay errores de versión de Java, verifique la versión del JDK: `java -version`.
- Para depurar la ejecución, habilite logs y verifique `tax-client/target` para artefactos generados.

## Aviso sobre código fuente y esquemas

- **Código fuente:** El único código fuente incluido en este árbol es `tax-client/src/main/java/tax/Client.java`.

- **Esquema Cap'n Proto:** El prototipo `tax-client/src/main/schema/references.capnp` debe ser idéntico al archivo raíz `schema.capnp` en la raíz del repositorio. Mantenga ambos archivos sincronizados para evitar discrepancias en la generación de clases y las interfaces RPC.

