-- User sessions stored in PostgreSQL.
CREATE TABLE sessions (
    token       TEXT PRIMARY KEY,
    user_id     UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    expires_at  TIMESTAMPTZ NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Fast search for authenticated query
CREATE INDEX idx_sessions_user_id ON sessions(user_id);
-- For background cleaning
CREATE INDEX idx_sessions_expires_at ON sessions(expires_at);