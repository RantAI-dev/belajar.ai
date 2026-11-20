# Recorded practice responses (the real upstream)

Recorded on 2026-11-20 (~14:00 WIB) from the lab copy of the upstream learning API ("Elise"): the
agents app at v1.14.3 (agents-cloud UGM, rantai-agents OSS `9cf4aadb`), the same code as
production, with the grade 7 Latihan assistant key. Nothing was edited except that no key or token
appears anywhere (the bodies carry none).

Each `<name>.meta.json` holds the request (path and body) and the response status and content
type; `<name>.json` is the JSON body (pretty-printed) and `<name>.sse` the raw SSE stream.

| File | What |
|---|---|
| `generate-<type>-json` | `POST /api/v1/practice/generate`, mapel IPA, topic Pemuaian, 5 questions, for each `practice_type` |
| `generate-<type>-stream` | the same with `stream: true`: `progress` events, then one `result` with the JSON body |
| `grade-<type>` | `POST /api/v1/practice/grade` with that set sent back and answers: q1 right, q2 wrong, q3 right (or a subset of a multi key), the rest unanswered |
| `grade-mcq_multiple_correct-subset-and-lowercase` | a subset of a multi key (wrong) and the full key in lower case, reversed (right) |
| `grade-error-answers-as-list` | 400: `answers` must be an object `question_id -> letter or letters` |
| `generate-error-unknown-topic[-stream]` | 422 `topic_not_in_books` (JSON), or an `error` event with `status: 422` (stream) |
| `generate-error-generation-failed[-stream]` | 502 `generation_failed` (every generated question failed verification) |
| `generate-error-bad-type`, `-unknown-mapel`, `-bad-key` | 400 / 400 / 401 |

What these show (and what the wire types in `rust/crates/upstream/src/wire.rs` read):

- A question is `{question_id, type: mcq_single|mcq_multi, materi, question, scenario?, options:
  {"A": text, ...}, answer_key: "B" | ["A","C"], pembahasan, topic}`. No per-question source.
- Story questions are `mcq_single` with a `scenario`; a mixed set mixes `mcq_multi`, plain
  `mcq_single` and `mcq_single` with a `scenario`.
- The set may hold fewer questions than requested (`total_questions` is the delivered count; the
  `report` says why).
- Grading wants the set back as `practice` and `answers` as an object; it matches letters exactly
  (case-insensitive, any order), so a subset of a multi key is wrong.

Re-record against a lab copy of the upstream (same requests as in the `.meta.json` files), never
against the production server, and never commit a key.
