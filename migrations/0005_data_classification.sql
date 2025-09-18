-- Add data classification field to metadata index
-- Week 7: Data Classification & Guardrails

-- Add classification enum
CREATE TYPE data_classification AS ENUM ('public', 'internal', 'restricted', 'secret');

-- Add classification field to metadata index
ALTER TABLE metadata_index
ADD COLUMN classification data_classification NOT NULL DEFAULT 'internal';

-- Add index for classification filtering
CREATE INDEX idx_metadata_classification ON metadata_index(classification);

-- Add classification to search index (Solr will be updated separately)
-- This field will be used for policy conditions and search filtering
