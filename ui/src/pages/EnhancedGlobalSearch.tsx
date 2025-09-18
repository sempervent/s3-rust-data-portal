import React, { useState, useEffect } from 'react'
import { useApi } from '@/hooks/useApi'
import { Button } from '@/components/ui/button'
import { Card } from '@/components/ui/card'
import { Tabs } from '@/components/ui/Tabs'
import { Modal } from '@/components/ui/Modal'
import { useAppStore } from '@/app/store'
import { SavedView, getSavedViews, saveView, deleteView } from '@/lib/savedViews'
import EnhancedSearchBar from '@/components/search/EnhancedSearchBar'
import SearchFacets from '@/components/search/SearchFacets'
import EnhancedSearchResults from '@/components/search/EnhancedSearchResults'
import { 
  Save, 
  Bookmark, 
  Settings, 
  Download,
  Share2,
  Filter,
  Grid,
  List
} from 'lucide-react'

interface SearchResult {
  id: string
  repo_id: string
  repo_name: string
  path: string
  file_name: string
  file_type: string
  file_size: number
  creation_dt: string
  creator?: string
  org_lab?: string
  tags?: string[]
  description?: string
  object_sha256: string
  meta?: Record<string, any>
}

interface SearchResponse {
  docs: SearchResult[]
  num_found: number
  facets?: {
    [key: string]: {
      buckets: Array<{ val: string; count: number }>
    }
  }
  suggestions?: string[]
}

interface SearchFilters {
  query: string
  file_type: string[]
  org_lab: string[]
  tags: string[]
  size_min: string
  size_max: string
  date_from: string
  date_to: string
}

interface FacetBucket {
  val: string
  count: number
}

interface Facet {
  field: string
  buckets: FacetBucket[]
}

