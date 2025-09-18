// Mobile-optimized search dashboard component
// Week 8: Mobile/responsive UX with PWA support

import React, { useState, useCallback } from 'react'
import { 
  Search, 
  TrendingUp, 
  Clock, 
  Star, 
  File,
  Folder,
  Tag,
  Filter,
  SortAsc,
  BarChart3,
  Settings,
  HelpCircle,
  Plus,
  ChevronRight
} from 'lucide-react'

interface SearchDashboardItem {
  id: string
  title: string
  description: string
  icon: React.ComponentType<{ className?: string }>
  count?: number
  trend?: 'up' | 'down' | 'stable'
  onClick: () => void
}

interface MobileSearchDashboardProps {
  onSearch: () => void
  onFilters: () => void
  onSort: () => void
  onAnalytics: () => void
  onSettings: () => void
  onHelp: () => void
  className?: string
}

export const MobileSearchDashboard: React.FC<MobileSearchDashboardProps> = ({
  onSearch,
  onFilters,
  onSort,
  onAnalytics,
  onSettings,
  onHelp,
  className = ''
}) => {
  const [recentSearches] = useState([
    { query: 'customer data', count: 45, timestamp: '2 hours ago' },
    { query: 'sales report', count: 23, timestamp: '1 day ago' },
    { query: 'Q4 analytics', count: 12, timestamp: '3 days ago' }
  ])

  const [popularSearches] = useState([
    { query: 'financial data', count: 156, trend: 'up' as const },
    { query: 'customer analytics', count: 134, trend: 'up' as const },
    { query: 'marketing campaigns', count: 98, trend: 'stable' as const },
    { query: 'product metrics', count: 87, trend: 'down' as const }
  ])

  const [quickActions] = useState<SearchDashboardItem[]>([
    {
      id: 'search',
      title: 'Search',
      description: 'Find files, folders, and data',
      icon: Search,
      onClick: onSearch
    },
    {
      id: 'filters',
      title: 'Filters',
      description: 'Narrow down results',
      icon: Filter,
      onClick: onFilters
    },
    {
      id: 'sort',
      title: 'Sort',
      description: 'Organize results',
      icon: SortAsc,
      onClick: onSort
    },
    {
      id: 'analytics',
      title: 'Analytics',
      description: 'Search insights',
      icon: BarChart3,
      onClick: onAnalytics
    }
  ])

  const [recentFiles] = useState([
    { name: 'customer_data.csv', path: '/datasets/', size: '2.4 MB', modified: '2 hours ago' },
    { name: 'sales_report.pdf', path: '/reports/', size: '1.8 MB', modified: '1 day ago' },
    { name: 'analytics.json', path: '/data/', size: '945 KB', modified: '3 days ago' }
  ])

  const [recentFolders] = useState([
    { name: 'Q4 Reports', path: '/reports/q4/', count: 12, modified: '1 day ago' },
    { name: 'Customer Data', path: '/datasets/customers/', count: 8, modified: '2 days ago' },
    { name: 'Marketing', path: '/campaigns/marketing/', count: 15, modified: '1 week ago' }
  ])

  // Get trend icon
  const getTrendIcon = (trend: string) => {
    switch (trend) {
      case 'up':
        return <TrendingUp className="w-4 h-4 text-green-500" />
      case 'down':
        return <TrendingUp className="w-4 h-4 text-red-500 rotate-180" />
      default:
        return <TrendingUp className="w-4 h-4 text-gray-400" />
    }
  }

  // Format file size
  const formatFileSize = (size: string) => {
    return size
  }

  return (
    <div className={`bg-gray-50 min-h-screen ${className}`}>
      {/* Header */}
      <div className="bg-white border-b border-gray-200 px-4 py-3">
        <div className="flex items-center justify-between">
          <div>
            <h1 className="text-lg font-semibold text-gray-900">
              Search Dashboard
            </h1>
            <p className="text-sm text-gray-600">
              Find and explore your data
            </p>
          </div>
          
          <div className="flex items-center space-x-2">
            <button
              onClick={onHelp}
              className="p-2 text-gray-600 hover:text-gray-900 hover:bg-gray-100 rounded-md transition-colors"
            >
              <HelpCircle className="w-5 h-5" />
            </button>
            <button
              onClick={onSettings}
              className="p-2 text-gray-600 hover:text-gray-900 hover:bg-gray-100 rounded-md transition-colors"
            >
              <Settings className="w-5 h-5" />
            </button>
          </div>
        </div>
      </div>

      {/* Quick Actions */}
      <div className="p-4">
        <h2 className="text-sm font-medium text-gray-900 mb-3">Quick Actions</h2>
        <div className="grid grid-cols-2 gap-3">
          {quickActions.map((action) => {
            const Icon = action.icon
            return (
              <button
                key={action.id}
                onClick={action.onClick}
                className="p-4 bg-white rounded-lg border border-gray-200 hover:border-blue-300 hover:shadow-md transition-all text-left"
              >
                <div className="flex items-center space-x-3">
                  <div className="w-10 h-10 bg-blue-100 rounded-lg flex items-center justify-center">
                    <Icon className="w-5 h-5 text-blue-600" />
                  </div>
                  <div className="flex-1 min-w-0">
                    <h3 className="text-sm font-medium text-gray-900">
                      {action.title}
                    </h3>
                    <p className="text-xs text-gray-600">
                      {action.description}
                    </p>
                  </div>
                  <ChevronRight className="w-4 h-4 text-gray-400" />
                </div>
              </button>
            )
          })}
        </div>
      </div>

      {/* Recent Searches */}
      <div className="px-4 pb-4">
        <div className="flex items-center justify-between mb-3">
          <h2 className="text-sm font-medium text-gray-900">Recent Searches</h2>
          <button className="text-xs text-blue-600 hover:text-blue-700">
            View All
          </button>
        </div>
        <div className="space-y-2">
          {recentSearches.map((search, index) => (
            <button
              key={index}
              onClick={onSearch}
              className="w-full p-3 bg-white rounded-lg border border-gray-200 hover:border-blue-300 hover:shadow-sm transition-all text-left"
            >
              <div className="flex items-center justify-between">
                <div className="flex items-center space-x-3">
                  <Clock className="w-4 h-4 text-gray-400" />
                  <div>
                    <p className="text-sm font-medium text-gray-900">
                      {search.query}
                    </p>
                    <p className="text-xs text-gray-500">
                      {search.timestamp}
                    </p>
                  </div>
                </div>
                <div className="text-right">
                  <p className="text-sm font-medium text-gray-900">
                    {search.count}
                  </p>
                  <p className="text-xs text-gray-500">results</p>
                </div>
              </div>
            </button>
          ))}
        </div>
      </div>

      {/* Popular Searches */}
      <div className="px-4 pb-4">
        <div className="flex items-center justify-between mb-3">
          <h2 className="text-sm font-medium text-gray-900">Popular Searches</h2>
          <button className="text-xs text-blue-600 hover:text-blue-700">
            View All
          </button>
        </div>
        <div className="space-y-2">
          {popularSearches.map((search, index) => (
            <button
              key={index}
              onClick={onSearch}
              className="w-full p-3 bg-white rounded-lg border border-gray-200 hover:border-blue-300 hover:shadow-sm transition-all text-left"
            >
              <div className="flex items-center justify-between">
                <div className="flex items-center space-x-3">
                  <TrendingUp className="w-4 h-4 text-gray-400" />
                  <div>
                    <p className="text-sm font-medium text-gray-900">
                      {search.query}
                    </p>
                    <p className="text-xs text-gray-500">
                      {search.count} searches
                    </p>
                  </div>
                </div>
                {getTrendIcon(search.trend)}
              </div>
            </button>
          ))}
        </div>
      </div>

      {/* Recent Files */}
      <div className="px-4 pb-4">
        <div className="flex items-center justify-between mb-3">
          <h2 className="text-sm font-medium text-gray-900">Recent Files</h2>
          <button className="text-xs text-blue-600 hover:text-blue-700">
            View All
          </button>
        </div>
        <div className="space-y-2">
          {recentFiles.map((file, index) => (
            <button
              key={index}
              className="w-full p-3 bg-white rounded-lg border border-gray-200 hover:border-blue-300 hover:shadow-sm transition-all text-left"
            >
              <div className="flex items-center justify-between">
                <div className="flex items-center space-x-3">
                  <File className="w-4 h-4 text-blue-500" />
                  <div className="flex-1 min-w-0">
                    <p className="text-sm font-medium text-gray-900 truncate">
                      {file.name}
                    </p>
                    <p className="text-xs text-gray-500">
                      {file.path} • {file.size} • {file.modified}
                    </p>
                  </div>
                </div>
                <ChevronRight className="w-4 h-4 text-gray-400" />
              </div>
            </button>
          ))}
        </div>
      </div>

      {/* Recent Folders */}
      <div className="px-4 pb-4">
        <div className="flex items-center justify-between mb-3">
          <h2 className="text-sm font-medium text-gray-900">Recent Folders</h2>
          <button className="text-xs text-blue-600 hover:text-blue-700">
            View All
          </button>
        </div>
        <div className="space-y-2">
          {recentFolders.map((folder, index) => (
            <button
              key={index}
              className="w-full p-3 bg-white rounded-lg border border-gray-200 hover:border-blue-300 hover:shadow-sm transition-all text-left"
            >
              <div className="flex items-center justify-between">
                <div className="flex items-center space-x-3">
                  <Folder className="w-4 h-4 text-green-500" />
                  <div className="flex-1 min-w-0">
                    <p className="text-sm font-medium text-gray-900 truncate">
                      {folder.name}
                    </p>
                    <p className="text-xs text-gray-500">
                      {folder.path} • {folder.count} items • {folder.modified}
                    </p>
                  </div>
                </div>
                <ChevronRight className="w-4 h-4 text-gray-400" />
              </div>
            </button>
          ))}
        </div>
      </div>

      {/* Search Tips */}
      <div className="px-4 pb-4">
        <div className="bg-blue-50 border border-blue-200 rounded-lg p-4">
          <div className="flex items-start space-x-3">
            <div className="w-8 h-8 bg-blue-100 rounded-full flex items-center justify-center flex-shrink-0">
              <Search className="w-4 h-4 text-blue-600" />
            </div>
            <div>
              <h3 className="text-sm font-medium text-blue-900">
                Search Tip
              </h3>
              <p className="text-sm text-blue-700 mt-1">
                Try using semantic search for better results. Ask questions like "Show me customer data from last month" instead of just "customer data".
              </p>
            </div>
          </div>
        </div>
      </div>
    </div>
  )
}

export default MobileSearchDashboard
