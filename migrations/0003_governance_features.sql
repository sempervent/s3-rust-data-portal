-- Week 4: Governance & Safety Rails
-- Branch protection, quotas, retention, webhooks, and export jobs

-- Branch protection rules
CREATE TABLE protected_refs (
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
CREATE TABLE repo_quota (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    repo_id UUID NOT NULL REFERENCES repo(id) ON DELETE CASCADE,
    bytes_soft BIGINT NOT NULL DEFAULT 1073741824, -- 1GB
    bytes_hard BIGINT NOT NULL DEFAULT 10737418240, -- 10GB
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(repo_id)
);

-- Repository usage tracking
CREATE TABLE repo_usage (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    repo_id UUID NOT NULL REFERENCES repo(id) ON DELETE CASCADE,
    current_bytes BIGINT NOT NULL DEFAULT 0,
    last_calculated TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(repo_id)
);

-- Retention policies
CREATE TABLE repo_retention (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    repo_id UUID NOT NULL REFERENCES repo(id) ON DELETE CASCADE,
    retention_policy JSONB NOT NULL DEFAULT '{"tombstone_days": 30, "hard_delete_days": 90, "legal_hold": false}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(repo_id)
);

-- Webhooks
CREATE TABLE webhooks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    repo_id UUID NOT NULL REFERENCES repo(id) ON DELETE CASCADE,
    url TEXT NOT NULL,
    secret TEXT NOT NULL,
    events JSONB NOT NULL DEFAULT '[]'::jsonb,
    active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Webhook delivery attempts
CREATE TABLE webhook_deliveries (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    webhook_id UUID NOT NULL REFERENCES webhooks(id) ON DELETE CASCADE,
    event_type TEXT NOT NULL,
    payload JSONB NOT NULL,
    response_status INTEGER,
    response_body TEXT,
    attempts INTEGER NOT NULL DEFAULT 0,
    max_attempts INTEGER NOT NULL DEFAULT 3,
    next_retry_at TIMESTAMPTZ,
    delivered_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Webhook dead letter queue
CREATE TABLE webhook_dead (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    webhook_id UUID NOT NULL REFERENCES webhooks(id) ON DELETE CASCADE,
    event_type TEXT NOT NULL,
    payload JSONB NOT NULL,
    failure_reason TEXT NOT NULL,
    attempts INTEGER NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Export jobs
CREATE TABLE export_jobs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    repo_id UUID NOT NULL REFERENCES repo(id) ON DELETE CASCADE,
    user_id TEXT NOT NULL,
    manifest JSONB NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'running', 'completed', 'failed')),
    s3_key TEXT,
    download_url TEXT,
    error_message TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Check results for branch protection
CREATE TABLE check_results (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    repo_id UUID NOT NULL REFERENCES repo(id) ON DELETE CASCADE,
    ref_name TEXT NOT NULL,
    commit_id UUID NOT NULL REFERENCES commit(id) ON DELETE CASCADE,
    check_name TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'success', 'failure', 'error')),
    details_url TEXT,
    output TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for performance
CREATE INDEX idx_protected_refs_repo_ref ON protected_refs(repo_id, ref_name);
CREATE INDEX idx_repo_quota_repo ON repo_quota(repo_id);
CREATE INDEX idx_repo_usage_repo ON repo_usage(repo_id);
CREATE INDEX idx_repo_retention_repo ON repo_retention(repo_id);
CREATE INDEX idx_webhooks_repo ON webhooks(repo_id);
CREATE INDEX idx_webhooks_active ON webhooks(active) WHERE active = true;
CREATE INDEX idx_webhook_deliveries_webhook ON webhook_deliveries(webhook_id);
CREATE INDEX idx_webhook_deliveries_retry ON webhook_deliveries(next_retry_at) WHERE next_retry_at IS NOT NULL;
CREATE INDEX idx_webhook_dead_webhook ON webhook_dead(webhook_id);
CREATE INDEX idx_export_jobs_repo ON export_jobs(repo_id);
CREATE INDEX idx_export_jobs_status ON export_jobs(status);
CREATE INDEX idx_check_results_repo_ref ON check_results(repo_id, ref_name);
CREATE INDEX idx_check_results_commit ON check_results(commit_id);

-- Triggers for updated_at
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER update_protected_refs_updated_at BEFORE UPDATE ON protected_refs FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_repo_quota_updated_at BEFORE UPDATE ON repo_quota FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_repo_retention_updated_at BEFORE UPDATE ON repo_retention FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_webhooks_updated_at BEFORE UPDATE ON webhooks FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_export_jobs_updated_at BEFORE UPDATE ON export_jobs FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_check_results_updated_at BEFORE UPDATE ON check_results FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Initialize default quotas for existing repos
INSERT INTO repo_quota (repo_id, bytes_soft, bytes_hard)
SELECT id, 1073741824, 10737418240 FROM repo
ON CONFLICT (repo_id) DO NOTHING;

-- Initialize default usage tracking for existing repos
INSERT INTO repo_usage (repo_id, current_bytes)
SELECT id, 0 FROM repo
ON CONFLICT (repo_id) DO NOTHING;

-- Initialize default retention policies for existing repos
INSERT INTO repo_retention (repo_id, retention_policy)
SELECT id, '{"tombstone_days": 30, "hard_delete_days": 90, "legal_hold": false}'::jsonb FROM repo
ON CONFLICT (repo_id) DO NOTHING;