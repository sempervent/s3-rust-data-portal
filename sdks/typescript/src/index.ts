/**
 * BlackLake Data Portal TypeScript SDK
 * 
 * Enterprise data portal with multi-tenant access controls and governance.
 */

export { BlackLakeClient } from './client'
export { BlackLakeError, AuthenticationError, AuthorizationError, NotFoundError } from './exceptions'
export type {
  Repository,
  SearchResult,
  SearchResponse,
  TreeEntry,
  TreeResponse,
  UploadInitResponse,
  CommitResponse,
  ExportResponse,
  HealthResponse,
  Policy,
  Tenant,
  SubjectAttribute,
  ApiResponse,
  ClientOptions,
  SearchOptions,
  UploadOptions,
  CommitOptions,
} from './types'

export const VERSION = '0.1.0'
