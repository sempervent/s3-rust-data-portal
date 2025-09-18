-- Add compliance-related tables

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
    entry_id UUID NOT NULL REFERENCES repo_entry(id) ON DELETE CASCADE,
    reason TEXT NOT NULL,
    created_by UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ,
    status TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'released', 'expired'))
);

-- Audit logs
CREATE TABLE audit_log (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id),
    action TEXT NOT NULL,
    resource_type TEXT NOT NULL,
    resource_id UUID NOT NULL,
    details JSONB NOT NULL DEFAULT '{}',
    ip_address INET,
    user_agent TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Compliance exports
CREATE TABLE compliance_export (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    export_type TEXT NOT NULL CHECK (export_type IN ('audit_logs', 'retention_status', 'legal_holds', 'compliance_report')),
    filters JSONB NOT NULL DEFAULT '{}',
    status TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'processing', 'completed', 'failed')),
    file_path TEXT,
    created_by UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ
);

-- Add compliance fields to repo_entry
ALTER TABLE repo_entry
ADD COLUMN retention_policy_id UUID REFERENCES retention_policy(id),
ADD COLUMN retention_until TIMESTAMPTZ,
ADD COLUMN legal_hold BOOLEAN NOT NULL DEFAULT false;

-- Create indexes for performance
CREATE INDEX idx_legal_hold_entry_id ON legal_hold(entry_id);
CREATE INDEX idx_legal_hold_status ON legal_hold(status);
CREATE INDEX idx_audit_log_user_id ON audit_log(user_id);
CREATE INDEX idx_audit_log_action ON audit_log(action);
CREATE INDEX idx_audit_log_resource_type ON audit_log(resource_type);
CREATE INDEX idx_audit_log_created_at ON audit_log(created_at);
CREATE INDEX idx_compliance_export_status ON compliance_export(status);
CREATE INDEX idx_compliance_export_created_by ON compliance_export(created_by);
CREATE INDEX idx_repo_entry_retention_until ON repo_entry(retention_until);
CREATE INDEX idx_repo_entry_legal_hold ON repo_entry(legal_hold);

-- Create default retention policy (7 years)
INSERT INTO retention_policy (name, description, retention_days, legal_hold_override)
VALUES ('Default 7 Year Retention', 'Default retention policy for all entries', 2555, true);

-- Create function to automatically update updated_at timestamps
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Create trigger for retention_policy
CREATE TRIGGER update_retention_policy_updated_at
    BEFORE UPDATE ON retention_policy
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- Create function to log audit events
CREATE OR REPLACE FUNCTION log_audit_event(
    p_user_id UUID,
    p_action TEXT,
    p_resource_type TEXT,
    p_resource_id UUID,
    p_details JSONB DEFAULT '{}',
    p_ip_address INET DEFAULT NULL,
    p_user_agent TEXT DEFAULT NULL
)
RETURNS VOID AS $$
BEGIN
    INSERT INTO audit_log (user_id, action, resource_type, resource_id, details, ip_address, user_agent)
    VALUES (p_user_id, p_action, p_resource_type, p_resource_id, p_details, p_ip_address, p_user_agent);
END;
$$ LANGUAGE plpgsql;

-- Create function to check if entry can be deleted
CREATE OR REPLACE FUNCTION can_delete_entry(p_entry_id UUID)
RETURNS BOOLEAN AS $$
DECLARE
    v_legal_hold BOOLEAN;
    v_retention_until TIMESTAMPTZ;
BEGIN
    SELECT legal_hold, retention_until
    INTO v_legal_hold, v_retention_until
    FROM repo_entry
    WHERE id = p_entry_id;

    -- Cannot delete if under legal hold
    IF v_legal_hold THEN
        RETURN FALSE;
    END IF;

    -- Cannot delete if retention period not expired
    IF v_retention_until IS NOT NULL AND v_retention_until > NOW() THEN
        RETURN FALSE;
    END IF;

    RETURN TRUE;
END;
$$ LANGUAGE plpgsql;

-- Create function to get retention status summary
CREATE OR REPLACE FUNCTION get_retention_status_summary()
RETURNS JSONB AS $$
DECLARE
    v_result JSONB;
BEGIN
    SELECT jsonb_build_object(
        'total_entries', COUNT(*),
        'legal_hold_entries', COUNT(CASE WHEN legal_hold = true THEN 1 END),
        'expired_retention', COUNT(CASE WHEN retention_until IS NOT NULL AND retention_until <= NOW() THEN 1 END),
        'active_retention', COUNT(CASE WHEN retention_until IS NOT NULL AND retention_until > NOW() THEN 1 END)
    )
    INTO v_result
    FROM repo_entry;

    RETURN v_result;
END;
$$ LANGUAGE plpgsql;
