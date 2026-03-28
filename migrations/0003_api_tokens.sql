-- API tokens table for authentication
CREATE TABLE IF NOT EXISTS api_tokens (
    token TEXT PRIMARY KEY,
    user_key TEXT NOT NULL,
    name TEXT,
    active INTEGER DEFAULT 1,
    last_used_at TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_api_tokens_user_key ON api_tokens(user_key);
CREATE INDEX IF NOT EXISTS idx_api_tokens_active ON api_tokens(active);
