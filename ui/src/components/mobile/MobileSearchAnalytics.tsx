// Mobile-optimized search analytics component
// Week 8: Mobile/responsive UX with PWA support

import React, { useState, useCallback } from 'react'
import { 
  TrendingUp, 
  Search, 
  Clock, 
  Users, 
  FileText,
  BarChart3,
  Calendar,
  Filter,
  SortAsc
} from 'lucide-react'

interface SearchAnalytics {
  totalSearches: number
  uniqueUsers: number
  averageResponseTime: number
  topQueries: Array<{
    query: string
    count: number
    trend: 'up' | 'down' | 'stable'
  }>
  searchTrends: Array<{
    date: string
    searches: number
    users: number
  }>
  popularFilters: Array<{
    filter: string
    count: number
    percentage: number
  }>
  searchSources: Array<{
    source: string
    count: number
    percentage: number
  }>
}

interface MobileSearchAnalyticsProps {
  analytics: SearchAnalytics
  className?: string
}

export const MobileSearchAnalytics: React.FC<MobileSearchAnalyticsProps> = ({
  analytics,
  className = ''
}) => {
  const [activeTab, setActiveTab] = useState<'overview' | 'trends' | 'queries' | 'filters'>('overview')
  const [timeRange, setTimeRange] = useState<'7d' | '30d' | '90d'>('7d')

  // Format number
  const formatNumber = (num: number) => {
    if (num >= 1000000) {
      return (num / 1000000).toFixed(1) + 'M'
    } else if (num >= 1000) {
      return (num / 1000).toFixed(1) + 'K'
    }
    return num.toString()
  }

  // Format percentage
  const formatPercentage = (num: number) => {
    return Math.round(num * 100) + '%'
  }

  // Format date
  const formatDate = (date: string) => {
    const d = new Date(date)
    return d.toLocaleDateString('en-US', { month: 'short', day: 'numeric' })
  }

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

  return (
    <div className={`bg-white ${className}`}>
      {/* Header */}
      <div className="p-4 border-b border-gray-200">
        <div className="flex items-center justify-between">
          <div>
            <h2 className="text-lg font-semibold text-gray-900">
              Search Analytics
            </h2>
            <p className="text-sm text-gray-600">
              Insights into search behavior and performance
            </p>
          </div>
          
          <div className="flex items-center space-x-2">
            <select
              value={timeRange}
              onChange={(e) => setTimeRange(e.target.value as any)}
              className="px-3 py-2 border border-gray-200 rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
            >
              <option value="7d">Last 7 days</option>
              <option value="30d">Last 30 days</option>
              <option value="90d">Last 90 days</option>
            </select>
          </div>
        </div>
      </div>

      {/* Tabs */}
      <div className="flex border-b border-gray-200">
        {[
          { id: 'overview', label: 'Overview', icon: BarChart3 },
          { id: 'trends', label: 'Trends', icon: TrendingUp },
          { id: 'queries', label: 'Queries', icon: Search },
          { id: 'filters', label: 'Filters', icon: Filter },
        ].map(({ id, label, icon: Icon }) => (
          <button
            key={id}
            onClick={() => setActiveTab(id as any)}
            className={`flex-1 flex items-center justify-center space-x-2 py-3 text-sm font-medium border-b-2 transition-colors ${
              activeTab === id
                ? 'border-blue-500 text-blue-600'
                : 'border-transparent text-gray-500 hover:text-gray-700'
            }`}
          >
            <Icon className="w-4 h-4" />
            <span>{label}</span>
          </button>
        ))}
      </div>

      {/* Content */}
      <div className="p-4">
        {activeTab === 'overview' && (
          <div className="space-y-6">
            {/* Key Metrics */}
            <div className="grid grid-cols-2 gap-4">
              <div className="bg-blue-50 rounded-lg p-4">
                <div className="flex items-center space-x-2">
                  <Search className="w-5 h-5 text-blue-600" />
                  <span className="text-sm font-medium text-blue-800">Total Searches</span>
                </div>
                <p className="text-2xl font-bold text-blue-900 mt-1">
                  {formatNumber(analytics.totalSearches)}
                </p>
              </div>
              
              <div className="bg-green-50 rounded-lg p-4">
                <div className="flex items-center space-x-2">
                  <Users className="w-5 h-5 text-green-600" />
                  <span className="text-sm font-medium text-green-800">Unique Users</span>
                </div>
                <p className="text-2xl font-bold text-green-900 mt-1">
                  {formatNumber(analytics.uniqueUsers)}
                </p>
              </div>
              
              <div className="bg-yellow-50 rounded-lg p-4">
                <div className="flex items-center space-x-2">
                  <Clock className="w-5 h-5 text-yellow-600" />
                  <span className="text-sm font-medium text-yellow-800">Avg Response</span>
                </div>
                <p className="text-2xl font-bold text-yellow-900 mt-1">
                  {analytics.averageResponseTime}ms
                </p>
              </div>
              
              <div className="bg-purple-50 rounded-lg p-4">
                <div className="flex items-center space-x-2">
                  <FileText className="w-5 h-5 text-purple-600" />
                  <span className="text-sm font-medium text-purple-800">Results Found</span>
                </div>
                <p className="text-2xl font-bold text-purple-900 mt-1">
                  {formatNumber(analytics.totalSearches * 10)}
                </p>
              </div>
            </div>

            {/* Search Sources */}
            <div>
              <h3 className="text-sm font-medium text-gray-900 mb-3">Search Sources</h3>
              <div className="space-y-2">
                {analytics.searchSources.map((source, index) => (
                  <div key={index} className="flex items-center justify-between">
                    <span className="text-sm text-gray-700">{source.source}</span>
                    <div className="flex items-center space-x-2">
                      <div className="w-20 bg-gray-200 rounded-full h-2">
                        <div
                          className="bg-blue-600 h-2 rounded-full"
                          style={{ width: `${source.percentage * 100}%` }}
                        />
                      </div>
                      <span className="text-sm text-gray-500 w-12 text-right">
                        {formatPercentage(source.percentage)}
                      </span>
                    </div>
                  </div>
                ))}
              </div>
            </div>
          </div>
        )}

        {activeTab === 'trends' && (
          <div className="space-y-4">
            <h3 className="text-sm font-medium text-gray-900">Search Trends</h3>
            <div className="space-y-3">
              {analytics.searchTrends.slice(-7).map((trend, index) => (
                <div key={index} className="flex items-center justify-between p-3 bg-gray-50 rounded-lg">
                  <div>
                    <p className="text-sm font-medium text-gray-900">
                      {formatDate(trend.date)}
                    </p>
                    <p className="text-xs text-gray-500">
                      {trend.users} users
                    </p>
                  </div>
                  <div className="text-right">
                    <p className="text-lg font-bold text-gray-900">
                      {trend.searches}
                    </p>
                    <p className="text-xs text-gray-500">searches</p>
                  </div>
                </div>
              ))}
            </div>
          </div>
        )}

        {activeTab === 'queries' && (
          <div className="space-y-4">
            <h3 className="text-sm font-medium text-gray-900">Top Queries</h3>
            <div className="space-y-2">
              {analytics.topQueries.map((query, index) => (
                <div key={index} className="flex items-center justify-between p-3 bg-gray-50 rounded-lg">
                  <div className="flex items-center space-x-3">
                    <span className="text-sm font-medium text-gray-500">
                      #{index + 1}
                    </span>
                    <span className="text-sm text-gray-900">{query.query}</span>
                  </div>
                  <div className="flex items-center space-x-2">
                    <span className="text-sm text-gray-700">
                      {query.count}
                    </span>
                    {getTrendIcon(query.trend)}
                  </div>
                </div>
              ))}
            </div>
          </div>
        )}

        {activeTab === 'filters' && (
          <div className="space-y-4">
            <h3 className="text-sm font-medium text-gray-900">Popular Filters</h3>
            <div className="space-y-2">
              {analytics.popularFilters.map((filter, index) => (
                <div key={index} className="flex items-center justify-between">
                  <span className="text-sm text-gray-700">{filter.filter}</span>
                  <div className="flex items-center space-x-2">
                    <div className="w-20 bg-gray-200 rounded-full h-2">
                      <div
                        className="bg-green-600 h-2 rounded-full"
                        style={{ width: `${filter.percentage * 100}%` }}
                      />
                    </div>
                    <span className="text-sm text-gray-500 w-12 text-right">
                      {formatPercentage(filter.percentage)}
                    </span>
                  </div>
                </div>
              ))}
            </div>
          </div>
        )}
      </div>
    </div>
  )
}

export default MobileSearchAnalytics
