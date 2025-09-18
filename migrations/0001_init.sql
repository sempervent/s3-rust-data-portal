-- Enable UUID extension
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- repositories
CREATE TABLE repo (
  id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  name TEXT UNIQUE NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  created_by TEXT NOT NULL
);

-- references (branches and tags)
CREATE TABLE ref (
  repo_id UUID REFERENCES repo(id) ON DELETE CASCADE,
  name TEXT NOT NULL,
  kind TEXT NOT NULL CHECK (kind IN ('branch','tag')),
  commit_id UUID NOT NULL,
  PRIMARY KEY (repo_id, name)
);

-- commits (immutable)
CREATE TABLE commit (
  id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  repo_id UUID REFERENCES repo(id) ON DELETE CASCADE,
  parent_id UUID,
  author TEXT NOT NULL,
  message TEXT,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  stats JSONB
);

-- objects (content-addressed)
CREATE TABLE object (
  sha256 TEXT PRIMARY KEY,
  size BIGINT NOT NULL,
  media_type TEXT,
  s3_key TEXT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- tree entries (path bindings per commit)
CREATE TABLE entry (
  commit_id UUID REFERENCES commit(id) ON DELETE CASCADE,
  path TEXT NOT NULL,
  object_sha256 TEXT REFERENCES object(sha256),
  meta JSONB NOT NULL,
  is_dir BOOLEAN NOT NULL DEFAULT FALSE,
  PRIMARY KEY (commit_id, path)
);

-- ACLs (coarse repo-level for MVP)
CREATE TABLE acl (
  repo_id UUID REFERENCES repo(id) ON DELETE CASCADE,
  subject TEXT NOT NULL,
  perm TEXT NOT NULL CHECK (perm IN ('read','write','admin')),
  PRIMARY KEY (repo_id, subject, perm)
);

-- audit log
CREATE TABLE audit_log (
  id BIGSERIAL PRIMARY KEY,
  at TIMESTAMPTZ NOT NULL DEFAULT now(),
  actor TEXT NOT NULL,
  action TEXT NOT NULL,
  repo_name TEXT,
  ref_name TEXT,
  path TEXT,
  request_meta JSONB,
  response_meta JSONB
);

-- indexes
CREATE INDEX idx_entry_meta_gin ON entry USING GIN (meta);
CREATE INDEX idx_entry_path ON entry(path);
CREATE INDEX idx_object_media ON object(media_type);
CREATE INDEX idx_commit_repo ON commit(repo_id);
CREATE INDEX idx_ref_repo ON ref(repo_id);
CREATE INDEX idx_audit_log_at ON audit_log(at);
CREATE INDEX idx_audit_log_actor ON audit_log(actor);
