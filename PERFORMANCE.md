
# Resultados de Pruebas de Performance (Goose)

Fecha de ejecución: 2025-12-26

## Ejecucion

```bash
cargo run --release --bin tax-manager &
target/release/load-test --users 10 --run-time 10s
```

## Resumen

```
╔════════════════════════════════════════════════════════════╗
║          TAX ENGINE RPC LOAD TEST RESULTS                  ║
╠════════════════════════════════════════════════════════════╣
║  REQUESTS                                                  ║
║    Total:           14135                                  ║
║    Successful:      14111                                  ║
║    Errors:             24                                  ║
║    Error Rate:      0.17%                                  ║
╠════════════════════════════════════════════════════════════╣
║  THROUGHPUT                                                ║
║    Duration:       20.10s                                  ║
║    RPS:            703.15                                  ║
╠════════════════════════════════════════════════════════════╣
║  LATENCY                                                   ║
║    < 1ms:            6358 ( 45.06%)                        ║
║    1-5ms:             694 (  4.92%)                        ║
║    5-10ms:            779 (  5.52%)                        ║
║    > 10ms:           6280 ( 44.50%)                        ║
╚════════════════════════════════════════════════════════════╝
```