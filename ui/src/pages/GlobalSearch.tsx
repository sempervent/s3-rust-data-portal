import React, { useState, useEffect } from 'react'
import { Link } from 'react-router-dom'
import { useApi } from '@/hooks/useApi'
import { Button } from '@/components/ui/button'
import { DataTable } from '@/components/ui/DataTable'
import { Modal } from '@/components/ui/Modal'
import { useAppStore } from '@/app/store'
import { SavedView, getSavedViews, saveView, deleteView, exportViews, importViews } from '@/lib/savedViews'

interface SearchResult {
  id: string
  repo: string
  path: string
  size: number
  sha256: string
  created_at: string
  file_type: string
  meta?: {
    creator?: string
    org_lab?: string
    tags?: string[]
    description?: string
  }
}

interface SearchResponse {
  results: SearchResult[]
  total: number
  page: number
  per_page: number
}

interface SearchFilters {
  query: string
  file_type: string
  org_lab: string
  tags: string
  creator: string
  size_min: string
  size_max: string
  date_from: string
  date_to: string
}

const DEFAULT_FILTERS: SearchFilters = {
  query: '',
  file_type: '',
  org_lab: '',
  tags: '',
  creator: '',
  size_min: '',
  size_max: '',
  date_from: '',
  date_to: ''
}

