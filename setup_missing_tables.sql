-- Create missing tables for SQLx compilation
-- Based on the errors we're seeing

-- Protected refs table
CREATE TABLE IF NOT EXISTS protected_refs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    repo_id UUID NOT NULL REFERENCES repo(id) ON DELETE CASCADE,
    ref_name TEXT NOT NULL,
    require_admin BOOLEAN NOT NULL DEFAULT false,
    allow_fast_forward BOOLEAN NOT NULL DEFAULT true,
    allow_delete BOOLEAN NOT NULL DEFAULT false,
    required_checks JSONB NOT NULL DEFAULT '[]'::jsonb,
    required_reviewers INTEGER NOT NULL DEFAULT 0,
    require_schema_pass BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(repo_id, ref_name)
);

-- Repository quotas
CREATE TABLE IF NOT EXISTS repo_quota (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    repo_id UUID NOT NULL REFERENCES repo(id) ON DELETE CASCADE,
    bytes_soft BIGINT NOT NULL DEFAULT 1073741824, -- 1GB
    bytes_hard BIGINT NOT NULL DEFAULT 10737418240, -- 10GB
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(repo_id)
);

-- Repository usage tracking
CREATE TABLE IF NOT EXISTS repo_usage (
    repo_id UUID PRIMARY KEY REFERENCES repo(id) ON DELETE CASCADE,
    current_bytes BIGINT NOT NULL DEFAULT 0,
    last_calculated TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Repository retention policies
CREATE TABLE IF NOT EXISTS repo_retention (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    repo_id UUID NOT NULL REFERENCES repo(id) ON DELETE CASCADE,
    retention_policy JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(repo_id)
);

-- Check results
CREATE TABLE IF NOT EXISTS check_results (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    repo_id UUID NOT NULL REFERENCES repo(id) ON DELETE CASCADE,
    ref_name TEXT NOT NULL,
    commit_id UUID NOT NULL REFERENCES commit(id) ON DELETE CASCADE,
    check_name TEXT NOT NULL,
    status TEXT NOT NULL,
    details_url TEXT,
    output TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(repo_id, ref_name, commit_id, check_name)
);

-- Webhooks
CREATE TABLE IF NOT EXISTS webhooks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    repo_id UUID NOT NULL REFERENCES repo(id) ON DELETE CASCADE,
    url TEXT NOT NULL,
    secret TEXT NOT NULL,
    events JSONB NOT NULL,
    active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Webhook deliveries
CREATE TABLE IF NOT EXISTS webhook_deliveries (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    webhook_id UUID NOT NULL REFERENCES webhooks(id) ON DELETE CASCADE,
    event TEXT NOT NULL,
    payload JSONB NOT NULL,
    status TEXT NOT NULL,
    attempts INTEGER NOT NULL DEFAULT 0,
    max_attempts INTEGER NOT NULL DEFAULT 3,
    last_attempt_at TIMESTAMPTZ,
    next_retry_at TIMESTAMPTZ,
    response_status INTEGER,
    response_body TEXT,
    error_message TEXT,
    delivered_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Webhook dead letter
CREATE TABLE IF NOT EXISTS webhook_dead (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    webhook_id UUID NOT NULL REFERENCES webhooks(id) ON DELETE CASCADE,
    event TEXT NOT NULL,
    payload JSONB NOT NULL,
    attempts INTEGER NOT NULL DEFAULT 0,
    failure_reason TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    moved_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Export jobs
CREATE TABLE IF NOT EXISTS export_jobs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    repo_id UUID NOT NULL REFERENCES repo(id) ON DELETE CASCADE,
    user_id UUID NOT NULL,
    manifest JSONB NOT NULL,
    status TEXT NOT NULL,
    progress INTEGER NOT NULL DEFAULT 0,
    total_items INTEGER,
    processed_items INTEGER,
    output_size BIGINT,
    s3_key TEXT,
    download_url TEXT,
    error_message TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    expires_at TIMESTAMPTZ
);

-- Create indexes for performance
CREATE INDEX IF NOT EXISTS idx_protected_refs_repo_id ON protected_refs(repo_id);
CREATE INDEX IF NOT EXISTS idx_repo_quota_repo_id ON repo_quota(repo_id);
CREATE INDEX IF NOT EXISTS idx_repo_usage_repo_id ON repo_usage(repo_id);
CREATE INDEX IF NOT EXISTS idx_repo_retention_repo_id ON repo_retention(repo_id);
CREATE INDEX IF NOT EXISTS idx_check_results_repo_id ON check_results(repo_id);
CREATE INDEX IF NOT EXISTS idx_webhooks_repo_id ON webhooks(repo_id);
CREATE INDEX IF NOT EXISTS idx_webhook_deliveries_webhook_id ON webhook_deliveries(webhook_id);
CREATE INDEX IF NOT EXISTS idx_webhook_dead_webhook_id ON webhook_dead(webhook_id);
CREATE INDEX IF NOT EXISTS idx_export_jobs_repo_id ON export_jobs(repo_id);
