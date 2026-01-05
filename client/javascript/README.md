
# Cliente JavaScript (Cap'n Proto RPC)

Este directorio contiene un cliente Node.js/TypeScript que se conecta al servidor RPC de `tax-manager` usando **Cap'n Proto RPC** vía la librería `capnp-es`.

## Prerrequisitos

- Node.js (recomendado: 18+)
- npm
- El servidor Rust corriendo y escuchando en `RPC_ADDR` (por defecto: `0.0.0.0:50051`)

## Preparación del ambiente

Desde este directorio:

```bash
cd client/javascript
npm install
```

## Generar el código desde el schema (.capnp)

El archivo de schema es `src/capnp/references.capnp`. Para regenerar los stubs TypeScript/JS:

```bash
npm run capnpc
```

Esto genera archivos en `src/capnp/` (por ejemplo `references.ts`, `references.js`, `references.d.ts`).

## Build (compilar a dist/)

```bash
npm run build
```

La salida queda en `dist/` (por ejemplo `dist/client.js`).

Notas:

- El proyecto está configurado como ESM (`"type": "module"`).
- Con `moduleResolution: NodeNext`, los imports locales usan extensión `.js` aunque el source sea `.ts`. TypeScript mantiene ese specifier y el runtime de Node encuentra el archivo compilado en `dist/`.

## Ejecutar el cliente

Con el servidor escuchando en `127.0.0.1:50051`:

```bash
npm start
```

Si necesitás apuntar a otra IP/puerto, editá el string que se pasa a `connectTaxEngine()` en `src/client.ts`.

## Explicación del código (src/client.ts)

El flujo del archivo `src/client.ts` es:

1. Importa `connectTaxEngine` desde `./connection.js`.
2. Importa el tipo `TaxEngine_Calculate$Params` desde los stubs generados (`./capnp/references.js`).
3. Conecta al servidor con `connectTaxEngine("127.0.0.1:50051")`.
4. Llama al método RPC `calculate` construyendo los parámetros Cap'n Proto usando el callback:

	- `calculate((params) => { ... })` recibe una instancia de `TaxEngine_Calculate$Params`.
	- `params._initTx()` inicializa el struct anidado `tx :TransactionRequest`.
	- Se setean los campos `clientId`, `amount`, `jurisdiction`, `product`.

5. Espera el resultado con `.promise()` y lee `execution.response` (tipo `TaxResponse`).
6. Imprime `response.totalAmount`.
7. Itera `response.breakdown` (lista Cap'n Proto) usando `breakdown.length` y `breakdown.get(i)`.
8. Cierra la conexión con `connection.close()`.

## Cómo funciona la conexión (src/connection.ts)

`capnp-es` provee la implementación de RPC (clases como `Conn`), pero el transporte de red depende de la app.
En `src/connection.ts` se implementa un transporte TCP mínimo:

- Se abre un `net.Socket` hacia el host/puerto.
- `TcpTransport.sendMessage()` serializa el mensaje en formato *unpacked* (no packed) usando `message.toArrayBuffer()`.
- `TcpTransport.recvMessage()` entrega el próximo mensaje recibido, esperando si aún no hay.
- El parser en `tryReadFrame()` reensambla frames del stream según el framing estándar de Cap'n Proto (header con cantidad de segmentos + tamaños), compatible con el servidor Rust que usa `capnp_futures::serialize::try_read_message`.

Luego:

- Se crea `const conn = new Conn(transport)`.
- Se obtiene el cliente remoto con `conn.bootstrap(TaxEngine)`.

Eso devuelve un stub con el método `calculate(...)` que usa el schema generado.

