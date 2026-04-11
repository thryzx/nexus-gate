-- Full CRS alignment: expand platforms, add account groups, quota cards,
-- webhooks, users, request details, balance scripts, model pricing, service rates.

-- ── 1. Expand accounts table ──

-- Drop old CHECK constraint and add expanded one
ALTER TABLE accounts DROP CONSTRAINT IF EXISTS accounts_platform_check;
ALTER TABLE accounts ADD CONSTRAINT accounts_platform_check
  CHECK (platform IN (
    'claude', 'claude-console', 'bedrock', 'ccr',
    'gemini', 'gemini-api',
    'openai', 'openai-responses', 'azure-openai',
    'droid'
  ));

ALTER TABLE accounts DROP CONSTRAINT IF EXISTS accounts_account_type_check;
ALTER TABLE accounts ADD CONSTRAINT accounts_account_type_check
  CHECK (account_type IN ('oauth', 'apikey', 'bedrock', 'cookie'));

ALTER TABLE accounts DROP CONSTRAINT IF EXISTS accounts_status_check;
ALTER TABLE accounts ADD CONSTRAINT accounts_status_check
  CHECK (status IN ('active', 'unavailable', 'disabled', 'error', 'blocked', 'paused'));

-- Add new columns
ALTER TABLE accounts ADD COLUMN IF NOT EXISTS description TEXT DEFAULT '';
ALTER TABLE accounts ADD COLUMN IF NOT EXISTS schedulable BOOLEAN DEFAULT true;
ALTER TABLE accounts ADD COLUMN IF NOT EXISTS group_id UUID;
ALTER TABLE accounts ADD COLUMN IF NOT EXISTS expires_at TIMESTAMPTZ;
ALTER TABLE accounts ADD COLUMN IF NOT EXISTS rate_limit INT DEFAULT 0;
ALTER TABLE accounts ADD COLUMN IF NOT EXISTS extra_config JSONB DEFAULT '{}';

-- ── 2. Account Groups ──

CREATE TABLE IF NOT EXISTS account_groups (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  name TEXT UNIQUE NOT NULL,
  description TEXT DEFAULT '',
  priority INT DEFAULT 50,
  created_at TIMESTAMPTZ DEFAULT NOW(),
  updated_at TIMESTAMPTZ DEFAULT NOW()
);

ALTER TABLE accounts ADD CONSTRAINT fk_accounts_group
  FOREIGN KEY (group_id) REFERENCES account_groups(id) ON DELETE SET NULL;

-- ── 3. API Key soft delete and tags ──

ALTER TABLE api_keys ADD COLUMN IF NOT EXISTS deleted_at TIMESTAMPTZ;
ALTER TABLE api_keys ADD COLUMN IF NOT EXISTS tags JSONB DEFAULT '[]';
ALTER TABLE api_keys ADD COLUMN IF NOT EXISTS description TEXT DEFAULT '';
ALTER TABLE api_keys ADD COLUMN IF NOT EXISTS weekly_cost_limit DOUBLE PRECISION DEFAULT 0;

-- ── 4. Quota Cards ──

