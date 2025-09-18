export interface Repository {
  id: string;
  name: string;
  created_at: string;
  created_by: string;
}

export interface Commit {
  id: string;
  repo_id: string;
  message: string;
  author: string;
  created_at: string;
  parent_id?: string;
}

export interface TreeEntry {
  path: string;
  sha256: string;
  size: number;
  is_dir: boolean;
  meta?: Record<string, any>;
}

export interface TreeResponse {
  entries: TreeEntry[];
  total: number;
}

export interface SearchRequest {
  query?: string;
  file_type?: string;
  org_lab?: string;
  tags?: string[];
  created_after?: string;
  created_before?: string;
  limit?: number;
  offset?: number;
}

export interface SearchResponse {
  entries: TreeEntry[];
  total: number;
  limit: number;
  offset: number;
}

export interface UploadInitRequest {
  path: string;
  size: number;
  media_type?: string;
}

export interface UploadInitResponse {
  upload_url: string;
  sha256: string;
  expires_at: string;
}

export interface CommitRequest {
  ref: string;
  message: string;
  changes: Change[];
}

export interface Change {
  op: 'put' | 'delete';
  path: string;
  sha256?: string;
  meta?: Record<string, any>;
}

export interface CommitResponse {
  commit_id: string;
  message: string;
  author: string;
  created_at: string;
}

export interface User {
  sub: string;
  name?: string;
  email?: string;
  groups?: string[];
}

export interface AuthState {
  user: User | null;
  token: string | null;
  isAuthenticated: boolean;
  isLoading: boolean;
}

export interface AppState {
  selectedRepo: string | null;
  selectedBranch: string;
  selectedPath: string;
}
