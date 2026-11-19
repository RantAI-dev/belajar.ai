#!/usr/bin/env bash
# Regenerates the typed web contracts from the server's OpenAPI document:
#   web/src/contracts/openapi.json  — printed by `cargo run -p belajar-server --bin openapi`
#   web/src/contracts/api.gen.ts   — generated from that document by openapi-typescript
# Run it after changing any handler or wire DTO, and commit both files. `cargo test` and the web
# test `api-gen.test.ts` both fail when the committed files are stale.
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"

cargo run --manifest-path rust/Cargo.toml -p belajar-server --bin openapi \
    > web/src/contracts/openapi.json
(cd web && bunx biome format --write src/contracts/openapi.json)
(cd web && bunx openapi-typescript src/contracts/openapi.json -o src/contracts/api.gen.ts)
(cd web && bunx biome format --write src/contracts/api.gen.ts)
echo "gen-contracts: web/src/contracts/openapi.json and api.gen.ts are up to date."
