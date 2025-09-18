-- Data lineage tracking
CREATE TABLE entry_lineage (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  repo_id UUID REFERENCES repo(id) ON DELETE CASCADE,
  commit_id UUID REFERENCES commit(id) ON DELETE CASCADE,
  path TEXT NOT NULL,
  parent_paths TEXT[] NOT NULL DEFAULT '{}',
  child_paths TEXT[] NOT NULL DEFAULT '{}',
  lineage_type TEXT NOT NULL CHECK (lineage_type IN ('derived', 'transformed', 'aggregated', 'filtered', 'joined')),
  lineage_metadata JSONB,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  UNIQUE(repo_id, commit_id, path)
);

-- Repository quotas
CREATE TABLE repo_quota (
  repo_id UUID PRIMARY KEY REFERENCES repo(id) ON DELETE CASCADE,
  max_size_bytes BIGINT NOT NULL DEFAULT 10737418240, -- 10GB default
  max_files INTEGER NOT NULL DEFAULT 10000,
  max_commits INTEGER NOT NULL DEFAULT 1000,
  current_size_bytes BIGINT NOT NULL DEFAULT 0,
  current_files INTEGER NOT NULL DEFAULT 0,
  current_commits INTEGER NOT NULL DEFAULT 0,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- User quotas
CREATE TABLE user_quota (
  user_id TEXT PRIMARY KEY,
  max_repos INTEGER NOT NULL DEFAULT 10,
  max_total_size_bytes BIGINT NOT NULL DEFAULT 107374182400, -- 100GB default
  current_repos INTEGER NOT NULL DEFAULT 0,
  current_total_size_bytes BIGINT NOT NULL DEFAULT 0,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Quota usage tracking
CREATE TABLE quota_usage_log (
  id BIGSERIAL PRIMARY KEY,
  user_id TEXT NOT NULL,
  repo_id UUID REFERENCES repo(id) ON DELETE CASCADE,
  action TEXT NOT NULL CHECK (action IN ('create_repo', 'upload_file', 'delete_file', 'create_commit')),
  size_delta BIGINT NOT NULL DEFAULT 0,
  file_delta INTEGER NOT NULL DEFAULT 0,
  commit_delta INTEGER NOT NULL DEFAULT 0,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Indexes for performance
CREATE INDEX idx_entry_lineage_repo_commit ON entry_lineage(repo_id, commit_id);
CREATE INDEX idx_entry_lineage_parents ON entry_lineage USING GIN(parent_paths);
CREATE INDEX idx_entry_lineage_children ON entry_lineage USING GIN(child_paths);
CREATE INDEX idx_quota_usage_user ON quota_usage_log(user_id);
CREATE INDEX idx_quota_usage_repo ON quota_usage_log(repo_id);
CREATE INDEX idx_quota_usage_created ON quota_usage_log(created_at);

-- Functions for quota management
CREATE OR REPLACE FUNCTION update_repo_quota_usage()
RETURNS TRIGGER AS $$
BEGIN
  -- Update repo quota usage
  INSERT INTO repo_quota (repo_id, max_size_bytes, max_files, max_commits, current_size_bytes, current_files, current_commits)
  VALUES (NEW.repo_id, 10737418240, 10000, 1000, 0, 0, 0)
  ON CONFLICT (repo_id) DO NOTHING;

  -- Update user quota usage
  INSERT INTO user_quota (user_id, max_repos, max_total_size_bytes, current_repos, current_total_size_bytes)
  VALUES (NEW.created_by, 10, 107374182400, 0, 0)
  ON CONFLICT (user_id) DO NOTHING;

  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_update_repo_quota_usage
  AFTER INSERT ON repo
  FOR EACH ROW
  EXECUTE FUNCTION update_repo_quota_usage();

-- Function to check quota limits
CREATE OR REPLACE FUNCTION check_repo_quota(
  p_repo_id UUID,
  p_size_delta BIGINT,
  p_file_delta INTEGER,
  p_commit_delta INTEGER
) RETURNS BOOLEAN AS $$
DECLARE
  quota_record repo_quota%ROWTYPE;
BEGIN
  SELECT * INTO quota_record FROM repo_quota WHERE repo_id = p_repo_id;
  
  IF NOT FOUND THEN
    RETURN TRUE; -- No quota set, allow
  END IF;
  
  -- Check size limit
  IF quota_record.current_size_bytes + p_size_delta > quota_record.max_size_bytes THEN
    RETURN FALSE;
  END IF;
  
  -- Check file limit
  IF quota_record.current_files + p_file_delta > quota_record.max_files THEN
    RETURN FALSE;
  END IF;
  
  -- Check commit limit
  IF quota_record.current_commits + p_commit_delta > quota_record.max_commits THEN
    RETURN FALSE;
  END IF;
  
  RETURN TRUE;
END;
$$ LANGUAGE plpgsql;

-- Function to check user quota
CREATE OR REPLACE FUNCTION check_user_quota(
  p_user_id TEXT,
  p_repo_delta INTEGER,
  p_size_delta BIGINT
) RETURNS BOOLEAN AS $$
DECLARE
  quota_record user_quota%ROWTYPE;
BEGIN
  SELECT * INTO quota_record FROM user_quota WHERE user_id = p_user_id;
  
  IF NOT FOUND THEN
    RETURN TRUE; -- No quota set, allow
  END IF;
  
  -- Check repo limit
  IF quota_record.current_repos + p_repo_delta > quota_record.max_repos THEN
    RETURN FALSE;
  END IF;
  
  -- Check total size limit
  IF quota_record.current_total_size_bytes + p_size_delta > quota_record.max_total_size_bytes THEN
    RETURN FALSE;
  END IF;
  
  RETURN TRUE;
END;
$$ LANGUAGE plpgsql;

-- Function to update quota usage
CREATE OR REPLACE FUNCTION update_quota_usage(
  p_user_id TEXT,
  p_repo_id UUID,
  p_action TEXT,
  p_size_delta BIGINT DEFAULT 0,
  p_file_delta INTEGER DEFAULT 0,
  p_commit_delta INTEGER DEFAULT 0
) RETURNS VOID AS $$
BEGIN
  -- Log quota usage
  INSERT INTO quota_usage_log (user_id, repo_id, action, size_delta, file_delta, commit_delta)
  VALUES (p_user_id, p_repo_id, p_action, p_size_delta, p_file_delta, p_commit_delta);
  
  -- Update repo quota
  UPDATE repo_quota 
  SET 
    current_size_bytes = current_size_bytes + p_size_delta,
    current_files = current_files + p_file_delta,
    current_commits = current_commits + p_commit_delta,
    updated_at = now()
  WHERE repo_id = p_repo_id;
  
  -- Update user quota
  UPDATE user_quota 
  SET 
    current_repos = current_repos + CASE WHEN p_action = 'create_repo' THEN 1 ELSE 0 END,
    current_total_size_bytes = current_total_size_bytes + p_size_delta,
    updated_at = now()
  WHERE user_id = p_user_id;
END;
$$ LANGUAGE plpgsql;
