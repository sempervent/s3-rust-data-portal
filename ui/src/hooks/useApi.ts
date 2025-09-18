import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { apiClient } from '../utils/api';
import { 
  Repository, 
  TreeResponse, 
  SearchRequest, 
  SearchResponse, 
  UploadInitRequest, 
  UploadInitResponse,
  CommitRequest,
  CommitResponse
} from '../types';

// Repository hooks
export const useRepositories = () => {
  return useQuery<Repository[]>({
    queryKey: ['repositories'],
    queryFn: () => apiClient.get<Repository[]>('/v1/repos'),
  });
};

export const useCreateRepository = () => {
  const queryClient = useQueryClient();
  
  return useMutation<Repository, Error, { name: string }>({
    mutationFn: ({ name }) => 
      apiClient.post<Repository>('/v1/repos', { name }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['repositories'] });
    },
  });
};

// Tree hooks
export const useTree = (repo: string, ref: string, path?: string) => {
  return useQuery<TreeResponse>({
    queryKey: ['tree', repo, ref, path],
    queryFn: () => {
      const url = path 
        ? `/v1/repos/${repo}/tree/${ref}?path=${encodeURIComponent(path)}`
        : `/v1/repos/${repo}/tree/${ref}`;
      return apiClient.get<TreeResponse>(url);
    },
    enabled: !!repo && !!ref,
  });
};

// Search hooks
export const useSearch = (repo: string, params: SearchRequest) => {
  return useQuery<SearchResponse>({
    queryKey: ['search', repo, params],
    queryFn: () => {
      const searchParams = new URLSearchParams();
      Object.entries(params).forEach(([key, value]) => {
        if (value !== undefined && value !== null) {
          if (Array.isArray(value)) {
            value.forEach(v => searchParams.append(key, v));
          } else {
            searchParams.append(key, String(value));
          }
        }
      });
      return apiClient.get<SearchResponse>(`/v1/repos/${repo}/search?${searchParams}`);
    },
    enabled: !!repo,
  });
};

// Upload hooks
export const useUploadInit = () => {
  return useMutation<UploadInitResponse, Error, { repo: string; request: UploadInitRequest }>({
    mutationFn: ({ repo, request }) =>
      apiClient.post<UploadInitResponse>(`/v1/repos/${repo}/upload-init`, request),
  });
};

// Commit hooks
export const useCommit = () => {
  const queryClient = useQueryClient();
  
  return useMutation<CommitResponse, Error, { repo: string; request: CommitRequest }>({
    mutationFn: ({ repo, request }) =>
      apiClient.post<CommitResponse>(`/v1/repos/${repo}/commit`, request),
    onSuccess: (_, { repo }) => {
      queryClient.invalidateQueries({ queryKey: ['tree', repo] });
    },
  });
};

// RDF hooks
export const useRdf = (repo: string, ref: string, path: string, format: 'turtle' | 'jsonld' = 'turtle') => {
  return useQuery<string>({
    queryKey: ['rdf', repo, ref, path, format],
    queryFn: () => 
      apiClient.get<string>(`/v1/repos/${repo}/rdf/${ref}/${encodeURIComponent(path)}?format=${format}`),
    enabled: !!repo && !!ref && !!path,
  });
};
