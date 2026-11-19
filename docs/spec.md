# belajar.ai — spec

Decisions and constraints. How each slice is built lives in its pull request. Owner: @kleopasevan.

## What it is

A learning app for school students (SD to SMA). Three modes, all answered from the student's own
textbooks with the page each answer came from:

- **Tanya** — chat questions, cited sources, page preview, quick follow-ups ("simpler", "another
  example", "make 3 questions"), history with rename.
- **Belajar** — pick subject and chapter, a guided session (explain, check understanding, short quiz),
  a chapter map with stars.
- **Latihan** — multiple-choice practice (4 types: normal, multiple-correct, story, mixed; 5/10/15
  questions), live progress while questions are generated, grading, encouraging explanations,
  topic mastery.

Plus Beranda (greeting, quick ask, daily missions, streak, stars, continue learning, subject grid) and
accounts (register, login, pick grade, change password). UI copy is Indonesian. Mockup:
https://claude.ai/artifact/NkN1vM6i3uXQevZ2fFRBJp (kid-friendly version: mascot, one colour per subject,
big buttons, Nunito/Fredoka, RantAI Lakehouse blue).

Deferred: answering offline (needs a local model), teacher dashboard, short-answer and essay questions.

## Architecture

```
browser (React SPA, PWA)
  │ HTTPS, httpOnly session cookie
  ▼
belajar-server (one Rust binary): /v1 API, auth, memory, progress, gamification,
  │                               SSE proxy to the upstream, job queue, serves web/dist
  ├──► Postgres (its own database)
  └──► upstream learning API ("Elise"): chat, practice, subjects, chapters, page images
```

- The browser talks only to `/v1` of this server. Upstream API keys live in server env, never in the
  repo or the browser. Each grade maps to three upstream assistant keys (ask, learn, practice).
- Crates: `core` (pure domain, no I/O) <- `ports` (traits: Store, Upstream, Clock) <- `store-pg`
  (sqlx), `upstream` (HTTP + SSE client) <- `server` (axum, composition root). Add a crate when it is
  first needed.
- Web: `contracts` <- `ui` <- `features/<slice>` <- `app` (enforced by `web/scripts/check-boundaries.ts`).
  API types are generated from the server's OpenAPI (utoipa); the web app never hand-writes them.

## Upstream API (what the server calls)

OpenAI-style, Bearer key per assistant. Responses carry `sources[]` with `number`, `cited`,
`page_url`; `grounded`; `retrieval_score`.

| Upstream | Used for |
|---|---|
| `GET /api/v1/mapel` | subjects of a grade |
| `POST /api/v1/chat/completions` (`stream`, `mapel`, `sources:"cited"`, `messages[]` incl. one `system`) | Tanya, Belajar |
| `POST /api/v1/practice/generate` (`stream:true` → SSE `progress`, `result`, `error` events) | Latihan |
| `POST /api/v1/practice/grade` | grading |
| `page_url` (signed, from a source) | page preview |
| chapters per subject | **not available yet** — upstream work; fake it until then |

Development and CI use an in-repo fake upstream (`upstream` crate test double + a small HTTP fake)
with recorded response shapes; nothing in the gate needs the real service.

## Data (Postgres)

Every row that belongs to a student carries `student_id`; repositories require it, so student A can
never read student B's data. Tables: `schools`, `users` (username, argon2id hash, role student|teacher),
`sessions`, `student_profiles` (grade, nickname, style), `conversations` (mode, subject, title),
`messages` (role, content, cited sources), `lesson_progress` (subject, chapter, step, stars 0–3),
`practice_sets` / `practice_answers` (answer keys stay server-side until submitted), `topic_mastery`,
`memory_notes`, `daily_missions`, `streaks`, `star_ledger` (entries, not a balance), `jobs`.

## Memory

The upstream is stateless and keyed per assistant, not per student, so memory lives here:

1. Conversation: the last N turns are re-sent with each request.
2. Learning profile: grade, nickname, style, weak topics, recent session notes → one short `system`
   message (~200–300 tokens) relevant to the active subject. Personalisation only; answers stay
   grounded in the books.
3. Session notes: an async job summarises a finished session in 2–3 lines with the upstream's `base`
   model. No student data leaves the deployment.

## Security and privacy (children)

- argon2id passwords, httpOnly SameSite session cookie, login rate limit, roles.
- Every endpoint gets an isolation test: another student's resource answers 404 (same body as missing),
  with a positive control for the owner — and the test is shown failing against the bug.
- Minimal data (no required email/phone), export and delete, retention agreed with the school.
- Public repository: no internal hosts, keys or student data in it.

## Conventions

`docs/conventions.md` (RantAI): one error body with `requestId`, codes from `.agents/error-codes.tsv`,
camelCase JSON, lists `{ data, nextCursor }`, `type(scope): summary` commits.

## Milestones

1. Spec and decisions (this file), backlog, acceptance tests for the first slices.
2. Foundation: auth and roles, migrations, SPA shell, design system (tokens, components, subject icons,
   mascot). Repo, gate and CI already exist.
3. Core flows: Tanya, Latihan, Belajar (Belajar needs the upstream chapters endpoint).
4. Memory and motivation: session notes, profile, missions, streak, stars.
5. Hardening: accessibility (axe), performance on slow school PCs and tablets, read-offline PWA,
   security review, a try-out with students.

Gates: decisions answered → internal demo on staging → ready for UAT.
