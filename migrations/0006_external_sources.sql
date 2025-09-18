-- Add external sources table for federation
-- Week 8: Federation across data sources

-- External sources table
CREATE TABLE external_source (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL,
    description TEXT,
    connector_type TEXT NOT NULL CHECK (connector_type IN ('s3', 'postgres', 'ckan')),
    config JSONB NOT NULL,
    enabled BOOLEAN NOT NULL DEFAULT true,
    sync_interval_minutes INTEGER NOT NULL DEFAULT 60,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- External entries table (indexed in Solr)
CREATE TABLE external_entry (
    id TEXT PRIMARY KEY, -- Format: connector_type:source_id:entry_id
    source_id UUID NOT NULL REFERENCES external_source(id) ON DELETE CASCADE,
    title TEXT NOT NULL,
    description TEXT,
    url TEXT NOT NULL,
    content_type TEXT,
    size BIGINT,
    modified_at TIMESTAMPTZ,
    tags TEXT[],
    metadata JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Sync history table
CREATE TABLE external_sync_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    source_id UUID NOT NULL REFERENCES external_source(id) ON DELETE CASCADE,
    started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ,
    status TEXT NOT NULL CHECK (status IN ('running', 'completed', 'failed')),
    entries_processed BIGINT DEFAULT 0,
    entries_added BIGINT DEFAULT 0,
    entries_updated BIGINT DEFAULT 0,
    entries_removed BIGINT DEFAULT 0,
    error_message TEXT,
    duration_seconds FLOAT
);

-- Indexes for performance
CREATE INDEX idx_external_source_type ON external_source(connector_type);
CREATE INDEX idx_external_source_enabled ON external_source(enabled);
CREATE INDEX idx_external_entry_source_id ON external_entry(source_id);
CREATE INDEX idx_external_entry_modified_at ON external_entry(modified_at);
CREATE INDEX idx_external_entry_tags ON external_entry USING GIN(tags);
CREATE INDEX idx_external_entry_metadata ON external_entry USING GIN(metadata);
CREATE INDEX idx_external_sync_history_source_id ON external_sync_history(source_id);
CREATE INDEX idx_external_sync_history_started_at ON external_sync_history(started_at);

-- Add classification to external entries
ALTER TABLE external_entry
ADD COLUMN classification data_classification NOT NULL DEFAULT 'internal';

-- Add embeddings field for AI-assisted search
ALTER TABLE external_entry
ADD COLUMN embeddings VECTOR(384); -- 384-dimensional vector for MiniLM

-- Add index for vector similarity search
CREATE INDEX idx_external_entry_embeddings ON external_entry USING ivfflat (embeddings vector_cosine_ops) WITH (lists = 100);

-- Add retention and legal hold fields
ALTER TABLE external_entry
ADD COLUMN retention_until TIMESTAMPTZ,
ADD COLUMN legal_hold BOOLEAN NOT NULL DEFAULT false;

-- Add indexes for compliance
CREATE INDEX idx_external_entry_retention_until ON external_entry(retention_until);
CREATE INDEX idx_external_entry_legal_hold ON external_entry(legal_hold);
