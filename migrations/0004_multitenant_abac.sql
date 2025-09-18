-- Migration: Multi-tenant Access Control & ABAC
-- Week 7: Enterprise hardening with tenant isolation and attribute-based access control

-- Create tenants table
CREATE TABLE IF NOT EXISTS tenants (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT UNIQUE NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Add tenant_id to repos table (nullable for backward compatibility)
ALTER TABLE repos ADD COLUMN IF NOT EXISTS tenant_id UUID REFERENCES tenants(id) ON DELETE SET NULL;

-- Create policies table for ABAC
CREATE TABLE IF NOT EXISTS policies (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    effect TEXT NOT NULL CHECK (effect IN ('allow', 'deny')),
    actions TEXT[] NOT NULL DEFAULT '{}',
    resources TEXT[] NOT NULL DEFAULT '{}',
    condition JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(tenant_id, name)
);

-- Create subject attributes table for caching user/group attributes
CREATE TABLE IF NOT EXISTS subject_attributes (
    subject TEXT NOT NULL,
    key TEXT NOT NULL,
    value TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (subject, key)
);

-- Add classification field to metadata
ALTER TABLE metadata ADD COLUMN IF NOT EXISTS classification TEXT DEFAULT 'internal' 
    CHECK (classification IN ('public', 'internal', 'restricted', 'secret'));

-- Create indexes for performance
CREATE INDEX IF NOT EXISTS idx_repos_tenant_id ON repos(tenant_id);
CREATE INDEX IF NOT EXISTS idx_policies_tenant_id ON policies(tenant_id);
CREATE INDEX IF NOT EXISTS idx_policies_effect ON policies(effect);
CREATE INDEX IF NOT EXISTS idx_subject_attributes_subject ON subject_attributes(subject);
CREATE INDEX IF NOT EXISTS idx_metadata_classification ON metadata(classification);

-- Create audit table for policy decisions
CREATE TABLE IF NOT EXISTS policy_audit (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    subject TEXT NOT NULL,
    action TEXT NOT NULL,
    resource TEXT NOT NULL,
    policy_id UUID REFERENCES policies(id),
    decision TEXT NOT NULL CHECK (decision IN ('allow', 'deny')),
    reason TEXT,
    context JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_policy_audit_subject ON policy_audit(subject);
CREATE INDEX IF NOT EXISTS idx_policy_audit_decision ON policy_audit(decision);
CREATE INDEX IF NOT EXISTS idx_policy_audit_created_at ON policy_audit(created_at);

-- Create default tenant for existing repos
INSERT INTO tenants (id, name) VALUES ('00000000-0000-0000-0000-000000000000', 'default')
ON CONFLICT (name) DO NOTHING;

-- Assign existing repos to default tenant
UPDATE repos SET tenant_id = '00000000-0000-0000-0000-000000000000' WHERE tenant_id IS NULL;

-- Create default policies for the default tenant
INSERT INTO policies (tenant_id, name, effect, actions, resources, condition) VALUES
    ('00000000-0000-0000-0000-000000000000', 'default-allow-read', 'allow', 
     ARRAY['read', 'search'], ARRAY['*'], 
     '{"subject.roles": {"$contains": "user"}}'),
    ('00000000-0000-0000-0000-000000000000', 'default-allow-write', 'allow', 
     ARRAY['write', 'upload', 'commit'], ARRAY['*'], 
     '{"subject.roles": {"$contains": "user"}}'),
    ('00000000-0000-0000-0000-000000000000', 'default-allow-admin', 'allow', 
     ARRAY['admin', 'delete', 'export'], ARRAY['*'], 
     '{"subject.roles": {"$contains": "admin"}}')
ON CONFLICT (tenant_id, name) DO NOTHING;

-- Add triggers for updated_at
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER update_tenants_updated_at BEFORE UPDATE ON tenants
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_policies_updated_at BEFORE UPDATE ON policies
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_subject_attributes_updated_at BEFORE UPDATE ON subject_attributes
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Add comments for documentation
COMMENT ON TABLE tenants IS 'Multi-tenant isolation boundary';
COMMENT ON TABLE policies IS 'Attribute-based access control policies';
COMMENT ON TABLE subject_attributes IS 'Cached user/group attributes for policy evaluation';
COMMENT ON TABLE policy_audit IS 'Audit log for policy decisions';
COMMENT ON COLUMN repos.tenant_id IS 'Tenant assignment for multi-tenant isolation';
COMMENT ON COLUMN metadata.classification IS 'Data classification level for policy conditions';
