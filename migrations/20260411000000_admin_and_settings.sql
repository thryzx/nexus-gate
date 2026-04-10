-- Admin users table for login authentication
CREATE TABLE IF NOT EXISTS admin_users (
    id         UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    username   TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,   -- SHA-256 hash
    role       TEXT NOT NULL DEFAULT 'admin',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- System settings key-value store
CREATE TABLE IF NOT EXISTS settings (
    key        TEXT PRIMARY KEY,
    value      JSONB NOT NULL DEFAULT '{}',
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Add cache_read/cache_creation tokens to usage_logs
ALTER TABLE usage_logs ADD COLUMN IF NOT EXISTS cache_read_tokens BIGINT NOT NULL DEFAULT 0;
ALTER TABLE usage_logs ADD COLUMN IF NOT EXISTS cache_creation_tokens BIGINT NOT NULL DEFAULT 0;
ALTER TABLE usage_logs ADD COLUMN IF NOT EXISTS platform TEXT NOT NULL DEFAULT '';
ALTER TABLE usage_logs ADD COLUMN IF NOT EXISTS request_id TEXT NOT NULL DEFAULT '';

-- Insert default admin user (password: admin123, SHA-256 hashed)
-- This should be changed on first login
INSERT INTO admin_users (username, password_hash) VALUES (
    'admin',
    '240be518fabd2724ddb6f04eeb1da5967448d7e831c08c8fa822809f74c720a9'
) ON CONFLICT (username) DO NOTHING;
