// Mobile-optimized admin connectors page
// Week 8: Mobile/responsive UX with PWA support

import React, { useState, useEffect, useCallback } from 'react'
import { 
  Plus, 
  Settings, 
  TestTube, 
  Trash2, 
  RefreshCw, 
  ExternalLink,
  Database,
  Cloud,
  Server
} from 'lucide-react'

interface Connector {
  id: string
  name: string
  type: 's3' | 'postgres' | 'ckan'
  config: any
  status: 'active' | 'inactive' | 'error'
  lastSync?: string
  entryCount?: number
}

const MobileAdminConnectorsPage: React.FC = () => {
  const [connectors, setConnectors] = useState<Connector[]>([])
  const [loading, setLoading] = useState(true)
  const [showAddForm, setShowAddForm] = useState(false)
  const [selectedConnector, setSelectedConnector] = useState<Connector | null>(null)

  // Load connectors
  useEffect(() => {
    const loadConnectors = async () => {
      setLoading(true)
      try {
        // TODO: Replace with actual API call
        const mockConnectors: Connector[] = [
          {
            id: '1',
            name: 'Production S3 Bucket',
            type: 's3',
            config: { bucket: 'prod-data', region: 'us-east-1' },
            status: 'active',
            lastSync: new Date().toISOString(),
            entryCount: 1250
          },
          {
            id: '2',
            name: 'Analytics Database',
            type: 'postgres',
            config: { host: 'db.example.com', database: 'analytics' },
            status: 'active',
            lastSync: new Date().toISOString(),
            entryCount: 890
          },
          {
            id: '3',
            name: 'Open Data Portal',
            type: 'ckan',
            config: { baseUrl: 'https://data.example.com' },
            status: 'error',
            lastSync: new Date(Date.now() - 86400000).toISOString(),
            entryCount: 0
          }
        ]
        
        setConnectors(mockConnectors)
      } catch (error) {
        console.error('Failed to load connectors:', error)
      } finally {
        setLoading(false)
      }
    }

    loadConnectors()
  }, [])

  // Handle add connector
  const handleAddConnector = useCallback(() => {
    setShowAddForm(true)
  }, [])

  // Handle test connector
  const handleTestConnector = useCallback(async (connector: Connector) => {
    try {
      // TODO: Implement connector test
      console.log('Testing connector:', connector)
    } catch (error) {
      console.error('Failed to test connector:', error)
    }
  }, [])

  // Handle sync connector
  const handleSyncConnector = useCallback(async (connector: Connector) => {
    try {
      // TODO: Implement connector sync
      console.log('Syncing connector:', connector)
    } catch (error) {
      console.error('Failed to sync connector:', error)
    }
  }, [])

  // Handle delete connector
  const handleDeleteConnector = useCallback(async (connector: Connector) => {
    if (window.confirm(`Are you sure you want to delete "${connector.name}"?`)) {
      try {
        // TODO: Implement connector deletion
        console.log('Deleting connector:', connector)
        setConnectors(prev => prev.filter(c => c.id !== connector.id))
      } catch (error) {
        console.error('Failed to delete connector:', error)
      }
    }
  }, [])

  // Get connector icon
  const getConnectorIcon = (type: string) => {
    switch (type) {
      case 's3':
        return <Cloud className="w-5 h-5 text-blue-500" />
      case 'postgres':
        return <Database className="w-5 h-5 text-green-500" />
      case 'ckan':
        return <Server className="w-5 h-5 text-purple-500" />
      default:
        return <Database className="w-5 h-5 text-gray-500" />
    }
  }

  // Get status color
  const getStatusColor = (status: string) => {
    switch (status) {
      case 'active':
        return 'bg-green-100 text-green-800'
      case 'inactive':
        return 'bg-gray-100 text-gray-800'
      case 'error':
        return 'bg-red-100 text-red-800'
      default:
        return 'bg-gray-100 text-gray-800'
    }
  }

  // Format date
  const formatDate = (date: string) => {
    const d = new Date(date)
    const now = new Date()
    const diffTime = Math.abs(now.getTime() - d.getTime())
    const diffDays = Math.ceil(diffTime / (1000 * 60 * 60 * 24))
    
    if (diffDays === 1) return 'Yesterday'
    if (diffDays < 7) return `${diffDays} days ago`
    if (diffDays < 30) return `${Math.ceil(diffDays / 7)} weeks ago`
    return d.toLocaleDateString()
  }

  if (loading) {
    return (
      <div className="min-h-screen bg-gray-50 flex items-center justify-center">
        <div className="text-center">
          <div className="w-8 h-8 border-2 border-blue-500 border-t-transparent rounded-full animate-spin mx-auto mb-4"></div>
          <p className="text-sm text-gray-600">Loading connectors...</p>
        </div>
      </div>
    )
  }

  return (
    <div className="min-h-screen bg-gray-50">
      {/* Header */}
      <div className="bg-white border-b border-gray-200 px-4 py-3">
        <div className="flex items-center justify-between">
          <div>
            <h1 className="text-lg font-semibold text-gray-900">
              Data Connectors
            </h1>
            <p className="text-sm text-gray-600">
              Manage external data sources
            </p>
          </div>
          <button
            onClick={handleAddConnector}
            className="flex items-center space-x-2 px-3 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 transition-colors"
          >
            <Plus className="w-4 h-4" />
            <span className="text-sm font-medium">Add</span>
          </button>
        </div>
      </div>

      {/* Connectors List */}
      <div className="divide-y divide-gray-100">
        {connectors.length === 0 ? (
          <div className="p-8 text-center text-gray-500">
            <Database className="w-12 h-12 mx-auto mb-3 text-gray-300" />
            <p className="text-sm">No connectors configured</p>
            <p className="text-xs mt-2">Add a connector to get started</p>
          </div>
        ) : (
          connectors.map((connector) => (
            <div key={connector.id} className="p-4 bg-white">
              <div className="flex items-start space-x-3">
                {/* Icon */}
                <div className="flex-shrink-0">
                  {getConnectorIcon(connector.type)}
                </div>

                {/* Content */}
                <div className="flex-1 min-w-0">
                  <div className="flex items-start justify-between">
                    <div className="flex-1 min-w-0">
                      <h3 className="text-sm font-medium text-gray-900 truncate">
                        {connector.name}
                      </h3>
                      <p className="text-xs text-gray-600 mt-1">
                        {connector.type.toUpperCase()} • {connector.entryCount?.toLocaleString()} entries
                      </p>
                    </div>
                    
                    <span className={`inline-flex items-center px-2 py-1 rounded-full text-xs font-medium ${getStatusColor(connector.status)}`}>
                      {connector.status}
                    </span>
                  </div>
                  
                  {/* Last Sync */}
                  {connector.lastSync && (
                    <div className="mt-2 text-xs text-gray-500">
                      Last sync: {formatDate(connector.lastSync)}
                    </div>
                  )}

                  {/* Actions */}
                  <div className="mt-3 flex items-center space-x-2">
                    <button
                      onClick={() => handleTestConnector(connector)}
                      className="flex items-center space-x-1 px-2 py-1 text-xs text-gray-600 hover:text-gray-900 hover:bg-gray-100 rounded transition-colors"
                    >
                      <TestTube className="w-3 h-3" />
                      <span>Test</span>
                    </button>
                    
                    <button
                      onClick={() => handleSyncConnector(connector)}
                      className="flex items-center space-x-1 px-2 py-1 text-xs text-gray-600 hover:text-gray-900 hover:bg-gray-100 rounded transition-colors"
                    >
                      <RefreshCw className="w-3 h-3" />
                      <span>Sync</span>
                    </button>
                    
                    <button
                      onClick={() => setSelectedConnector(connector)}
                      className="flex items-center space-x-1 px-2 py-1 text-xs text-gray-600 hover:text-gray-900 hover:bg-gray-100 rounded transition-colors"
                    >
                      <Settings className="w-3 h-3" />
                      <span>Edit</span>
                    </button>
                    
                    <button
                      onClick={() => handleDeleteConnector(connector)}
                      className="flex items-center space-x-1 px-2 py-1 text-xs text-red-600 hover:text-red-900 hover:bg-red-50 rounded transition-colors"
                    >
                      <Trash2 className="w-3 h-3" />
                      <span>Delete</span>
                    </button>
                  </div>
                </div>
              </div>
            </div>
          ))
        )}
      </div>

      {/* Add Connector Form */}
      {showAddForm && (
        <div className="fixed inset-0 bg-black bg-opacity-50 z-50 flex items-end">
          <div className="w-full bg-white rounded-t-lg">
            {/* Header */}
            <div className="p-4 border-b border-gray-200">
              <div className="flex items-center justify-between">
                <h3 className="text-lg font-semibold text-gray-900">
                  Add Connector
                </h3>
                <button
                  onClick={() => setShowAddForm(false)}
                  className="p-2 text-gray-400 hover:text-gray-600 transition-colors"
                >
                  <X className="w-5 h-5" />
                </button>
              </div>
            </div>

            {/* Form */}
            <div className="p-4 space-y-4">
              {/* Connector Type */}
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-2">
                  Connector Type
                </label>
                <div className="grid grid-cols-3 gap-2">
                  {[
                    { type: 's3', label: 'S3 Bucket', icon: Cloud },
                    { type: 'postgres', label: 'PostgreSQL', icon: Database },
                    { type: 'ckan', label: 'CKAN API', icon: Server },
                  ].map(({ type, label, icon: Icon }) => (
                    <button
                      key={type}
                      className="flex flex-col items-center space-y-2 p-3 border border-gray-200 rounded-lg hover:bg-gray-50 transition-colors"
                    >
                      <Icon className="w-6 h-6 text-gray-600" />
                      <span className="text-xs text-gray-700">{label}</span>
                    </button>
                  ))}
                </div>
              </div>

              {/* Name */}
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-2">
                  Name
                </label>
                <input
                  type="text"
                  className="w-full px-3 py-2 border border-gray-200 rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                  placeholder="Enter connector name"
                />
              </div>

              {/* Configuration */}
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-2">
                  Configuration
                </label>
                <textarea
                  className="w-full px-3 py-2 border border-gray-200 rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                  rows={4}
                  placeholder="Enter configuration JSON"
                />
              </div>

              {/* Actions */}
              <div className="flex items-center space-x-3 pt-4">
                <button
                  onClick={() => setShowAddForm(false)}
                  className="flex-1 px-4 py-2 border border-gray-200 text-gray-700 rounded-md hover:bg-gray-50 transition-colors"
                >
                  Cancel
                </button>
                <button
                  onClick={() => {
                    // TODO: Implement connector creation
                    setShowAddForm(false)
                  }}
                  className="flex-1 px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 transition-colors"
                >
                  Add Connector
                </button>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  )
}

export default MobileAdminConnectorsPage
