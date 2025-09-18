// Mobile-optimized search settings component
// Week 8: Mobile/responsive UX with PWA support

import React, { useState, useCallback } from 'react'
import { 
  Settings, 
  Search, 
  Bell, 
  Eye, 
  Shield, 
  Database,
  ToggleLeft,
  ToggleRight,
  ChevronRight,
  Info
} from 'lucide-react'

interface SearchSettings {
  enableSemanticSearch: boolean
  enableAutoComplete: boolean
  enableSearchHistory: boolean
  enableSearchSuggestions: boolean
  enableSearchAnalytics: boolean
  enableSearchPerformance: boolean
  defaultSearchMode: 'keyword' | 'semantic' | 'hybrid'
  resultsPerPage: number
  enableSearchFilters: boolean
  enableSearchSort: boolean
  enableSearchExport: boolean
  enableSearchSharing: boolean
  enableSearchBookmarks: boolean
  enableSearchNotifications: boolean
  enableSearchPrivacy: boolean
  enableSearchSecurity: boolean
}

interface MobileSearchSettingsProps {
  settings: SearchSettings
  onSettingsChange: (settings: SearchSettings) => void
  className?: string
}

export const MobileSearchSettings: React.FC<MobileSearchSettingsProps> = ({
  settings,
  onSettingsChange,
  className = ''
}) => {
  const [activeTab, setActiveTab] = useState<'general' | 'privacy' | 'advanced'>('general')

  // Handle setting change
  const handleSettingChange = useCallback((key: keyof SearchSettings, value: any) => {
    onSettingsChange({
      ...settings,
      [key]: value
    })
  }, [settings, onSettingsChange])

  // Toggle boolean setting
  const toggleSetting = useCallback((key: keyof SearchSettings) => {
    handleSettingChange(key, !settings[key])
  }, [settings, handleSettingChange])

  // Get toggle icon
  const getToggleIcon = (enabled: boolean) => {
    return enabled ? (
      <ToggleRight className="w-6 h-6 text-blue-600" />
    ) : (
      <ToggleLeft className="w-6 h-6 text-gray-400" />
    )
  }

  return (
    <div className={`bg-white ${className}`}>
      {/* Header */}
      <div className="p-4 border-b border-gray-200">
        <div className="flex items-center justify-between">
          <div>
            <h2 className="text-lg font-semibold text-gray-900">
              Search Settings
            </h2>
            <p className="text-sm text-gray-600">
              Customize your search experience
            </p>
          </div>
          
          <Settings className="w-6 h-6 text-gray-400" />
        </div>
      </div>

      {/* Tabs */}
      <div className="flex border-b border-gray-200">
        {[
          { id: 'general', label: 'General', icon: Search },
          { id: 'privacy', label: 'Privacy', icon: Shield },
          { id: 'advanced', label: 'Advanced', icon: Database },
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
        {activeTab === 'general' && (
          <div className="space-y-6">
            {/* Search Mode */}
            <div>
              <h3 className="text-sm font-medium text-gray-900 mb-3">Search Mode</h3>
              <div className="space-y-2">
                {[
                  { value: 'keyword', label: 'Keyword Search', description: 'Traditional text-based search' },
                  { value: 'semantic', label: 'Semantic Search', description: 'AI-powered meaning-based search' },
                  { value: 'hybrid', label: 'Hybrid Search', description: 'Combines keyword and semantic search' },
                ].map(({ value, label, description }) => (
                  <button
                    key={value}
                    onClick={() => handleSettingChange('defaultSearchMode', value)}
                    className={`w-full flex items-center justify-between p-3 rounded-lg border transition-colors ${
                      settings.defaultSearchMode === value
                        ? 'border-blue-200 bg-blue-50'
                        : 'border-gray-200 hover:bg-gray-50'
                    }`}
                  >
                    <div className="text-left">
                      <p className="text-sm font-medium text-gray-900">{label}</p>
                      <p className="text-xs text-gray-500">{description}</p>
                    </div>
                    {settings.defaultSearchMode === value && (
                      <div className="w-2 h-2 bg-blue-600 rounded-full" />
                    )}
                  </button>
                ))}
              </div>
            </div>

            {/* Results Per Page */}
            <div>
              <h3 className="text-sm font-medium text-gray-900 mb-3">Results Per Page</h3>
              <div className="grid grid-cols-3 gap-2">
                {[10, 25, 50].map((count) => (
                  <button
                    key={count}
                    onClick={() => handleSettingChange('resultsPerPage', count)}
                    className={`p-3 text-center rounded-lg border transition-colors ${
                      settings.resultsPerPage === count
                        ? 'border-blue-200 bg-blue-50 text-blue-700'
                        : 'border-gray-200 hover:bg-gray-50'
                    }`}
                  >
                    <span className="text-sm font-medium">{count}</span>
                  </button>
                ))}
              </div>
            </div>

            {/* Search Features */}
            <div>
              <h3 className="text-sm font-medium text-gray-900 mb-3">Search Features</h3>
              <div className="space-y-3">
                {[
                  { key: 'enableSemanticSearch', label: 'Semantic Search', description: 'AI-powered search understanding' },
                  { key: 'enableAutoComplete', label: 'Auto Complete', description: 'Search suggestions as you type' },
                  { key: 'enableSearchHistory', label: 'Search History', description: 'Remember your previous searches' },
                  { key: 'enableSearchSuggestions', label: 'Search Suggestions', description: 'Helpful search recommendations' },
                ].map(({ key, label, description }) => (
                  <div key={key} className="flex items-center justify-between">
                    <div className="flex-1">
                      <p className="text-sm font-medium text-gray-900">{label}</p>
                      <p className="text-xs text-gray-500">{description}</p>
                    </div>
                    <button
                      onClick={() => toggleSetting(key as keyof SearchSettings)}
                      className="ml-3"
                    >
                      {getToggleIcon(settings[key as keyof SearchSettings] as boolean)}
                    </button>
                  </div>
                ))}
              </div>
            </div>
          </div>
        )}

        {activeTab === 'privacy' && (
          <div className="space-y-6">
            {/* Privacy Settings */}
            <div>
              <h3 className="text-sm font-medium text-gray-900 mb-3">Privacy & Security</h3>
              <div className="space-y-3">
                {[
                  { key: 'enableSearchPrivacy', label: 'Search Privacy', description: 'Protect your search queries' },
                  { key: 'enableSearchSecurity', label: 'Search Security', description: 'Secure search connections' },
                  { key: 'enableSearchAnalytics', label: 'Search Analytics', description: 'Help improve search quality' },
                ].map(({ key, label, description }) => (
                  <div key={key} className="flex items-center justify-between">
                    <div className="flex-1">
                      <p className="text-sm font-medium text-gray-900">{label}</p>
                      <p className="text-xs text-gray-500">{description}</p>
                    </div>
                    <button
                      onClick={() => toggleSetting(key as keyof SearchSettings)}
                      className="ml-3"
                    >
                      {getToggleIcon(settings[key as keyof SearchSettings] as boolean)}
                    </button>
                  </div>
                ))}
              </div>
            </div>

            {/* Data Collection */}
            <div>
              <h3 className="text-sm font-medium text-gray-900 mb-3">Data Collection</h3>
              <div className="bg-blue-50 border border-blue-200 rounded-lg p-3">
                <div className="flex items-start space-x-2">
                  <Info className="w-4 h-4 text-blue-600 flex-shrink-0 mt-0.5" />
                  <div>
                    <p className="text-sm text-blue-800">
                      We collect minimal data to improve your search experience. Your queries are encrypted and never shared with third parties.
                    </p>
                  </div>
                </div>
              </div>
            </div>
          </div>
        )}

        {activeTab === 'advanced' && (
          <div className="space-y-6">
            {/* Advanced Features */}
            <div>
              <h3 className="text-sm font-medium text-gray-900 mb-3">Advanced Features</h3>
              <div className="space-y-3">
                {[
                  { key: 'enableSearchFilters', label: 'Search Filters', description: 'Advanced filtering options' },
                  { key: 'enableSearchSort', label: 'Search Sort', description: 'Sort search results' },
                  { key: 'enableSearchExport', label: 'Search Export', description: 'Export search results' },
                  { key: 'enableSearchSharing', label: 'Search Sharing', description: 'Share search results' },
                  { key: 'enableSearchBookmarks', label: 'Search Bookmarks', description: 'Bookmark search results' },
                  { key: 'enableSearchNotifications', label: 'Search Notifications', description: 'Get notified of new results' },
                ].map(({ key, label, description }) => (
                  <div key={key} className="flex items-center justify-between">
                    <div className="flex-1">
                      <p className="text-sm font-medium text-gray-900">{label}</p>
                      <p className="text-xs text-gray-500">{description}</p>
                    </div>
                    <button
                      onClick={() => toggleSetting(key as keyof SearchSettings)}
                      className="ml-3"
                    >
                      {getToggleIcon(settings[key as keyof SearchSettings] as boolean)}
                    </button>
                  </div>
                ))}
              </div>
            </div>

            {/* Performance Settings */}
            <div>
              <h3 className="text-sm font-medium text-gray-900 mb-3">Performance</h3>
              <div className="space-y-3">
                {[
                  { key: 'enableSearchPerformance', label: 'Performance Monitoring', description: 'Monitor search performance' },
                ].map(({ key, label, description }) => (
                  <div key={key} className="flex items-center justify-between">
                    <div className="flex-1">
                      <p className="text-sm font-medium text-gray-900">{label}</p>
                      <p className="text-xs text-gray-500">{description}</p>
                    </div>
                    <button
                      onClick={() => toggleSetting(key as keyof SearchSettings)}
                      className="ml-3"
                    >
                      {getToggleIcon(settings[key as keyof SearchSettings] as boolean)}
                    </button>
                  </div>
                ))}
              </div>
            </div>
          </div>
        )}
      </div>
    </div>
  )
}

export default MobileSearchSettings
