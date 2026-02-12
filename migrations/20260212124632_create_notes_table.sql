-- Add migration script here
CREATE TABLE IF NOT EXISTS notes (
    id TEXT PRIMARY KEY,
    raw_text TEXT NOT NULL,
    processed_markdown TEXT NOT NULL,
    created_at TEXT NOT NULL
);