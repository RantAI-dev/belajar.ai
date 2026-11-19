#!/usr/bin/env bash
# Makes a fresh checkout or worktree ready to work in. T3 Code runs it for every new worktree
# (see t3.json); a person runs it once after cloning. Safe to run again.
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"

git config core.hooksPath .githooks

# Worktrees do not get gitignored files. Link the env file from the main checkout instead of copying
# it, so there is one place to rotate a secret.
main_root="${T3CODE_PROJECT_ROOT:-$(git worktree list --porcelain | awk '/^worktree /{print $2; exit}')}"
for f in .env .env.local; do
  [ -e "$f" ] || { [ -f "$main_root/$f" ] && [ "$main_root" != "$PWD" ] && ln -s "$main_root/$f" "$f"; } || true
done

# A stale lockfile must fail here, not be rewritten by an agent halfway through a task.
# A CI job installs only its own part's toolchain, so a stack whose tool is missing is skipped.
have() { command -v "$1" >/dev/null 2>&1 || { echo "setup: $1 not installed, skipping its install"; return 1; }; }
if [ -f bun.lock ]        && have bun;   then bun install --frozen-lockfile; fi
if [ -f web/bun.lock ]    && have bun;   then (cd web && bun install --frozen-lockfile); fi
if [ -f pnpm-lock.yaml ]  && have pnpm;  then pnpm install --frozen-lockfile; fi
if [ -f uv.lock ]         && have uv;    then uv sync --frozen; fi
if [ -f rust/Cargo.toml ] && have cargo; then (cd rust && cargo fetch --locked); fi

echo "setup: ready"
