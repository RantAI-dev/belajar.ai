# belajar.ai

A learning app for school students (SD to SMA): ask questions, guided lessons and practice sets, all
answered from their own textbooks. A Rust server (`rust/`) serves the API under `/v1` and the built
web app (`web/`, a React SPA); it calls an upstream learning API for answers and keeps each
student's accounts, history and progress in its own database.

## Commands

| What | Command |
|---|---|
| Everything, before any handoff | `scripts/check.sh` (budget: 10 minutes) |
| Only the parts your change can affect, while iterating | `scripts/check.sh --changed` (budget: 2 minutes) |
| One test | `cd rust && cargo test -p belajar-server --test http unknown_api_path` or `cd web && bun test src/contracts/api-error.test.ts -t "standard"` |
| Regenerate the typed web contracts after a handler or DTO change | `scripts/gen-contracts.sh` (writes `web/src/contracts/openapi.json` and `api.gen.ts`; commit both) |
| Run it locally | `cd web && bun run build`, then `cd rust && DATABASE_URL=… BELAJAR_WEB_DIR=../web/dist cargo run -p belajar-server --bin belajar-server` (port 8080; `DATABASE_URL` from `scripts/dev-db.sh`, demo student from `cargo run -p belajar-server --bin seed-demo`). For UI work, also `cd web && bun run dev` (proxies `/v1` to 8080). |

## Where code lives

Dependencies point down this list; nothing imports from a line above it.

- `rust/crates/core/` — domain rules with no I/O (grades, progress, streaks). Depends on no other crate here.
- `rust/crates/ports/` — traits the server depends on: `Upstream`, `Clock`, and one store trait per slice. Contract: the lead changes `clock.rs` and `upstream.rs`.
- `rust/crates/store-pg/` — Postgres implementations of the store traits (sqlx). Schema: `rust/migrations/`.
- `rust/crates/upstream/` — the HTTP+SSE client for the upstream API. Its wire JSON lives in one module.
- `rust/crates/fake-upstream/` — an in-repo fake of the upstream for dev, tests and the demo.
- `rust/crates/server/` — axum: `/v1` routes, middleware, the static web app, `main.rs`. Handlers stay thin.
- `web/src/contracts/` — types that cross the API boundary, `Result`, the error-body parser. Imports nothing else.
- `web/src/ui/` — design tokens and shared components. Imports only contracts.
- `web/src/features/<slice>/` — one folder per capability; `index.ts` is its public surface.
- `web/src/app/` — router, layout and providers. May import everything above.
- Generated, never edited by hand: `rust/crates/server/src/error_codes.rs`, `web/src/contracts/error-codes.ts`
  (from `.agents/error-codes.tsv` via `scripts/error-codes.sh --write`).

## Rules the gate enforces

`scripts/check.sh` and CI fail on these. A rule that exists only as a sentence here will be broken.

1. Rust: no `unwrap`, `expect`, `panic!`, `todo!`, slice indexing or `unsafe` outside tests; a
   suppression needs `reason = "..."` (workspace lints in `rust/Cargo.toml`, reported by clippy).
2. TypeScript: `strict`, `noUncheckedIndexedAccess`, `exactOptionalPropertyTypes`; Biome forbids `any`
   and `!` (`cd web && bun run typecheck && bun run lint`).
3. Web layers: contracts <- ui <- features <- app, slices import each other only through `index.ts`,
   no `throw` in feature code (`cd web && bun run boundaries`).
4. A source file stays under 600 lines, 1,000 for Rust (`scripts/file-size.sh`).
5. Error codes come only from `.agents/error-codes.tsv`; the generated files must be current
   (`scripts/error-codes.sh`).

## Database

- Postgres only in a container: `scripts/dev-db.sh` starts it and prints `DATABASE_URL`; the gate calls it.
- Tests that touch the database use `#[sqlx::test(migrations = "../../migrations")]` (a fresh database each).
- Runtime-checked queries only (`sqlx::query_as::<_, T>(...)`): no `query!` macros, so building needs no database.
- Every query on a student's rows filters by `student_id`. Another student's row answers 404, like a missing one.

## Conventions

The same in every RantAI repository. The full list is `docs/conventions.md`; read it before you add
an endpoint, an error, a log line, a config variable or a table. The ones broken most often:

- Every error response is `{ "error": { "code", "message", "requestId" } }`, built by one function.
- A `code` is a row of `.agents/error-codes.tsv`. New code: add a row, run
  `scripts/error-codes.sh --write`. Never rename or remove a row. Its status comes from the table.
- No internal text in a message: no exception text, SQL or upstream response. Log it instead.
- JSON fields are `camelCase`; times are `2026-11-11T08:30:00.000Z` in fields ending `At`.
- A list returns `{ "data": [...], "nextCursor": ... }`, never a bare array.
- Tests assert on the status and the `code`, not on message text.
- Commit subject: `type(scope): summary`.

## Working agreements

1. **Evidence, not claims.** "Green" means you ran `scripts/check.sh` after your last edit and can
   quote its last line. Anything you did not run is `not verified`.
2. **Never weaken a gate to get green.** No deleted or skipped tests, no suppression without a reason
   on the same line, no edits to anything in `.agents/protected-paths`. If a test is wrong, say so
   and stop.
3. **When the plan and the code disagree, do not pick a side silently.** Stop and report, or follow
   the code or contract test and list it under DEVIATIONS in your handoff.
4. **Stay in scope.** Change only the paths your task names. Everything else you noticed goes in the
   handoff.
5. **Proof depends on the change.** Logic or data: a test that failed before and passes now, in
   the module's existing test file. Visual: before and after captures. Server only: the test plus a
   short runtime transcript. Nobody will read your diff; the proof is what gets read.
6. **Comments say why, or nothing.** One line for a constraint the code cannot show. If a line looks
   wrong but is right, say why at that line, or the next reader will "fix" it.
7. **Search before you write.** Look for an existing helper first. A second copy is a review finding.
   New code goes in a file named for what it does, not at the bottom of the nearest big one. The
   gate fails a source file over 600 lines (1,000 for Rust), and a file already over that may only
   shrink.
8. **No new dependency** without the owner's approval. Say why you need it in the handoff.
9. **Keep the gate fast.** If it prints `check: SLOW`, or your change makes it slower, say so in the
   handoff. Speed comes from caching and from smaller crates or packages, never from removing tests.

## Traps

Things that went wrong in this repo. Add a line when the same mistake happens twice. Delete the line
once a lint rule or a test catches it.

- `scripts/check.sh` runs a part only for files git tracks. A new file is not checked until it is
  staged: `git add` it before you trust a green run.

## Roles

- You are the long-lived thread connected to the tracker: you are the **desk**. Load the `desk` skill.
- Another agent gave you a brief: you are the **worker**. Follow the brief and end with its handoff.
- The desk, or a person, asked you to plan, delegate or review one feature: you are the **lead**.
  Load the `orchestrate` skill first.
- You were asked to bring this repository up to the guidelines, or to clean it up: load the
  `adopt-guidelines` skill.
- You were asked to move code to another language: load the `port-code` skill.
