#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"

# build
./mvnw -DskipTests -pl :tax-client -am install

# run client
cd tax-client
../mvnw exec:java -Dexec.mainClass=tax.Client
