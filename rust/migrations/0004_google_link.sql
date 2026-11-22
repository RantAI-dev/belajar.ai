-- "Hubungkan dengan Google" from Akun: a signed-in student links a Google account to the student
-- they already are. The attempt records, server-side and at start, which student and which session
-- it is for; the callback never takes either from the query. Both are NULL for a sign-in attempt.
ALTER TABLE google_sign_in_attempts
    ADD COLUMN link_student_id uuid REFERENCES users (id) ON DELETE CASCADE,
    ADD COLUMN link_session_id uuid REFERENCES sessions (id) ON DELETE CASCADE,
    ADD CONSTRAINT ck_google_sign_in_attempts_link
        CHECK ((link_student_id IS NULL) = (link_session_id IS NULL));