const GlobalSearch: React.FC = () => {
  const [filters, setFilters] = useState<SearchFilters>(DEFAULT_FILTERS)
  const [currentPage, setCurrentPage] = useState(1)
  const [sortField, setSortField] = useState<string | null>(null)
  const [sortDirection, setSortDirection] = useState<'asc' | 'desc' | null>(null)
  const [selectedRows, setSelectedRows] = useState<string[]>([])
  const [savedViews, setSavedViews] = useState<SavedView[]>([])
  const [showSaveModal, setShowSaveModal] = useState(false)
  const [showViewsModal, setShowViewsModal] = useState(false)
  const [newViewName, setNewViewName] = useState('')
  const { addToast } = useAppStore()

  const searchParams = new URLSearchParams({
    q: filters.query,
    page: currentPage.toString(),
    per_page: '20',
    ...(sortField && { sort: sortField }),
    ...(sortDirection && { order: sortDirection }),
    ...Object.fromEntries(
      Object.entries(filters).filter(([key, value]) => key !== 'query' && value !== '')
    )
  })

  const { data: searchData, isLoading } = useApi<SearchResponse>(
    `/v1/search?${searchParams.toString()}`,
    { enabled: filters.query.length > 0 }
  )

  useEffect(() => {
    setSavedViews(getSavedViews())
  }, [])

  const formatFileSize = (bytes: number): string => {
    if (bytes === 0) return '0 B'
    const k = 1024
    const sizes = ['B', 'KB', 'MB', 'GB']
    const i = Math.floor(Math.log(bytes) / Math.log(k))
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i]
  }

  const getFileIcon = (path: string): string => {
    const ext = path.split('.').pop()?.toLowerCase()
    switch (ext) {
      case 'csv': return '📊'
      case 'json': return '📄'
      case 'parquet': return '🗃️'
      case 'onnx':
      case 'pt':
      case 'pth': return '🤖'
      case 'png':
      case 'jpg':
      case 'jpeg': return '🖼️'
      default: return '📁'
    }
  }

  const handleSearch = (e: React.FormEvent) => {
    e.preventDefault()
    setCurrentPage(1)
  }

  const handleFilterChange = (key: keyof SearchFilters, value: string) => {
    setFilters(prev => ({ ...prev, [key]: value }))
    setCurrentPage(1)
  }

  const handleSort = (field: string) => {
    if (sortField === field) {
      setSortDirection(prev => prev === 'asc' ? 'desc' : prev === 'desc' ? null : 'asc')
      if (sortDirection === 'desc') setSortField(null)
    } else {
      setSortField(field)
      setSortDirection('asc')
    }
    setCurrentPage(1)
  }

  const handleSaveView = () => {
    if (!newViewName.trim()) {
      addToast('Please enter a view name', 'error')
      return
    }

    try {
      const newView = saveView({
        name: newViewName,
        filters,
        columns: ['repo', 'path', 'size', 'created_at', 'org_lab', 'tags'],
        sorting: sortField ? { field: sortField, direction: sortDirection || 'asc' } : undefined
      })

      setSavedViews(getSavedViews())
      setShowSaveModal(false)
      setNewViewName('')
      addToast(`View "${newView.name}" saved successfully`, 'success')
    } catch (error) {
      addToast('Failed to save view', 'error')
    }
  }

  const handleLoadView = (view: SavedView) => {
    setFilters(view.filters as SearchFilters)
    if (view.sorting) {
      setSortField(view.sorting.field)
      setSortDirection(view.sorting.direction)
    }
    setCurrentPage(1)
    setShowViewsModal(false)
    addToast(`Loaded view "${view.name}"`, 'success')
  }

  const handleDeleteView = (viewId: string) => {
    if (deleteView(viewId)) {
      setSavedViews(getSavedViews())
      addToast('View deleted successfully', 'success')
    } else {
      addToast('Failed to delete view', 'error')
    }
  }

  const handleExportResults = () => {
    if (!searchData?.results) return

    const csv = [
      ['Repository', 'Path', 'Size', 'Type', 'Creator', 'Organization', 'Tags', 'Created'],
      ...searchData.results.map(result => [
        result.repo,
        result.path,
        formatFileSize(result.size),
        result.file_type,
        result.meta?.creator || '',
        result.meta?.org_lab || '',
        result.meta?.tags?.join(', ') || '',
        new Date(result.created_at).toISOString()
      ])
    ].map(row => row.join(',')).join('\n')

    const blob = new Blob([csv], { type: 'text/csv' })
    const url = URL.createObjectURL(blob)
    const a = document.createElement('a')
    a.href = url
    a.download = `search-results-${new Date().toISOString().split('T')[0]}.csv`
    document.body.appendChild(a)
    a.click()
    document.body.removeChild(a)
    URL.revokeObjectURL(url)
  }

  const columns = [
    {
      key: 'repo',
      header: 'Repository',
      sortable: true,
      render: (value: string) => (
        <Link to={`/repos/${value}`} className="font-medium text-primary hover:underline">
          {value}
        </Link>
      )
    },
    {
      key: 'path',
      header: 'Path',
      sortable: true,
      render: (value: string, row: SearchResult) => (
        <Link 
          to={`/repos/${row.repo}/entry/main/${row.path}`}
          className="flex items-center space-x-2 hover:text-primary"
        >
          <span>{getFileIcon(value)}</span>
          <span>{value}</span>
        </Link>
      )
    },
    {
      key: 'size',
      header: 'Size',
      sortable: true,
      render: (value: number) => <span className="font-mono text-sm">{formatFileSize(value)}</span>
    },
    {
      key: 'file_type',
      header: 'Type',
      sortable: true,
      render: (value: string) => (
        <span className="text-xs bg-muted px-2 py-1 rounded">{value}</span>
      )
    },
    {
      key: 'org_lab',
      header: 'Organization',
      sortable: true,
      render: (value: string, row: SearchResult) => row.meta?.org_lab || '—'
    },
    {
      key: 'tags',
      header: 'Tags',
      render: (value: string, row: SearchResult) => (
        <div className="flex flex-wrap gap-1">
          {row.meta?.tags?.slice(0, 3).map((tag, index) => (
            <span key={index} className="text-xs bg-primary/10 text-primary px-1 py-0.5 rounded">
              {tag}
            </span>
          ))}
          {row.meta?.tags && row.meta.tags.length > 3 && (
            <span className="text-xs text-muted-foreground">+{row.meta.tags.length - 3}</span>
          )}
        </div>
      )
    },
    {
      key: 'created_at',
      header: 'Created',
      sortable: true,
      render: (value: string) => new Date(value).toLocaleDateString()
    }
  ]

  return (
    <div className="container mx-auto px-4 py-8">
      <div className="max-w-7xl mx-auto">
        <div className="flex items-center justify-between mb-8">
          <h1 className="text-3xl font-bold">Global Search</h1>
          <div className="flex space-x-2">
            <Button variant="outline" onClick={() => setShowViewsModal(true)}>
              📚 Saved Views
            </Button>
            <Button variant="outline" onClick={() => setShowSaveModal(true)}>
              💾 Save View
            </Button>
          </div>
        </div>

        {/* Search Form */}
        <form onSubmit={handleSearch} className="mb-8 space-y-4">
          <div className="flex space-x-2">
            <input
              type="text"
              value={filters.query}
              onChange={(e) => handleFilterChange('query', e.target.value)}
              placeholder="Search across all repositories..."
              className="flex-1 form-input"
            />
            <Button type="submit">🔍 Search</Button>
          </div>

          {/* Advanced Filters */}
          <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
            <input
              type="text"
              value={filters.file_type}
              onChange={(e) => handleFilterChange('file_type', e.target.value)}
              placeholder="File type (e.g., text/csv)"
              className="form-input text-sm"
            />
            <input
              type="text"
              value={filters.org_lab}
              onChange={(e) => handleFilterChange('org_lab', e.target.value)}
              placeholder="Organization/Lab"
              className="form-input text-sm"
            />
            <input
              type="text"
              value={filters.creator}
              onChange={(e) => handleFilterChange('creator', e.target.value)}
              placeholder="Creator"
              className="form-input text-sm"
            />
            <input
              type="text"
              value={filters.tags}
              onChange={(e) => handleFilterChange('tags', e.target.value)}
              placeholder="Tags (comma-separated)"
              className="form-input text-sm"
            />
          </div>

          <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
            <input
              type="number"
              value={filters.size_min}
              onChange={(e) => handleFilterChange('size_min', e.target.value)}
              placeholder="Min size (bytes)"
              className="form-input text-sm"
            />
            <input
              type="number"
              value={filters.size_max}
              onChange={(e) => handleFilterChange('size_max', e.target.value)}
              placeholder="Max size (bytes)"
              className="form-input text-sm"
            />
            <input
              type="date"
              value={filters.date_from}
              onChange={(e) => handleFilterChange('date_from', e.target.value)}
              className="form-input text-sm"
            />
            <input
              type="date"
              value={filters.date_to}
              onChange={(e) => handleFilterChange('date_to', e.target.value)}
              className="form-input text-sm"
            />
          </div>
        </form>

        {/* Results */}
        {filters.query ? (
          <div className="space-y-4">
            {searchData && (
              <div className="flex items-center justify-between">
                <p className="text-sm text-muted-foreground">
                  Found {searchData.total} results
                </p>
                <div className="flex space-x-2">
                  <Button variant="outline" size="sm" onClick={handleExportResults}>
                    📊 Export CSV
                  </Button>
                  {selectedRows.length > 0 && (
                    <Button variant="outline" size="sm">
                      📥 Bulk Download ({selectedRows.length})
                    </Button>
                  )}
                </div>
              </div>
            )}

            <DataTable
              columns={columns}
              data={searchData?.results || []}
              loading={isLoading}
              pagination={searchData ? {
                page: currentPage,
                totalPages: Math.ceil(searchData.total / searchData.per_page),
                onPageChange: setCurrentPage
              } : undefined}
              sorting={{
                field: sortField,
                direction: sortDirection,
                onSort: handleSort
              }}
              selection={{
                selectedRows,
                onSelectionChange: setSelectedRows,
                rowKey: 'id'
              }}
            />
          </div>
        ) : (
          <div className="text-center py-12">
            <div className="text-6xl mb-4">🔍</div>
            <h3 className="text-lg font-semibold mb-2">Search for Data Artifacts</h3>
            <p className="text-muted-foreground mb-4">
              Enter a search query to find files across all repositories
            </p>
            {savedViews.length > 0 && (
              <Button variant="outline" onClick={() => setShowViewsModal(true)}>
                📚 Load Saved View
              </Button>
            )}
          </div>
        )}

        {/* Save View Modal */}
        <Modal
          isOpen={showSaveModal}
          onClose={() => setShowSaveModal(false)}
          title="Save Search View"
        >
          <div className="space-y-4">
            <div>
              <label className="form-label">View Name</label>
              <input
                type="text"
                value={newViewName}
                onChange={(e) => setNewViewName(e.target.value)}
                placeholder="My Search View"
                className="form-input"
              />
            </div>
            <div className="flex justify-end space-x-2">
              <Button variant="outline" onClick={() => setShowSaveModal(false)}>
                Cancel
              </Button>
              <Button onClick={handleSaveView}>
                Save View
              </Button>
            </div>
          </div>
        </Modal>

        {/* Saved Views Modal */}
        <Modal
          isOpen={showViewsModal}
          onClose={() => setShowViewsModal(false)}
          title="Saved Views"
          size="lg"
        >
          <div className="space-y-4">
            {savedViews.length === 0 ? (
              <p className="text-muted-foreground text-center py-8">
                No saved views yet. Save your first search view to get started.
              </p>
            ) : (
              <div className="space-y-3">
                {savedViews.map((view) => (
                  <div key={view.id} className="flex items-center justify-between p-3 border border-border rounded">
                    <div>
                      <h4 className="font-medium">{view.name}</h4>
                      <p className="text-sm text-muted-foreground">
                        Created {new Date(view.created_at).toLocaleDateString()}
                      </p>
                    </div>
                    <div className="flex space-x-2">
                      <Button variant="outline" size="sm" onClick={() => handleLoadView(view)}>
                        Load
                      </Button>
                      <Button 
                        variant="outline" 
                        size="sm" 
                        onClick={() => handleDeleteView(view.id)}
                        className="text-destructive hover:bg-destructive/10"
                      >
                        Delete
                      </Button>
                    </div>
                  </div>
                ))}
              </div>
            )}
          </div>
        </Modal>
      </div>
    </div>
  )
}

export default GlobalSearch
