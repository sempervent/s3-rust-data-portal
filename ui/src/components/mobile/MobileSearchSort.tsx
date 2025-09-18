// Mobile-optimized search sort component
// Week 8: Mobile/responsive UX with PWA support

import React, { useState, useCallback } from 'react'
import { 
  SortAsc, 
  SortDesc, 
  Check, 
  ArrowUpDown,
  Calendar,
  FileText,
  User,
  Database
} from 'lucide-react'

interface SortOption {
  value: string
  label: string
  icon: React.ComponentType<{ className?: string }>
  direction: 'asc' | 'desc'
}

interface MobileSearchSortProps {
  options: SortOption[]
  selectedSort: string
  onSortChange: (sort: string) => void
  className?: string
}

export const MobileSearchSort: React.FC<MobileSearchSortProps> = ({
  options,
  selectedSort,
  onSortChange,
  className = ''
}) => {
  const [isOpen, setIsOpen] = useState(false)

  // Handle sort selection
  const handleSortSelect = useCallback((sort: string) => {
    onSortChange(sort)
    setIsOpen(false)
  }, [onSortChange])

  // Get current sort option
  const currentSort = options.find(option => option.value === selectedSort)

  return (
    <div className={`relative ${className}`}>
      {/* Sort Button */}
      <button
        onClick={() => setIsOpen(!isOpen)}
        className="flex items-center space-x-2 px-3 py-2 bg-gray-100 text-gray-700 rounded-md hover:bg-gray-200 transition-colors"
      >
        <ArrowUpDown className="w-4 h-4" />
        <span className="text-sm font-medium">
          {currentSort?.label || 'Sort by'}
        </span>
        {currentSort && (
          <div className="flex items-center space-x-1">
            {currentSort.direction === 'asc' ? (
              <SortAsc className="w-3 h-3" />
            ) : (
              <SortDesc className="w-3 h-3" />
            )}
          </div>
        )}
      </button>

      {/* Sort Options */}
      {isOpen && (
        <div className="absolute top-full left-0 right-0 mt-1 bg-white border border-gray-200 rounded-lg shadow-lg z-50">
          <div className="py-2">
            {options.map((option) => {
              const Icon = option.icon
              const isSelected = option.value === selectedSort
              
              return (
                <button
                  key={option.value}
                  onClick={() => handleSortSelect(option.value)}
                  className={`w-full flex items-center justify-between px-4 py-3 text-left hover:bg-gray-50 transition-colors ${
                    isSelected ? 'bg-blue-50 text-blue-700' : 'text-gray-700'
                  }`}
                >
                  <div className="flex items-center space-x-3">
                    <Icon className="w-4 h-4" />
                    <span className="text-sm font-medium">{option.label}</span>
                  </div>
                  
                  <div className="flex items-center space-x-2">
                    {option.direction === 'asc' ? (
                      <SortAsc className="w-4 h-4" />
                    ) : (
                      <SortDesc className="w-4 h-4" />
                    )}
                    {isSelected && (
                      <Check className="w-4 h-4 text-blue-600" />
                    )}
                  </div>
                </button>
              )
            })}
          </div>
        </div>
      )}

      {/* Backdrop */}
      {isOpen && (
        <div
          className="fixed inset-0 z-40"
          onClick={() => setIsOpen(false)}
        />
      )}
    </div>
  )
}

export default MobileSearchSort
