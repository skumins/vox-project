-- Encrypted API keys for each user and each provider.
-- Requires ENCRYPTION_KEY from environment to decrypt.
CREATE TABLE api_keys (
    id             UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id        UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    provider       TEXT NOT NULL,       -- 'deepgram', 'openrouter'
    encrypted_key  BYTEA NOT NULL,      
    nonce          BYTEA NOT NULL,      
    label          TEXT,                -- optional: 'My Deepgram Key'
    created_at     TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT unique_user_provider UNIQUE (user_id, provider)
);

CREATE INDEX idx_api_keys_user_id ON api_keys(user_id);