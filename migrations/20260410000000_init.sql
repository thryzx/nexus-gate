-- nexus-gate initial schema
-- Designed with zero fingerprint overlap to CRS (Redis-only) and S2A (Ent ORM).

CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- ── Fingerprint Profiles ──
CREATE TABLE fingerprint_profiles (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name                TEXT NOT NULL UNIQUE,
    tls_profile         JSONB NOT NULL DEFAULT '{}',
    http2_settings      JSONB NOT NULL DEFAULT '{}',
    header_order        JSONB NOT NULL DEFAULT '[]',
    user_agent_template TEXT NOT NULL DEFAULT '',
    extra_headers       JSONB NOT NULL DEFAULT '{}',
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- ── Accounts (upstream AI provider credentials) ──
CREATE TABLE accounts (
    id                     UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name                   TEXT NOT NULL,
    platform               TEXT NOT NULL CHECK (platform IN ('claude','openai','gemini','bedrock','azure')),
    account_type           TEXT NOT NULL CHECK (account_type IN ('oauth','apikey','bedrock')),
    credentials_enc        TEXT NOT NULL,          -- AES-256-GCM encrypted JSON
    status                 TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active','unavailable','disabled','error','blocked')),
    priority               INT NOT NULL DEFAULT 50,
    max_concurrency        INT NOT NULL DEFAULT 1,
    proxy_url              TEXT,
    fingerprint_profile_id UUID REFERENCES fingerprint_profiles(id) ON DELETE SET NULL,
    created_at             TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at             TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_accounts_platform_status ON accounts (platform, status);

-- ── API Keys (client authentication) ──
CREATE TABLE api_keys (
    id                UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    key_hash          TEXT NOT NULL UNIQUE,        -- SHA-256 of raw key
    name              TEXT NOT NULL,
    permissions       JSONB NOT NULL DEFAULT '[]', -- ["claude","openai"] or [] = all
    daily_cost_limit  DOUBLE PRECISION NOT NULL DEFAULT 0,
    total_cost_limit  DOUBLE PRECISION NOT NULL DEFAULT 0,
    max_concurrency   INT NOT NULL DEFAULT 0,      -- 0 = unlimited
    rate_limit_rpm    INT NOT NULL DEFAULT 0,      -- 0 = use global default
    restricted_models TEXT NOT NULL DEFAULT '[]',   -- JSON array of allowed models, [] = all
    status            TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active','disabled')),
    expires_at        TIMESTAMPTZ,
    created_at        TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- ── Usage Logs ──
CREATE TABLE usage_logs (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    api_key_id    UUID NOT NULL REFERENCES api_keys(id) ON DELETE CASCADE,
    account_id    UUID NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    model         TEXT NOT NULL,
    input_tokens  BIGINT NOT NULL DEFAULT 0,
    output_tokens BIGINT NOT NULL DEFAULT 0,
    cost_usd      DOUBLE PRECISION NOT NULL DEFAULT 0,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_usage_key_date ON usage_logs (api_key_id, created_at);
CREATE INDEX idx_usage_account_date ON usage_logs (account_id, created_at);
CREATE INDEX idx_usage_model_date ON usage_logs (model, created_at);

-- ── Admin Sessions ──
CREATE TABLE admin_sessions (
    id         UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    username   TEXT NOT NULL,
    token_hash TEXT NOT NULL UNIQUE,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
