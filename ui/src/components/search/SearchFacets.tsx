import React from 'react'
import { Card } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/label'
import { X, ChevronDown, ChevronRight } from 'lucide-react'

interface FacetBucket {
  val: string
  count: number
}

interface Facet {
  field: string
  buckets: FacetBucket[]
}

interface SearchFacetsProps {
  facets: Facet[]
  selectedFacets: Record<string, string[]>
  onFacetSelect: (field: string, value: string) => void
  onFacetRemove: (field: string, value: string) => void
  onClearAll: () => void
}

const SearchFacets: React.FC<SearchFacetsProps> = ({
  facets,
  selectedFacets,
  onFacetSelect,
  onFacetRemove,
  onClearAll
}) => {
  const [expandedFacets, setExpandedFacets] = React.useState<Set<string>>(
    new Set(['file_type', 'org_lab', 'tags'])
  )

  const toggleFacet = (field: string) => {
    const newExpanded = new Set(expandedFacets)
    if (newExpanded.has(field)) {
      newExpanded.delete(field)
    } else {
      newExpanded.add(field)
    }
    setExpandedFacets(newExpanded)
  }

  const getFacetDisplayName = (field: string): string => {
    const displayNames: Record<string, string> = {
      file_type: 'File Type',
      org_lab: 'Organization/Lab',
      tags: 'Tags',
      creation_dt: 'Creation Date',
      creator: 'Creator',
      data_source: 'Data Source'
    }
    return displayNames[field] || field.replace(/_/g, ' ').replace(/\b\w/g, l => l.toUpperCase())
  }

  const formatFacetValue = (field: string, value: string): string => {
    if (field === 'file_type') {
      return value.split('/').pop() || value
    }
    if (field === 'creation_dt') {
      return new Date(value).toLocaleDateString()
    }
    return value
  }

  const getTotalSelectedFacets = (): number => {
    return Object.values(selectedFacets).reduce((total, values) => total + values.length, 0)
  }

  if (facets.length === 0) {
    return null
  }

  return (
    <div className="space-y-4">
      {/* Selected Facets */}
      {getTotalSelectedFacets() > 0 && (
        <Card className="p-4">
          <div className="flex items-center justify-between mb-3">
            <h3 className="text-sm font-medium text-gray-700">Active Filters</h3>
            <Button
              variant="outline"
              size="sm"
              onClick={onClearAll}
              className="text-xs"
            >
              Clear All
            </Button>
          </div>
          <div className="flex flex-wrap gap-2">
            {Object.entries(selectedFacets).map(([field, values]) =>
              values.map((value) => (
                <Badge
                  key={`${field}-${value}`}
                  variant="secondary"
                  className="flex items-center gap-1 px-2 py-1"
                >
                  <span className="text-xs">
                    {getFacetDisplayName(field)}: {formatFacetValue(field, value)}
                  </span>
                  <button
                    onClick={() => onFacetRemove(field, value)}
                    className="ml-1 hover:text-red-600"
                  >
                    <X className="h-3 w-3" />
                  </button>
                </Badge>
              ))
            )}
          </div>
        </Card>
      )}

      {/* Facet Panels */}
      {facets.map((facet) => {
        const isExpanded = expandedFacets.has(facet.field)
        const selectedValues = selectedFacets[facet.field] || []
        const hasSelection = selectedValues.length > 0

        return (
          <Card key={facet.field} className="p-4">
            <button
              onClick={() => toggleFacet(facet.field)}
              className="flex items-center justify-between w-full text-left"
            >
              <h3 className="text-sm font-medium text-gray-700">
                {getFacetDisplayName(facet.field)}
                {hasSelection && (
                  <Badge variant="secondary" className="ml-2 text-xs">
                    {selectedValues.length}
                  </Badge>
                )}
              </h3>
              {isExpanded ? (
                <ChevronDown className="h-4 w-4 text-gray-400" />
              ) : (
                <ChevronRight className="h-4 w-4 text-gray-400" />
              )}
            </button>

            {isExpanded && (
              <div className="mt-3 space-y-2">
                {facet.buckets.slice(0, 10).map((bucket) => {
                  const isSelected = selectedValues.includes(bucket.val)
                  
                  return (
                    <button
                      key={bucket.val}
                      onClick={() => 
                        isSelected 
                          ? onFacetRemove(facet.field, bucket.val)
                          : onFacetSelect(facet.field, bucket.val)
                      }
                      className={`flex items-center justify-between w-full p-2 rounded text-sm transition-colors ${
                        isSelected
                          ? 'bg-blue-50 text-blue-700 border border-blue-200'
                          : 'hover:bg-gray-50 text-gray-700'
                      }`}
                    >
                      <span className="truncate">
                        {formatFacetValue(facet.field, bucket.val)}
                      </span>
                      <span className="text-xs text-gray-500 ml-2">
                        {bucket.count.toLocaleString()}
                      </span>
                    </button>
                  )
                })}
                
                {facet.buckets.length > 10 && (
                  <div className="text-xs text-gray-500 text-center pt-2">
                    Showing top 10 of {facet.buckets.length} values
                  </div>
                )}
              </div>
            )}
          </Card>
        )
      })}

      {/* Date Histogram Facet */}
      {facets.find(f => f.field === 'creation_dt') && (
        <Card className="p-4">
          <h3 className="text-sm font-medium text-gray-700 mb-3">Creation Date</h3>
          <div className="text-sm text-gray-500">
            Date histogram visualization would go here
          </div>
        </Card>
      )}
    </div>
  )
}

export default SearchFacets
