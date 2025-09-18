// Mobile-optimized search filters component
// Week 8: Mobile/responsive UX with PWA support

import React, { useState, useCallback } from 'react'
import { 
  Filter, 
  X, 
  Calendar, 
  File, 
  Tag, 
  User, 
  Database,
  ChevronDown,
  ChevronUp
} from 'lucide-react'

interface FilterOption {
  value: string
  label: string
  count?: number
}

interface FilterGroup {
  id: string
  label: string
  icon: React.ComponentType<{ className?: string }>
  options: FilterOption[]
  multiSelect?: boolean
  searchable?: boolean
}

interface MobileSearchFiltersProps {
  filters: FilterGroup[]
  selectedFilters: Record<string, string[]>
  onFilterChange: (groupId: string, values: string[]) => void
  onClearAll: () => void
  className?: string
}

export const MobileSearchFilters: React.FC<MobileSearchFiltersProps> = ({
  filters,
  selectedFilters,
  onFilterChange,
  onClearAll,
  className = ''
}) => {
  const [expandedGroups, setExpandedGroups] = useState<Set<string>>(new Set())
  const [searchQueries, setSearchQueries] = useState<Record<string, string>>({})

  // Toggle group expansion
  const toggleGroup = useCallback((groupId: string) => {
    setExpandedGroups(prev => {
      const newSet = new Set(prev)
      if (newSet.has(groupId)) {
        newSet.delete(groupId)
      } else {
        newSet.add(groupId)
      }
      return newSet
    })
  }, [])

  // Handle filter selection
  const handleFilterSelect = useCallback((groupId: string, value: string, multiSelect: boolean = false) => {
    const currentValues = selectedFilters[groupId] || []
    
    if (multiSelect) {
      const newValues = currentValues.includes(value)
        ? currentValues.filter(v => v !== value)
        : [...currentValues, value]
      onFilterChange(groupId, newValues)
    } else {
      onFilterChange(groupId, [value])
    }
  }, [selectedFilters, onFilterChange])

  // Handle search query change
  const handleSearchChange = useCallback((groupId: string, query: string) => {
    setSearchQueries(prev => ({
      ...prev,
      [groupId]: query
    }))
  }, [])

  // Filter options based on search query
  const getFilteredOptions = useCallback((groupId: string, options: FilterOption[]) => {
    const query = searchQueries[groupId]?.toLowerCase() || ''
    if (!query) return options
    
    return options.filter(option =>
      option.label.toLowerCase().includes(query)
    )
  }, [searchQueries])

  // Get total selected count
  const getTotalSelectedCount = useCallback(() => {
    return Object.values(selectedFilters).reduce((total, values) => total + values.length, 0)
  }, [selectedFilters])

  return (
    <div className={`bg-white border-b border-gray-200 ${className}`}>
      {/* Header */}
      <div className="flex items-center justify-between p-4">
        <div className="flex items-center space-x-2">
          <Filter className="w-5 h-5 text-gray-600" />
          <span className="text-sm font-medium text-gray-900">Filters</span>
          {getTotalSelectedCount() > 0 && (
            <span className="inline-flex items-center px-2 py-1 rounded-full text-xs font-medium bg-blue-100 text-blue-800">
              {getTotalSelectedCount()}
            </span>
          )}
        </div>
        
        {getTotalSelectedCount() > 0 && (
          <button
            onClick={onClearAll}
            className="text-sm text-blue-600 hover:text-blue-700 transition-colors"
          >
            Clear All
          </button>
        )}
      </div>

      {/* Filter Groups */}
      <div className="divide-y divide-gray-100">
        {filters.map((group) => {
          const isExpanded = expandedGroups.has(group.id)
          const Icon = group.icon
          const selectedValues = selectedFilters[group.id] || []
          const filteredOptions = getFilteredOptions(group.id, group.options)

          return (
            <div key={group.id} className="p-4">
              {/* Group Header */}
              <button
                onClick={() => toggleGroup(group.id)}
                className="w-full flex items-center justify-between text-left"
              >
                <div className="flex items-center space-x-3">
                  <Icon className="w-5 h-5 text-gray-600" />
                  <span className="text-sm font-medium text-gray-900">
                    {group.label}
                  </span>
                  {selectedValues.length > 0 && (
                    <span className="inline-flex items-center px-2 py-1 rounded-full text-xs font-medium bg-blue-100 text-blue-800">
                      {selectedValues.length}
                    </span>
                  )}
                </div>
                
                {isExpanded ? (
                  <ChevronUp className="w-4 h-4 text-gray-400" />
                ) : (
                  <ChevronDown className="w-4 h-4 text-gray-400" />
                )}
              </button>

              {/* Group Content */}
              {isExpanded && (
                <div className="mt-3 space-y-3">
                  {/* Search Input */}
                  {group.searchable && (
                    <div className="relative">
                      <input
                        type="text"
                        value={searchQueries[group.id] || ''}
                        onChange={(e) => handleSearchChange(group.id, e.target.value)}
                        placeholder={`Search ${group.label.toLowerCase()}...`}
                        className="w-full px-3 py-2 pl-8 border border-gray-200 rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                      />
                      <Search className="absolute left-2 top-1/2 transform -translate-y-1/2 w-4 h-4 text-gray-400" />
                    </div>
                  )}

                  {/* Options */}
                  <div className="space-y-2">
                    {filteredOptions.length === 0 ? (
                      <p className="text-sm text-gray-500 text-center py-2">
                        No options found
                      </p>
                    ) : (
                      filteredOptions.map((option) => {
                        const isSelected = selectedValues.includes(option.value)
                        
                        return (
                          <button
                            key={option.value}
                            onClick={() => handleFilterSelect(group.id, option.value, group.multiSelect)}
                            className={`w-full flex items-center justify-between p-2 rounded-md text-sm transition-colors ${
                              isSelected
                                ? 'bg-blue-50 text-blue-700 border border-blue-200'
                                : 'text-gray-700 hover:bg-gray-50'
                            }`}
                          >
                            <span>{option.label}</span>
                            {option.count !== undefined && (
                              <span className="text-xs text-gray-500">
                                {option.count.toLocaleString()}
                              </span>
                            )}
                          </button>
                        )
                      })
                    )}
                  </div>
                </div>
              )}
            </div>
          )
        })}
      </div>
    </div>
  )
}

export default MobileSearchFilters
