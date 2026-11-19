-- The whole MVP schema in one migration, written before the slices so they never race on it.
-- Every row that belongs to a student carries student_id, and every query on it filters by it.
-- Ids are UUIDv7 made by the server. Times are timestamptz in UTC.

CREATE TABLE schools (
    id          uuid PRIMARY KEY,
    name        text NOT NULL,
    created_at  timestamptz NOT NULL DEFAULT now(),
    updated_at  timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE users (
    id             uuid PRIMARY KEY,
    school_id      uuid REFERENCES schools (id) ON DELETE SET NULL,
    username       text NOT NULL,
    password_hash  text NOT NULL,
    role           text NOT NULL DEFAULT 'student' CHECK (role IN ('student', 'teacher')),
    created_at     timestamptz NOT NULL DEFAULT now(),
    updated_at     timestamptz NOT NULL DEFAULT now()
);
-- Usernames are unique regardless of case: "Budi" and "budi" are the same account.
CREATE UNIQUE INDEX uq_users_username ON users (lower(username));

CREATE TABLE sessions (
    id          uuid PRIMARY KEY,
    user_id     uuid NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    -- SHA-256 of the cookie token; the token itself is never stored.
    token_hash  bytea NOT NULL,
    expires_at  timestamptz NOT NULL,
    created_at  timestamptz NOT NULL DEFAULT now(),
    updated_at  timestamptz NOT NULL DEFAULT now()
);
CREATE UNIQUE INDEX uq_sessions_token_hash ON sessions (token_hash);
CREATE INDEX idx_sessions_user_id ON sessions (user_id);

CREATE TABLE student_profiles (
    student_id  uuid PRIMARY KEY REFERENCES users (id) ON DELETE CASCADE,
    grade       smallint NOT NULL CHECK (grade BETWEEN 1 AND 12),
    nickname    text NOT NULL,
    -- Free-form learning style note used in the memory system message; null until set.
    style       text,
    created_at  timestamptz NOT NULL DEFAULT now(),
    updated_at  timestamptz NOT NULL DEFAULT now()
);

-- Tanya (mode ask) and Belajar (mode learn) conversations.
CREATE TABLE conversations (
    id          uuid PRIMARY KEY,
    student_id  uuid NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    mode        text NOT NULL CHECK (mode IN ('ask', 'learn')),
    -- Our subject id ("ipa", "mtk", ...), or null for "all subjects".
    subject_id  text,
    chapter_id  text,
    title       text NOT NULL,
    created_at  timestamptz NOT NULL DEFAULT now(),
    updated_at  timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX idx_conversations_student_id_updated_at
    ON conversations (student_id, updated_at DESC, id DESC);

CREATE TABLE messages (
    id               uuid PRIMARY KEY,
    conversation_id  uuid NOT NULL REFERENCES conversations (id) ON DELETE CASCADE,
    student_id       uuid NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    role             text NOT NULL CHECK (role IN ('user', 'assistant')),
    content          text NOT NULL,
    -- ports::upstream::Source values, page_url included (server-side only).
    sources          jsonb NOT NULL DEFAULT '[]'::jsonb,
    -- Null on user messages.
    is_grounded      boolean,
    created_at       timestamptz NOT NULL DEFAULT now(),
    updated_at       timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX idx_messages_conversation_id_created_at
    ON messages (conversation_id, created_at, id);
CREATE INDEX idx_messages_student_id_created_at ON messages (student_id, created_at);

-- Belajar: one row per student and chapter.
CREATE TABLE lesson_progress (
    id               uuid PRIMARY KEY,
    student_id       uuid NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    subject_id       text NOT NULL,
    chapter_id       text NOT NULL,
    conversation_id  uuid REFERENCES conversations (id) ON DELETE SET NULL,
    step             text NOT NULL CHECK (step IN ('explain', 'check', 'quiz', 'done')),
    -- Best result so far.
    stars            smallint NOT NULL DEFAULT 0 CHECK (stars BETWEEN 0 AND 3),
    completed_at     timestamptz,
    created_at       timestamptz NOT NULL DEFAULT now(),
    updated_at       timestamptz NOT NULL DEFAULT now()
);
CREATE UNIQUE INDEX uq_lesson_progress_student_id_subject_id_chapter_id
    ON lesson_progress (student_id, subject_id, chapter_id);
CREATE INDEX idx_lesson_progress_student_id_updated_at
    ON lesson_progress (student_id, updated_at DESC);

-- Latihan. A set is stored once the upstream delivers it.
CREATE TABLE practice_sets (
    id               uuid PRIMARY KEY,
    student_id       uuid NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    subject_id       text NOT NULL,
    topic            text NOT NULL,
    practice_type    text NOT NULL
        CHECK (practice_type IN ('mcq_normal', 'mcq_multiple_correct', 'mcq_story', 'mcq_mixed')),
    total_questions  smallint NOT NULL,
    status           text NOT NULL DEFAULT 'ready' CHECK (status IN ('ready', 'submitted')),
    -- ports::upstream::PracticeSet (answer keys included). Never sent to the browser as is.
    practice         jsonb NOT NULL,
    score_correct    smallint,
    submitted_at     timestamptz,
    created_at       timestamptz NOT NULL DEFAULT now(),
    updated_at       timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX idx_practice_sets_student_id_created_at
    ON practice_sets (student_id, created_at DESC, id DESC);

CREATE TABLE practice_answers (
    id               uuid PRIMARY KEY,
    practice_set_id  uuid NOT NULL REFERENCES practice_sets (id) ON DELETE CASCADE,
    student_id       uuid NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    question_id      text NOT NULL,
    -- Option labels, e.g. ["A"] or ["B","D"].
    selected         jsonb NOT NULL,
    is_correct       boolean NOT NULL,
    created_at       timestamptz NOT NULL DEFAULT now(),
    updated_at       timestamptz NOT NULL DEFAULT now()
);
CREATE UNIQUE INDEX uq_practice_answers_practice_set_id_question_id
    ON practice_answers (practice_set_id, question_id);

CREATE TABLE topic_mastery (
    id          uuid PRIMARY KEY,
    student_id  uuid NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    subject_id  text NOT NULL,
    topic       text NOT NULL,
    attempts    integer NOT NULL DEFAULT 0 CHECK (attempts >= 0),
    correct     integer NOT NULL DEFAULT 0 CHECK (correct >= 0 AND correct <= attempts),
    created_at  timestamptz NOT NULL DEFAULT now(),
    updated_at  timestamptz NOT NULL DEFAULT now()
);
CREATE UNIQUE INDEX uq_topic_mastery_student_id_subject_id_topic
    ON topic_mastery (student_id, subject_id, topic);

-- Memory: short notes about finished sessions (written by a later job; unused in the MVP).
CREATE TABLE memory_notes (
    id               uuid PRIMARY KEY,
    student_id       uuid NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    conversation_id  uuid REFERENCES conversations (id) ON DELETE SET NULL,
    subject_id       text,
    note             text NOT NULL,
    created_at       timestamptz NOT NULL DEFAULT now(),
    updated_at       timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX idx_memory_notes_student_id_created_at ON memory_notes (student_id, created_at DESC);

-- A mission completed on a day (Asia/Jakarta calendar date). Progress itself is derived from
-- activity; this row exists so the reward is given once.
CREATE TABLE daily_missions (
    id             uuid PRIMARY KEY,
    student_id     uuid NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    on_date        date NOT NULL,
    kind           text NOT NULL CHECK (kind IN ('ask', 'practice', 'lesson')),
    completed_at   timestamptz NOT NULL,
    stars_awarded  smallint NOT NULL CHECK (stars_awarded >= 0),
    created_at     timestamptz NOT NULL DEFAULT now(),
    updated_at     timestamptz NOT NULL DEFAULT now()
);
CREATE UNIQUE INDEX uq_daily_missions_student_id_on_date_kind
    ON daily_missions (student_id, on_date, kind);

-- Stars are entries, never a balance. (reason, ref_id) is unique per student so an award can be
-- retried safely: ref_id is the practice set id, "<subject>/<chapter>/<stars>", or "<date>/<kind>".
CREATE TABLE star_ledger (
    id          uuid PRIMARY KEY,
    student_id  uuid NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    amount      integer NOT NULL CHECK (amount > 0),
    reason      text NOT NULL CHECK (reason IN ('practice', 'lesson', 'mission')),
    ref_id      text NOT NULL,
    created_at  timestamptz NOT NULL DEFAULT now(),
    updated_at  timestamptz NOT NULL DEFAULT now()
);
CREATE UNIQUE INDEX uq_star_ledger_student_id_reason_ref_id
    ON star_ledger (student_id, reason, ref_id);

-- Background work (session notes later). Unused in the MVP.
CREATE TABLE jobs (
    id          uuid PRIMARY KEY,
    kind        text NOT NULL,
    status      text NOT NULL DEFAULT 'queued'
        CHECK (status IN ('queued', 'running', 'succeeded', 'failed', 'cancelled')),
    payload     jsonb NOT NULL DEFAULT '{}'::jsonb,
    error       jsonb,
    attempts    integer NOT NULL DEFAULT 0,
    run_after   timestamptz NOT NULL DEFAULT now(),
    created_at  timestamptz NOT NULL DEFAULT now(),
    updated_at  timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX idx_jobs_status_run_after ON jobs (status, run_after);
