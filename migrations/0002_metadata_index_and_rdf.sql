-- Fast-path index table for common metadata fields
CREATE TABLE entry_meta_index (
  commit_id UUID NOT NULL,
  path TEXT NOT NULL,
  -- denormalized, nullable to keep optionality intact
  creation_dt TIMESTAMPTZ,
  creator TEXT,
  file_name TEXT,
  file_type TEXT,
  file_size BIGINT,
  org_lab TEXT,
  description TEXT,
  data_source TEXT,
  data_collection_method TEXT,
  version TEXT,
  notes TEXT,
  tags TEXT[],
  license TEXT,
  PRIMARY KEY (commit_id, path),
  FOREIGN KEY (commit_id, path) REFERENCES entry(commit_id, path) ON DELETE CASCADE
);

-- Useful indexes
CREATE INDEX idx_meta_index_org ON entry_meta_index(org_lab);
CREATE INDEX idx_meta_index_ftype ON entry_meta_index(file_type);
CREATE INDEX idx_meta_index_ctime ON entry_meta_index(creation_dt);
CREATE INDEX idx_meta_index_tags ON entry_meta_index USING GIN(tags);
CREATE INDEX idx_meta_index_fname ON entry_meta_index(file_name);

-- Repo feature flags (per-repo configuration)
ALTER TABLE repo ADD COLUMN features JSONB NOT NULL DEFAULT '{}'::jsonb;

-- RDF storage table (optional materialization)
CREATE TABLE artifact_rdf (
  commit_id UUID NOT NULL,
  path TEXT NOT NULL,
  format TEXT NOT NULL CHECK (format IN ('turtle','jsonld')),
  graph TEXT NOT NULL,              -- serialized RDF
  graph_sha256 TEXT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  PRIMARY KEY (commit_id, path, format),
  FOREIGN KEY (commit_id, path) REFERENCES entry(commit_id, path) ON DELETE CASCADE
);

CREATE INDEX idx_artifact_rdf_sha ON artifact_rdf(graph_sha256);
