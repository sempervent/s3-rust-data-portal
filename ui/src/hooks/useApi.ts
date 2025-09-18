import { useQuery, useMutation, UseQueryOptions, UseMutationOptions } from '@tanstack/react-query'
import { api } from '@/lib/api'

interface UseApiOptions<T> {
  method?: 'GET' | 'POST' | 'PUT' | 'DELETE' | 'PATCH'
  enabled?: boolean
  onSuccess?: (data: T) => void
  onError?: (error: any) => void
}

export function useApi<T = any>(
  url: string,
  options?: UseApiOptions<T> & Omit<UseQueryOptions<T>, 'queryKey' | 'queryFn'>
) {
  const { method = 'GET', enabled = true, onSuccess, onError, ...queryOptions } = options || {}

  if (method === 'GET') {
    return useQuery({
      queryKey: [url],
      queryFn: async () => {
        const response = await api.get<T>(url)
        return response.data
      },
      enabled,
      ...queryOptions
    })
  }

  // For non-GET methods, return a mutation
  return useMutation({
    mutationFn: async (data?: any) => {
      const response = await api.request<T>({
        method: method as any,
        url,
        data
      })
      return response.data
    },
    onSuccess,
    onError,
    ...queryOptions
  })
}