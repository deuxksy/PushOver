-- Store API token for retry/recovery
ALTER TABLE messages ADD COLUMN api_token TEXT;
