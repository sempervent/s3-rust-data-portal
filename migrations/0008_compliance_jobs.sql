-- Add compliance jobs table for background processing

CREATE TABLE compliance_job (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    job_type TEXT NOT NULL CHECK (job_type IN ('retention_check', 'legal_hold_expiry', 'compliance_export', 'audit_log_cleanup', 'retention_policy_application')),
    status TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'running', 'completed', 'failed', 'cancelled')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    error_message TEXT,
    metadata JSONB NOT NULL DEFAULT '{}'
);

-- Create indexes for performance
CREATE INDEX idx_compliance_job_status ON compliance_job(status);
CREATE INDEX idx_compliance_job_type ON compliance_job(job_type);
CREATE INDEX idx_compliance_job_created_at ON compliance_job(created_at);

-- Create function to automatically create retention check jobs
CREATE OR REPLACE FUNCTION schedule_retention_check()
RETURNS VOID AS $$
BEGIN
    INSERT INTO compliance_job (job_type, status, metadata)
    VALUES ('retention_check', 'pending', '{}')
    ON CONFLICT DO NOTHING;
END;
$$ LANGUAGE plpgsql;

-- Create function to automatically create legal hold expiry check jobs
CREATE OR REPLACE FUNCTION schedule_legal_hold_expiry_check()
RETURNS VOID AS $$
BEGIN
    INSERT INTO compliance_job (job_type, status, metadata)
    VALUES ('legal_hold_expiry', 'pending', '{}')
    ON CONFLICT DO NOTHING;
END;
$$ LANGUAGE plpgsql;

-- Create function to automatically create audit log cleanup jobs
CREATE OR REPLACE FUNCTION schedule_audit_log_cleanup()
RETURNS VOID AS $$
BEGIN
    INSERT INTO compliance_job (job_type, status, metadata)
    VALUES ('audit_log_cleanup', 'pending', '{}')
    ON CONFLICT DO NOTHING;
END;
$$ LANGUAGE plpgsql;

-- Create a function to get compliance job statistics
CREATE OR REPLACE FUNCTION get_compliance_job_stats()
RETURNS JSONB AS $$
DECLARE
    v_result JSONB;
BEGIN
    SELECT jsonb_build_object(
        'total_jobs', COUNT(*),
        'pending_jobs', COUNT(CASE WHEN status = 'pending' THEN 1 END),
        'running_jobs', COUNT(CASE WHEN status = 'running' THEN 1 END),
        'completed_jobs', COUNT(CASE WHEN status = 'completed' THEN 1 END),
        'failed_jobs', COUNT(CASE WHEN status = 'failed' THEN 1 END),
        'jobs_by_type', jsonb_object_agg(job_type, type_count)
    )
    INTO v_result
    FROM (
        SELECT 
            job_type,
            COUNT(*) as type_count
        FROM compliance_job
        GROUP BY job_type
    ) type_stats
    CROSS JOIN (
        SELECT 
            COUNT(*) as total_jobs,
            COUNT(CASE WHEN status = 'pending' THEN 1 END) as pending_jobs,
            COUNT(CASE WHEN status = 'running' THEN 1 END) as running_jobs,
            COUNT(CASE WHEN status = 'completed' THEN 1 END) as completed_jobs,
            COUNT(CASE WHEN status = 'failed' THEN 1 END) as failed_jobs
        FROM compliance_job
    ) status_stats;

    RETURN v_result;
END;
$$ LANGUAGE plpgsql;

-- Create a function to clean up old completed jobs
CREATE OR REPLACE FUNCTION cleanup_old_compliance_jobs()
RETURNS INTEGER AS $$
DECLARE
    v_deleted_count INTEGER;
BEGIN
    DELETE FROM compliance_job 
    WHERE status IN ('completed', 'failed', 'cancelled')
    AND completed_at < NOW() - INTERVAL '30 days';
    
    GET DIAGNOSTICS v_deleted_count = ROW_COUNT;
    
    RETURN v_deleted_count;
END;
$$ LANGUAGE plpgsql;
