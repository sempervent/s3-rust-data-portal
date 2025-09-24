-- Fix missing columns discovered during compilation
-- Add missing columns to existing tables

-- Add id and created_at to entry table
ALTER TABLE entry ADD COLUMN IF NOT EXISTS id UUID DEFAULT gen_random_uuid();
ALTER TABLE entry ADD COLUMN IF NOT EXISTS created_at TIMESTAMPTZ DEFAULT NOW();

-- Add missing columns to webhook_deliveries
ALTER TABLE webhook_deliveries ADD COLUMN IF NOT EXISTS max_attempts INTEGER NOT NULL DEFAULT 5;
ALTER TABLE webhook_deliveries ADD COLUMN IF NOT EXISTS delivered_at TIMESTAMPTZ;
ALTER TABLE webhook_deliveries ADD COLUMN IF NOT EXISTS updated_at TIMESTAMPTZ DEFAULT NOW();

-- Fix webhook_dead table structure
ALTER TABLE webhook_dead ADD COLUMN IF NOT EXISTS failure_reason TEXT;
ALTER TABLE webhook_dead ADD COLUMN IF NOT EXISTS moved_at TIMESTAMPTZ DEFAULT NOW();

-- Fix export_jobs user_id type (already done via ALTER COLUMN TYPE)
-- ALTER TABLE export_jobs ALTER COLUMN user_id TYPE TEXT;

-- Add missing columns to check_results
ALTER TABLE check_results ADD COLUMN IF NOT EXISTS output JSONB;

-- Create indexes for new columns
CREATE INDEX IF NOT EXISTS idx_entry_created_at ON entry(created_at);
CREATE INDEX IF NOT EXISTS idx_webhook_deliveries_delivered_at ON webhook_deliveries(delivered_at);
CREATE INDEX IF NOT EXISTS idx_webhook_dead_moved_at ON webhook_dead(moved_at);

-- Update triggers for updated_at columns
CREATE TRIGGER update_webhook_deliveries_updated_at 
    BEFORE UPDATE ON webhook_deliveries 
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
