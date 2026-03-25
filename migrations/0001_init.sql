-- Messages table
CREATE TABLE IF NOT EXISTS messages (
    id TEXT PRIMARY KEY,
    user_key TEXT NOT NULL,
    message TEXT NOT NULL,
    title TEXT,
    priority INTEGER DEFAULT 0,
    sound TEXT DEFAULT 'pushover',
    url TEXT,
    url_title TEXT,
    device TEXT,
    retry INTEGER,
    expire INTEGER,
    html INTEGER DEFAULT 0,
    attachment TEXT,
    status TEXT DEFAULT 'pending', -- pending, sent, failed, acknowledged
    receipt TEXT,
    sent_at TEXT,
    delivered_at TEXT,
    acknowledged_at TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_messages_user_key ON messages(user_key);
CREATE INDEX IF NOT EXISTS idx_messages_status ON messages(status);
CREATE INDEX IF NOT EXISTS idx_messages_created_at ON messages(created_at);

-- Webhooks table
CREATE TABLE IF NOT EXISTS webhooks (
    id TEXT PRIMARY KEY,
    user_key TEXT NOT NULL,
    url TEXT NOT NULL,
    secret TEXT NOT NULL,
    events TEXT NOT NULL, -- JSON array: ["message_delivered", "message_acknowledged"]
    active INTEGER DEFAULT 1,
    last_triggered_at TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_webhooks_user_key ON webhooks(user_key);
CREATE INDEX IF NOT EXISTS idx_webhooks_active ON webhooks(active);

-- Webhook deliveries table (for tracking delivery status)
CREATE TABLE IF NOT EXISTS webhook_deliveries (
    id TEXT PRIMARY KEY,
    webhook_id TEXT NOT NULL,
    message_id TEXT NOT NULL,
    event_type TEXT NOT NULL, -- message_delivered, message_acknowledged
    status TEXT DEFAULT 'pending', -- pending, success, failed
    retry_count INTEGER DEFAULT 0,
    last_retry_at TEXT,
    response_code INTEGER,
    response_body TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (webhook_id) REFERENCES webhooks(id) ON DELETE CASCADE,
    FOREIGN KEY (message_id) REFERENCES messages(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_webhook_deliveries_webhook_id ON webhook_deliveries(webhook_id);
CREATE INDEX IF NOT EXISTS idx_webhook_deliveries_status ON webhook_deliveries(status);

-- Failed deliveries table (for recovery worker)
CREATE TABLE IF NOT EXISTS failed_deliveries (
    id TEXT PRIMARY KEY,
    message_id TEXT NOT NULL,
    attempt_count INTEGER DEFAULT 0,
    last_attempt_at TEXT,
    error_message TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (message_id) REFERENCES messages(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_failed_deliveries_message_id ON failed_deliveries(message_id);
CREATE INDEX IF NOT EXISTS idx_failed_deliveries_attempt_count ON failed_deliveries(attempt_count);
