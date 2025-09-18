import axios, { AxiosInstance, AxiosRequestConfig } from 'axios'
import { useAuthStore } from '@/stores/auth'

const API_BASE_URL = import.meta.env.VITE_API_BASE_URL || 'http://localhost:8080'

export const api: AxiosInstance = axios.create({
  baseURL: API_BASE_URL,
  timeout: 30000,
  headers: {
    'Content-Type': 'application/json',
  },
})

// Request interceptor to add auth token and request ID
api.interceptors.request.use(
  (config) => {
    const authStore = useAuthStore.getState()
    if (authStore.user?.access_token) {
      config.headers.Authorization = `Bearer ${authStore.user.access_token}`
    }
    
    // Add request ID for tracing
    config.headers['X-Request-ID'] = generateRequestId()
    
    return config
  },
  (error) => {
    return Promise.reject(error)
  }
)

// Response interceptor to handle auth errors
api.interceptors.response.use(
  (response) => response,
  (error) => {
    if (error.response?.status === 401) {
      const authStore = useAuthStore.getState()
      authStore.signoutRedirect()
    }
    return Promise.reject(error)
  }
)

// Helper functions
function generateRequestId(): string {
  return Date.now().toString(36) + Math.random().toString(36).substr(2)
}

// SSE helper for job status updates
export function subscribeJobs(repo: string): EventSource {
  const authStore = useAuthStore.getState()
  const url = new URL(`/v1/jobs/stream`, API_BASE_URL)
  url.searchParams.set('repo', repo)
  
  const eventSource = new EventSource(url.toString())
  
  // Add auth header for SSE (if supported by browser/server)
  // Note: SSE doesn't support custom headers directly, so we may need to pass auth via URL params
  // or implement WebSocket for authenticated streaming
  
  return eventSource
}

// Error mapping helper
export function mapApiError(error: any): string {
  if (error.response?.data?.message) {
    return error.response.data.message
  }
  if (error.message) {
    return error.message
  }
  return 'An unexpected error occurred'
}