CREATE TABLE IF NOT EXISTS quota_cards (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  code TEXT UNIQUE NOT NULL,
  name TEXT NOT NULL DEFAULT '',
  value_type TEXT NOT NULL DEFAULT 'cost_limit',
  value DOUBLE PRECISION NOT NULL DEFAULT 0,
  max_redemptions INT DEFAULT 1,
  current_redemptions INT DEFAULT 0,
  expires_at TIMESTAMPTZ,
  status TEXT DEFAULT 'active' CHECK (status IN ('active', 'disabled', 'exhausted')),
  created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS redemptions (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  card_id UUID NOT NULL REFERENCES quota_cards(id) ON DELETE CASCADE,
  api_key_id UUID NOT NULL REFERENCES api_keys(id) ON DELETE CASCADE,
  value DOUBLE PRECISION NOT NULL DEFAULT 0,
  revoked BOOLEAN DEFAULT false,
  created_at TIMESTAMPTZ DEFAULT NOW()
);

-- ── 5. Webhook Configuration ──

CREATE TABLE IF NOT EXISTS webhook_config (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  enabled BOOLEAN DEFAULT false,
  global_url TEXT DEFAULT '',
  events JSONB DEFAULT '[]',
  updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS webhook_platforms (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  name TEXT NOT NULL,
  url TEXT NOT NULL,
  events JSONB DEFAULT '[]',
  enabled BOOLEAN DEFAULT true,
  created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Insert default webhook config
INSERT INTO webhook_config (id, enabled, global_url, events)
VALUES (gen_random_uuid(), false, '', '[]')
ON CONFLICT DO NOTHING;

-- ── 6. Users (LDAP-style front-end users) ──

CREATE TABLE IF NOT EXISTS users (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  username TEXT UNIQUE NOT NULL,
  display_name TEXT DEFAULT '',
  role TEXT DEFAULT 'user' CHECK (role IN ('user', 'admin')),
  source TEXT DEFAULT 'local' CHECK (source IN ('local', 'ldap')),
  status TEXT DEFAULT 'active' CHECK (status IN ('active', 'disabled')),
  max_keys INT DEFAULT 5,
  created_at TIMESTAMPTZ DEFAULT NOW(),
  updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- ── 7. Request Details ──

CREATE TABLE IF NOT EXISTS request_details (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  request_id TEXT NOT NULL,
  api_key_id UUID REFERENCES api_keys(id) ON DELETE SET NULL,
  account_id UUID REFERENCES accounts(id) ON DELETE SET NULL,
  platform TEXT DEFAULT '',
  model TEXT DEFAULT '',
  method TEXT DEFAULT '',
  path TEXT DEFAULT '',
  status_code INT DEFAULT 0,
  input_tokens BIGINT DEFAULT 0,
  output_tokens BIGINT DEFAULT 0,
  cost_usd DOUBLE PRECISION DEFAULT 0,
  duration_ms BIGINT DEFAULT 0,
  body_preview TEXT DEFAULT '',
  error_message TEXT DEFAULT '',
  created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_request_details_created
  ON request_details (created_at DESC);
CREATE INDEX IF NOT EXISTS idx_request_details_api_key
  ON request_details (api_key_id, created_at DESC);

-- ── 8. Model Pricing ──

CREATE TABLE IF NOT EXISTS model_pricing (
  model TEXT PRIMARY KEY,
  input_price DOUBLE PRECISION DEFAULT 0,
  output_price DOUBLE PRECISION DEFAULT 0,
  cache_read_price DOUBLE PRECISION DEFAULT 0,
  cache_creation_price DOUBLE PRECISION DEFAULT 0,
  source TEXT DEFAULT 'manual',
  updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- ── 9. Service Rates ──

CREATE TABLE IF NOT EXISTS service_rates (
  service TEXT PRIMARY KEY,
  rate DOUBLE PRECISION DEFAULT 1.0,
  description TEXT DEFAULT '',
  updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Default service rates
INSERT INTO service_rates (service, rate, description) VALUES
  ('claude', 1.0, 'Claude API'),
  ('openai', 1.0, 'OpenAI API'),
  ('gemini', 1.0, 'Gemini API')
ON CONFLICT (service) DO NOTHING;

-- ── 10. Balance Scripts ──

CREATE TABLE IF NOT EXISTS balance_scripts (
  name TEXT PRIMARY KEY,
  platform TEXT NOT NULL DEFAULT '',
  script TEXT DEFAULT '',
  enabled BOOLEAN DEFAULT true,
  updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- ── 11. OEM Settings seed ──

INSERT INTO settings (key, value, updated_at) VALUES
  ('oem_site_name', '"Nexus Gate"', NOW()),
  ('oem_site_icon', '""', NOW()),
  ('oem_favicon', '""', NOW())
ON CONFLICT (key) DO NOTHING;
