-- "Masuk dengan Google": a student may sign in with a Google account instead of a password.
-- Children's privacy (docs/spec.md): only the Google subject id and the display name are kept;
-- no email, no picture, no Google tokens.

-- A Google-only student has no password; login by username skips them (password_hash IS NULL).
ALTER TABLE users ALTER COLUMN password_hash DROP NOT NULL;

-- One Google account links to exactly one student, and a student has at most one Google account.
-- Linked by `subject` (Google's stable `sub`), never by email.
CREATE TABLE google_identities (
    id            uuid PRIMARY KEY,
    student_id    uuid NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    subject       text NOT NULL,
    -- The Google profile name at link time (the student may change their nickname afterwards).
    display_name  text NOT NULL,
    created_at    timestamptz NOT NULL DEFAULT now(),
    updated_at    timestamptz NOT NULL DEFAULT now()
);
CREATE UNIQUE INDEX uq_google_identities_subject ON google_identities (subject);
CREATE UNIQUE INDEX uq_google_identities_student_id ON google_identities (student_id);

-- One sign-in in flight: state, nonce and PKCE verifier, bound to the browser by the SHA-256 of a
-- random httpOnly cookie. Single use (deleted when the callback reads it) and short-lived.
CREATE TABLE google_sign_in_attempts (
    id                 uuid PRIMARY KEY,
    browser_token_hash bytea NOT NULL,
    state              text NOT NULL,
    nonce              text NOT NULL,
    code_verifier      text NOT NULL,
    expires_at         timestamptz NOT NULL,
    created_at         timestamptz NOT NULL DEFAULT now(),
    updated_at         timestamptz NOT NULL DEFAULT now()
);
CREATE UNIQUE INDEX uq_google_sign_in_attempts_browser_token_hash
    ON google_sign_in_attempts (browser_token_hash);
CREATE INDEX idx_google_sign_in_attempts_expires_at ON google_sign_in_attempts (expires_at);

-- A verified Google account with no student yet: it waits here, bound to the browser by the hash
-- of a second httpOnly cookie, until the student picks a grade. Single use and short-lived.
CREATE TABLE google_pending_signups (
    id            uuid PRIMARY KEY,
    token_hash    bytea NOT NULL,
    subject       text NOT NULL,
    display_name  text NOT NULL,
    expires_at    timestamptz NOT NULL,
    created_at    timestamptz NOT NULL DEFAULT now(),
    updated_at    timestamptz NOT NULL DEFAULT now()
);
CREATE UNIQUE INDEX uq_google_pending_signups_token_hash ON google_pending_signups (token_hash);
CREATE INDEX idx_google_pending_signups_expires_at ON google_pending_signups (expires_at);
