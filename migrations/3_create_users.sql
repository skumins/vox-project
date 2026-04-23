-- Lecture notes  user_id can be null, change after adding authentication.
-- title is extracted from processed_markdown.
-- updated_at tracks note edits.
CREATE TABLE notes (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id             UUID REFERENCES users(id) ON DELETE SET NULL,
    title               TEXT,
    raw_text            TEXT NOT NULL,
    processed_markdown  TEXT NOT NULL,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Get all user notes, starting with the newest
CREATE INDEX idx_notes_user_id_created ON notes(user_id, created_at DESC);