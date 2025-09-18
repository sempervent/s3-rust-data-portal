/**
 * BlackLake TypeScript SDK Client
 * 
 * Main client for interacting with the BlackLake Data Portal API.
 */

import axios, { AxiosInstance, AxiosRequestConfig, AxiosResponse } from 'axios'
import {
  Repository,
  SearchResponse,
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
import { BlackLakeError, AuthenticationError, AuthorizationError, NotFoundError } from './exceptions'

export class BlackLakeClient {
  private client: AxiosInstance
  private baseUrl: string
  private apiKey?: string

  constructor(options: ClientOptions = {}) {
    this.baseUrl = options.baseUrl || 'http://localhost:8080'
    this.apiKey = options.apiKey

    this.client = axios.create({
      baseURL: this.baseUrl,
      timeout: options.timeout || 30000,
      headers: {
        'Content-Type': 'application/json',
        'User-Agent': `blacklake-sdk-typescript/0.1.0`,
      },
    })

    // Add request interceptor for authentication
    this.client.interceptors.request.use((config) => {
      if (this.apiKey) {
        config.headers.Authorization = `Bearer ${this.apiKey}`
      }
      return config
    })

    // Add response interceptor for error handling
    this.client.interceptors.response.use(
      (response) => response,
      (error) => {
        if (error.response) {
          const { status, data } = error.response
          
          switch (status) {
            case 401:
              throw new AuthenticationError('Authentication failed')
            case 403:
              throw new AuthorizationError('Access denied')
            case 404:
              throw new NotFoundError('Resource not found')
            default:
              const message = data?.error || `HTTP ${status}`
              throw new BlackLakeError(`API error: ${message}`)
          }
        } else if (error.request) {
          throw new BlackLakeError('Network error: No response received')
        } else {
          throw new BlackLakeError(`Request error: ${error.message}`)
        }
      }
    )
  }

  private async request<T>(config: AxiosRequestConfig): Promise<T> {
    try {
      const response: AxiosResponse<T> = await this.client.request(config)
      return response.data
    } catch (error) {
      if (error instanceof BlackLakeError) {
        throw error
      }
      throw new BlackLakeError(`Request failed: ${error}`)
    }
  }

  // Repository operations

  async listRepositories(): Promise<Repository[]> {
    const response = await this.request<ApiResponse<Repository[]>>({
      method: 'GET',
      url: '/v1/repos',
    })
    return response.data
  }

  async getRepository(name: string): Promise<Repository> {
    const response = await this.request<ApiResponse<Repository>>({
      method: 'GET',
      url: `/v1/repos/${name}`,
    })
    return response.data
  }

  async createRepository(name: string, description?: string): Promise<Repository> {
    const response = await this.request<ApiResponse<Repository>>({
      method: 'POST',
      url: '/v1/repos',
      data: { name, description },
    })
    return response.data
  }

  async getRepositoryTree(
    repoName: string,
    ref: string = 'main',
    path?: string
  ): Promise<TreeResponse> {
    const params: Record<string, any> = {}
    if (path) {
      params.path = path
    }

    const response = await this.request<TreeResponse>({
      method: 'GET',
      url: `/v1/repos/${repoName}/tree/${ref}`,
      params,
    })
    return response
  }

  // Search operations

  async search(query: string, options: SearchOptions = {}): Promise<SearchResponse> {
    const params = {
      q: query,
      limit: options.limit || 20,
      offset: options.offset || 0,
      ...(options.repo && { repo: options.repo }),
      ...(options.classification && { classification: options.classification }),
    }

    const response = await this.request<SearchResponse>({
      method: 'GET',
      url: '/v1/search',
      params,
    })
    return response
  }

  async searchSuggestions(query: string, count: number = 10): Promise<string[]> {
    const response = await this.request<ApiResponse<string[]>>({
      method: 'GET',
      url: '/v1/search/suggest',
      params: { q: query, count },
    })
    return response.data
  }

  // File operations

  async getFileMetadata(repoName: string, ref: string, path: string): Promise<Record<string, any>> {
    const response = await this.request<ApiResponse<Record<string, any>>>({
      method: 'GET',
      url: `/v1/repos/${repoName}/metadata/${ref}/${path}`,
    })
    return response.data
  }

  async updateFileMetadata(
    repoName: string,
    ref: string,
    path: string,
    metadata: Record<string, any>
  ): Promise<Record<string, any>> {
    const response = await this.request<ApiResponse<Record<string, any>>>({
      method: 'PUT',
      url: `/v1/repos/${repoName}/metadata/${ref}/${path}`,
      data: metadata,
    })
    return response.data
  }

  // Upload operations

  async initiateUpload(repoName: string, options: UploadOptions): Promise<UploadInitResponse> {
    const response = await this.request<ApiResponse<UploadInitResponse>>({
      method: 'POST',
      url: `/v1/repos/${repoName}/upload/init`,
      data: options,
    })
    return response.data
  }

  async commitUpload(repoName: string, options: CommitOptions): Promise<CommitResponse> {
    const response = await this.request<ApiResponse<CommitResponse>>({
      method: 'POST',
      url: `/v1/repos/${repoName}/commit`,
      data: options,
    })
    return response.data
  }

  // Export operations

  async exportRepository(
    repoName: string,
    ref: string = 'main',
    format: string = 'zip'
  ): Promise<ExportResponse> {
    const response = await this.request<ApiResponse<ExportResponse>>({
      method: 'POST',
      url: `/v1/repos/${repoName}/export`,
      params: { ref, format },
    })
    return response.data
  }

  async getExportStatus(exportId: string): Promise<ExportResponse> {
    const response = await this.request<ApiResponse<ExportResponse>>({
      method: 'GET',
      url: `/v1/exports/${exportId}`,
    })
    return response.data
  }

  // Health check

  async healthCheck(): Promise<HealthResponse> {
    const response = await this.request<HealthResponse>({
      method: 'GET',
      url: '/health',
    })
    return response
  }

  // Admin operations (require admin permissions)

  async listTenants(): Promise<Tenant[]> {
    const response = await this.request<ApiResponse<Tenant[]>>({
      method: 'GET',
      url: '/v1/admin/tenants',
    })
    return response.data
  }

  async createTenant(name: string): Promise<Tenant> {
    const response = await this.request<ApiResponse<Tenant>>({
      method: 'POST',
      url: '/v1/admin/tenants',
      data: { name },
    })
    return response.data
  }

  async listPolicies(tenantId: string): Promise<Policy[]> {
    const response = await this.request<ApiResponse<Policy[]>>({
      method: 'GET',
      url: `/v1/admin/tenants/${tenantId}/policies`,
    })
    return response.data
  }

  async createPolicy(tenantId: string, policy: Partial<Policy>): Promise<Policy> {
    const response = await this.request<ApiResponse<Policy>>({
      method: 'POST',
      url: `/v1/admin/tenants/${tenantId}/policies`,
      data: policy,
    })
    return response.data
  }

  async listSubjectAttributes(subject?: string): Promise<SubjectAttribute[]> {
    const params = subject ? { subject } : {}
    const response = await this.request<ApiResponse<SubjectAttribute[]>>({
      method: 'GET',
      url: '/v1/admin/attributes',
      params,
    })
    return response.data
  }

  async createSubjectAttribute(
    subject: string,
    key: string,
    value: string
  ): Promise<SubjectAttribute> {
    const response = await this.request<ApiResponse<SubjectAttribute>>({
      method: 'POST',
      url: '/v1/admin/attributes',
      data: { subject, key, value },
    })
    return response.data
  }
}
