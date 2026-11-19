#!/usr/bin/env bash
# Makes sure a local Postgres for development and tests is running, in a container, and prints its
# URL. Tests use `#[sqlx::test]`, which creates a fresh database per test on this server, so every
# worktree can share one container. CI sets DATABASE_URL itself and never calls this.
#
#   scripts/dev-db.sh          start it if needed, print the URL
#   scripts/dev-db.sh stop     remove the container (data is thrown away)
set -euo pipefail
name="${BELAJAR_DEV_DB_CONTAINER:-belajar-dev-pg}"
port="${BELAJAR_DEV_DB_PORT:-55432}"
url="postgres://postgres:postgres@127.0.0.1:${port}/postgres"
engine="$(command -v podman || command -v docker || true)"
[ -n "$engine" ] || { echo "dev-db: neither podman nor docker is installed" >&2; exit 1; }

if [ "${1:-}" = stop ]; then "$engine" rm -f "$name" >/dev/null; echo "dev-db: removed $name" >&2; exit 0; fi

if [ "$("$engine" inspect -f '{{.State.Running}}' "$name" 2>/dev/null)" != true ]; then
  "$engine" rm -f "$name" >/dev/null 2>&1 || true
  # Many worktrees run tests at once, each test with its own pool: raise the connection limit.
  "$engine" run -d --name "$name" -p "127.0.0.1:${port}:5432" -e POSTGRES_PASSWORD=postgres \
    docker.io/library/postgres:16 -c max_connections=1000 -c fsync=off >/dev/null
fi
for _ in $(seq 1 60); do
  "$engine" exec "$name" pg_isready -U postgres -h 127.0.0.1 >/dev/null 2>&1 && { echo "$url"; exit 0; }
  sleep 0.5
done
echo "dev-db: postgres in $name did not become ready" >&2; exit 1
