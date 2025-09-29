-- Week 7: Missing tables for API crate
-- External sources, retention policies, legal holds, and compliance exports

-- External data sources
CREATE TABLE external_source (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL UNIQUE,
    description TEXT,
    connector_type TEXT NOT NULL,
    config JSONB NOT NULL DEFAULT '{}'::jsonb,
    enabled BOOLEAN NOT NULL DEFAULT true,
    sync_interval_minutes INTEGER NOT NULL DEFAULT 60,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Retention policies
CREATE TABLE retention_policy (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL UNIQUE,
    description TEXT,
    retention_days INTEGER NOT NULL,
    legal_hold_override BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Legal holds
CREATE TABLE legal_hold (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    entry_id UUID NOT NULL,
    reason TEXT NOT NULL,
    created_by UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ,
    status TEXT NOT NULL DEFAULT 'active'
);

-- Compliance exports
CREATE TABLE compliance_export (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    export_type TEXT NOT NULL,
    filters JSONB NOT NULL DEFAULT '{}'::jsonb,
    status TEXT NOT NULL DEFAULT 'pending',
    file_path TEXT,
    created_by UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ
);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_external_source_name ON external_source(name);
CREATE INDEX IF NOT EXISTS idx_external_source_enabled ON external_source(enabled);
CREATE INDEX IF NOT EXISTS idx_retention_policy_name ON retention_policy(name);
CREATE INDEX IF NOT EXISTS idx_legal_hold_entry_id ON legal_hold(entry_id);
CREATE INDEX IF NOT EXISTS idx_legal_hold_status ON legal_hold(status);
CREATE INDEX IF NOT EXISTS idx_compliance_export_type ON compliance_export(export_type);
CREATE INDEX IF NOT EXISTS idx_compliance_export_status ON compliance_export(status);
CREATE INDEX IF NOT EXISTS idx_compliance_export_created_by ON compliance_export(created_by);

-- Update triggers
CREATE TRIGGER update_external_source_updated_at BEFORE UPDATE ON external_source FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_retention_policy_updated_at BEFORE UPDATE ON retention_policy FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
