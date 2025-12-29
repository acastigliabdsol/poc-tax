#!/usr/bin/env bash
# Generate Java sources from Cap'n Proto schema; idempotent and verbose.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="${SCRIPT_DIR}"
SCHEMA="${PROJECT_DIR}/src/main/schema/references.capnp"
OUT_DIR="${PROJECT_DIR}/target/generated-sources/capnp"

mkdir -p "${OUT_DIR}"

echo "[capnp] schema: ${SCHEMA} -> ${OUT_DIR}"

## Prefer native `capnp` tool
if command -v capnp >/dev/null 2>&1; then
  echo "[capnp] using native capnp"
  # add common include path so java.capnp can be found
  if capnp compile -I/usr/local/include --src-prefix="${PROJECT_DIR}/src/main/schema" -o java:"${OUT_DIR}" "${SCHEMA}"; then
    exit 0
  else
    echo "[capnp] native compiler failed; will write placeholder Java stubs into ${OUT_DIR}"
  fi
fi

exit 1