const EnhancedGlobalSearch: React.FC = () => {
  const [filters, setFilters] = useState<SearchFilters>({
    query: '',
    file_type: [],
    org_lab: [],
    tags: [],
    size_min: '',
    size_max: '',
    date_from: '',
    date_to: ''
  })
  const [selectedFacets, setSelectedFacets] = useState<Record<string, string[]>>({})
  const [currentPage, setCurrentPage] = useState(1)
  const [viewMode, setViewMode] = useState<'list' | 'grid'>('list')
  const [savedViews, setSavedViews] = useState<SavedView[]>([])
  const [showSaveModal, setShowSaveModal] = useState(false)
  const [showViewsModal, setShowViewsModal] = useState(false)
  const [newViewName, setNewViewName] = useState('')
  const [hasSearched, setHasSearched] = useState(false)
  
  const { addToast } = useAppStore()

  // Build search parameters
  const buildSearchParams = () => {
    const params = new URLSearchParams()
    
    if (filters.query) params.set('q', filters.query)
    if (filters.file_type.length > 0) params.set('fq', `file_type:(${filters.file_type.join(' OR ')})`)
    if (filters.org_lab.length > 0) params.set('fq', `org_lab:(${filters.org_lab.join(' OR ')})`)
    if (filters.tags.length > 0) params.set('fq', `tags:(${filters.tags.join(' OR ')})`)
    if (filters.size_min) params.set('fq', `file_size:[${parseInt(filters.size_min) * 1024 * 1024} TO *]`)
    if (filters.size_max) params.set('fq', `file_size:[* TO ${parseInt(filters.size_max) * 1024 * 1024}]`)
    if (filters.date_from) params.set('fq', `creation_dt:[${filters.date_from}T00:00:00Z TO *]`)
    if (filters.date_to) params.set('fq', `creation_dt:[* TO ${filters.date_to}T23:59:59Z]`)
    
    // Add selected facets
    Object.entries(selectedFacets).forEach(([field, values]) => {
      if (values.length > 0) {
        params.append('fq', `${field}:(${values.join(' OR ')})`)
      }
    })
    
    params.set('rows', '20')
    params.set('start', ((currentPage - 1) * 20).toString())
    
    // Add JSON facets for advanced faceting
    const jsonFacet = {
      file_type: { type: 'terms', field: 'file_type', limit: 20 },
      org_lab: { type: 'terms', field: 'org_lab', limit: 20 },
      tags: { type: 'terms', field: 'tags', limit: 20 },
      creation_dt: { 
        type: 'range', 
        field: 'creation_dt',
        ranges: [
          { from: '2024-01-01T00:00:00Z', to: '2024-12-31T23:59:59Z', label: '2024' },
          { from: '2023-01-01T00:00:00Z', to: '2023-12-31T23:59:59Z', label: '2023' },
          { from: '2022-01-01T00:00:00Z', to: '2022-12-31T23:59:59Z', label: '2022' }
        ]
      }
    }
    params.set('json.facet', JSON.stringify(jsonFacet))
    
    return params
  }

  const searchParams = buildSearchParams()
  const { data: searchData, isLoading } = useApi<SearchResponse>(
    `/v1/search?${searchParams.toString()}`,
    { enabled: hasSearched && filters.query.length > 0 }
  )

  // Load saved views
  useEffect(() => {
    setSavedViews(getSavedViews())
  }, [])

  // Handle search
  const handleSearch = (query: string, searchFilters: SearchFilters) => {
    setFilters({ ...searchFilters, query })
    setCurrentPage(1)
    setHasSearched(true)
  }

  // Handle filter changes
  const handleFiltersChange = (newFilters: SearchFilters) => {
    setFilters(newFilters)
    setCurrentPage(1)
    if (hasSearched) {
      setHasSearched(true)
    }
  }

  // Handle facet selection
  const handleFacetSelect = (field: string, value: string) => {
    setSelectedFacets(prev => ({
      ...prev,
      [field]: [...(prev[field] || []), value]
    }))
    setCurrentPage(1)
  }

  // Handle facet removal
  const handleFacetRemove = (field: string, value: string) => {
    setSelectedFacets(prev => ({
      ...prev,
      [field]: (prev[field] || []).filter(v => v !== value)
    }))
    setCurrentPage(1)
  }

  // Clear all facets
  const clearAllFacets = () => {
    setSelectedFacets({})
    setCurrentPage(1)
  }

  // Convert facets to the format expected by SearchFacets component
  const convertFacets = (): Facet[] => {
    if (!searchData?.facets) return []
    
    return Object.entries(searchData.facets).map(([field, facetData]) => ({
      field,
      buckets: facetData.buckets || []
    }))
  }

  // Save current search as a view
  const saveCurrentView = () => {
    if (!newViewName.trim()) return
    
    const view: SavedView = {
      id: Date.now().toString(),
      name: newViewName,
      query: filters.query,
      filters: {
        file_type: filters.file_type,
        org_lab: filters.org_lab,
        tags: filters.tags,
        size_min: filters.size_min,
        size_max: filters.size_max,
        date_from: filters.date_from,
        date_to: filters.date_to
      },
      selectedFacets,
      createdAt: new Date().toISOString()
    }
    
    saveView(view)
    setSavedViews(getSavedViews())
    setNewViewName('')
    setShowSaveModal(false)
    addToast('Search view saved successfully', 'success')
  }

  // Load a saved view
  const loadSavedView = (view: SavedView) => {
    setFilters({
      query: view.query,
      file_type: view.filters.file_type,
      org_lab: view.filters.org_lab,
      tags: view.filters.tags,
      size_min: view.filters.size_min,
      size_max: view.filters.size_max,
      date_from: view.filters.date_from,
      date_to: view.filters.date_to
    })
    setSelectedFacets(view.selectedFacets)
    setCurrentPage(1)
    setHasSearched(true)
    setShowViewsModal(false)
  }

  // Delete a saved view
  const deleteSavedView = (viewId: string) => {
    deleteView(viewId)
    setSavedViews(getSavedViews())
    addToast('Search view deleted', 'success')
  }

  return (
    <div className="min-h-screen bg-gray-50">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
        {/* Header */}
        <div className="mb-8">
          <h1 className="text-3xl font-bold text-gray-900 mb-2">Global Search</h1>
          <p className="text-gray-600">
            Search across all repositories, files, and metadata with advanced filtering and faceting.
          </p>
        </div>

        {/* Search Bar */}
        <Card className="p-6 mb-6">
          <EnhancedSearchBar
            onSearch={handleSearch}
            onFiltersChange={handleFiltersChange}
            initialQuery={filters.query}
            initialFilters={filters}
            showAdvanced={true}
          />
        </Card>

        {/* Results and Facets */}
        {hasSearched && (
          <div className="grid grid-cols-1 lg:grid-cols-4 gap-6">
            {/* Facets Sidebar */}
            <div className="lg:col-span-1">
              <div className="sticky top-4">
                <SearchFacets
                  facets={convertFacets()}
                  selectedFacets={selectedFacets}
                  onFacetSelect={handleFacetSelect}
                  onFacetRemove={handleFacetRemove}
                  onClearAll={clearAllFacets}
                />
              </div>
            </div>

            {/* Results */}
            <div className="lg:col-span-3">
              <Card className="p-6">
                {/* Results Header */}
                <div className="flex items-center justify-between mb-6">
                  <div className="flex items-center gap-4">
                    <h2 className="text-lg font-semibold text-gray-900">Search Results</h2>
                    {searchData && (
                      <span className="text-sm text-gray-500">
                        {searchData.num_found.toLocaleString()} results
                      </span>
                    )}
                  </div>
                  
                  <div className="flex items-center gap-2">
                    {/* View Mode Toggle */}
                    <div className="flex items-center border rounded-lg">
                      <button
                        onClick={() => setViewMode('list')}
                        className={`p-2 ${viewMode === 'list' ? 'bg-blue-50 text-blue-600' : 'text-gray-500'}`}
                      >
                        <List className="h-4 w-4" />
                      </button>
                      <button
                        onClick={() => setViewMode('grid')}
                        className={`p-2 ${viewMode === 'grid' ? 'bg-blue-50 text-blue-600' : 'text-gray-500'}`}
                      >
                        <Grid className="h-4 w-4" />
                      </button>
                    </div>

                    {/* Actions */}
                    <Button
                      variant="outline"
                      size="sm"
                      onClick={() => setShowSaveModal(true)}
                    >
                      <Save className="h-4 w-4 mr-2" />
                      Save View
                    </Button>
                    
                    <Button
                      variant="outline"
                      size="sm"
                      onClick={() => setShowViewsModal(true)}
                    >
                      <Bookmark className="h-4 w-4 mr-2" />
                      Saved Views
                    </Button>
                  </div>
                </div>

                {/* Search Results */}
                <EnhancedSearchResults
                  results={searchData?.docs || []}
                  total={searchData?.num_found || 0}
                  isLoading={isLoading}
                />
              </Card>
            </div>
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
              <label className="block text-sm font-medium text-gray-700 mb-1">
                View Name
              </label>
              <input
                type="text"
                value={newViewName}
                onChange={(e) => setNewViewName(e.target.value)}
                placeholder="Enter a name for this search view"
                className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
              />
            </div>
            <div className="flex justify-end gap-2">
              <Button
                variant="outline"
                onClick={() => setShowSaveModal(false)}
              >
                Cancel
              </Button>
              <Button
                onClick={saveCurrentView}
                disabled={!newViewName.trim()}
              >
                Save View
              </Button>
            </div>
          </div>
        </Modal>

        {/* Saved Views Modal */}
        <Modal
          isOpen={showViewsModal}
          onClose={() => setShowViewsModal(false)}
          title="Saved Search Views"
        >
          <div className="space-y-4">
            {savedViews.length === 0 ? (
              <div className="text-center py-8 text-gray-500">
                No saved views yet. Save your search to create a view.
              </div>
            ) : (
              <div className="space-y-2">
                {savedViews.map((view) => (
                  <div
                    key={view.id}
                    className="flex items-center justify-between p-3 border rounded-lg"
                  >
                    <div>
                      <div className="font-medium text-gray-900">{view.name}</div>
                      <div className="text-sm text-gray-500">
                        Query: {view.query || 'No query'}
                      </div>
                    </div>
                    <div className="flex items-center gap-2">
                      <Button
                        variant="outline"
                        size="sm"
                        onClick={() => loadSavedView(view)}
                      >
                        Load
                      </Button>
                      <Button
                        variant="outline"
                        size="sm"
                        onClick={() => deleteSavedView(view.id)}
                        className="text-red-600 hover:text-red-700"
                      >
                        Delete
                      </Button>
                    </div>
                  </div>
                ))}
              </div>
            )}
            <div className="flex justify-end">
              <Button
                variant="outline"
                onClick={() => setShowViewsModal(false)}
              >
                Close
              </Button>
            </div>
          </div>
        </Modal>
      </div>
    </div>
  )
}

export default EnhancedGlobalSearch
