/**
 * BlackLake TypeScript SDK Types
 * 
 * Type definitions for the BlackLake Data Portal API.
 */

export interface Repository {
  id: string
  name: string
  description?: string
  created_at: string
  updated_at: string
  tenant_id?: string
}

export interface TreeEntry {
  path: string
  name: string
  type: 'file' | 'directory'
  size?: number
  modified_at?: string
  content_type?: string
  classification?: string
}

export interface TreeResponse {
  success: boolean
  data: TreeEntry[]
}

export interface SearchResult {
  id: string
  repo_name: string
  path: string
  name: string
  content_type?: string
  size?: number
  modified_at?: string
  classification?: string
  score?: number
  highlights?: Record<string, string[]>
}

export interface SearchFacet {
  name: string
  values: Array<{
    value: string
    count: number
  }>
}

export interface SearchResponse {
  success: boolean
  data: {
    results: SearchResult[]
    total: number
    limit: number
    offset: number
    facets?: SearchFacet[]
  }
}

export interface UploadInitResponse {
  upload_id: string
  presigned_url: string
  expires_at: string
}

export interface CommitResponse {
  commit_id: string
  message: string
  created_at: string
  files_count: number
}

export interface ExportResponse {
  export_id: string
  status: string
  created_at: string
  download_url?: string
}

export interface HealthResponse {
  status: string
  timestamp: string
  version?: string
  uptime?: number
}

export interface Policy {
  id: string
  tenant_id: string
  name: string
  effect: 'allow' | 'deny'
  actions: string[]
  resources: string[]
  condition?: Record<string, any>
  created_at: string
  updated_at: string
}

export interface Tenant {
  id: string
  name: string
  created_at: string
}

export interface SubjectAttribute {
  subject: string
  key: string
  value: string
  created_at: string
}

export interface ApiResponse<T = any> {
  success: boolean
  data: T
  error?: string
  message?: string
}

export interface ClientOptions {
  baseUrl?: string
  apiKey?: string
  timeout?: number
  verifySsl?: boolean
}

export interface SearchOptions {
  limit?: number
  offset?: number
  repo?: string
  classification?: string
}

export interface UploadOptions {
  path: string
  size: number
  content_type: string
}

export interface CommitOptions {
  upload_id: string
  message: string
  metadata?: Record<string, any>
}
