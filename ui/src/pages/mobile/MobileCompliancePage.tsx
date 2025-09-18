// Mobile-optimized compliance page
// Week 8: Mobile/responsive UX with PWA support

import React, { useState, useEffect, useCallback } from 'react'
import { 
  Shield, 
  Lock, 
  Clock, 
  Download, 
  AlertTriangle, 
  CheckCircle,
  FileText,
  Calendar,
  User,
  Filter,
  Search
} from 'lucide-react'

interface ComplianceEntry {
  id: string
  name: string
  path: string
  classification: 'public' | 'internal' | 'restricted' | 'secret'
  retentionUntil?: string
  legalHold: boolean
  lastModified: string
  author: string
  size: number
}

interface AuditLog {
  id: string
  action: string
  user: string
  timestamp: string
  details: string
  ipAddress: string
}

const MobileCompliancePage: React.FC = () => {
  const [entries, setEntries] = useState<ComplianceEntry[]>([])
  const [auditLogs, setAuditLogs] = useState<AuditLog[]>([])
  const [loading, setLoading] = useState(true)
  const [activeTab, setActiveTab] = useState<'entries' | 'audit' | 'export'>('entries')
  const [showFilters, setShowFilters] = useState(false)
  const [searchQuery, setSearchQuery] = useState('')

  // Load compliance data
  useEffect(() => {
    const loadData = async () => {
      setLoading(true)
      try {
        // TODO: Replace with actual API calls
        const mockEntries: ComplianceEntry[] = [
          {
            id: '1',
            name: 'customer_data.csv',
            path: '/datasets/customer_data.csv',
            classification: 'restricted',
            retentionUntil: '2025-12-31',
            legalHold: true,
            lastModified: new Date().toISOString(),
            author: 'John Doe',
            size: 1024000
          },
          {
            id: '2',
            name: 'sales_report.pdf',
            path: '/reports/sales_report.pdf',
            classification: 'internal',
            retentionUntil: '2024-12-31',
            legalHold: false,
            lastModified: new Date().toISOString(),
            author: 'Jane Smith',
            size: 512000
          },
          {
            id: '3',
            name: 'public_dataset.json',
            path: '/public/public_dataset.json',
            classification: 'public',
            retentionUntil: undefined,
            legalHold: false,
            lastModified: new Date().toISOString(),
            author: 'Bob Johnson',
            size: 256000
          }
        ]

        const mockAuditLogs: AuditLog[] = [
          {
            id: '1',
            action: 'VIEW',
            user: 'John Doe',
            timestamp: new Date().toISOString(),
            details: 'Viewed customer_data.csv',
            ipAddress: '192.168.1.100'
          },
          {
            id: '2',
            action: 'DOWNLOAD',
            user: 'Jane Smith',
            timestamp: new Date(Date.now() - 3600000).toISOString(),
            details: 'Downloaded sales_report.pdf',
            ipAddress: '192.168.1.101'
          },
          {
            id: '3',
            action: 'DELETE_ATTEMPT',
            user: 'Bob Johnson',
            timestamp: new Date(Date.now() - 7200000).toISOString(),
            details: 'Attempted to delete customer_data.csv (blocked by legal hold)',
            ipAddress: '192.168.1.102'
          }
        ]

        setEntries(mockEntries)
        setAuditLogs(mockAuditLogs)
      } catch (error) {
        console.error('Failed to load compliance data:', error)
      } finally {
        setLoading(false)
      }
    }

    loadData()
  }, [])

  // Get classification color
  const getClassificationColor = (classification: string) => {
    switch (classification) {
      case 'public':
        return 'bg-green-100 text-green-800'
      case 'internal':
        return 'bg-blue-100 text-blue-800'
      case 'restricted':
        return 'bg-yellow-100 text-yellow-800'
      case 'secret':
        return 'bg-red-100 text-red-800'
      default:
        return 'bg-gray-100 text-gray-800'
    }
  }

  // Get action color
  const getActionColor = (action: string) => {
    switch (action) {
      case 'VIEW':
        return 'bg-blue-100 text-blue-800'
      case 'DOWNLOAD':
        return 'bg-green-100 text-green-800'
      case 'DELETE_ATTEMPT':
        return 'bg-red-100 text-red-800'
      case 'MODIFY':
        return 'bg-yellow-100 text-yellow-800'
      default:
        return 'bg-gray-100 text-gray-800'
    }
  }

  // Format date
  const formatDate = (date: string) => {
    const d = new Date(date)
    return d.toLocaleDateString() + ' ' + d.toLocaleTimeString()
  }

  // Format file size
  const formatFileSize = (bytes: number) => {
    if (bytes === 0) return '0 B'
    const k = 1024
    const sizes = ['B', 'KB', 'MB', 'GB']
    const i = Math.floor(Math.log(bytes) / Math.log(k))
    return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i]
  }

  // Handle export
  const handleExport = useCallback(async (format: 'json' | 'csv' | 'pdf') => {
    try {
      // TODO: Implement compliance export
      console.log('Exporting compliance data in', format, 'format')
    } catch (error) {
      console.error('Failed to export compliance data:', error)
    }
  }, [])

  // Filter entries
  const filteredEntries = entries.filter(entry =>
    entry.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
    entry.path.toLowerCase().includes(searchQuery.toLowerCase())
  )

  if (loading) {
    return (
      <div className="min-h-screen bg-gray-50 flex items-center justify-center">
        <div className="text-center">
          <div className="w-8 h-8 border-2 border-blue-500 border-t-transparent rounded-full animate-spin mx-auto mb-4"></div>
          <p className="text-sm text-gray-600">Loading compliance data...</p>
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
              Compliance & Legal Hold
            </h1>
            <p className="text-sm text-gray-600">
              Manage data retention and legal holds
            </p>
          </div>
          <button
            onClick={() => setShowFilters(!showFilters)}
            className="p-2 text-gray-600 hover:text-gray-900 hover:bg-gray-100 rounded-md transition-colors"
          >
            <Filter className="w-5 h-5" />
          </button>
        </div>
      </div>

      {/* Tabs */}
      <div className="bg-white border-b border-gray-200">
        <div className="flex">
          {[
            { id: 'entries', label: 'Entries', icon: FileText },
            { id: 'audit', label: 'Audit Log', icon: Shield },
            { id: 'export', label: 'Export', icon: Download },
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
      </div>

      {/* Search */}
      {activeTab === 'entries' && (
        <div className="bg-white border-b border-gray-200 px-4 py-3">
          <div className="relative">
            <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 w-4 h-4 text-gray-400" />
            <input
              type="text"
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              placeholder="Search entries..."
              className="w-full pl-10 pr-4 py-2 border border-gray-200 rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
            />
          </div>
        </div>
      )}

      {/* Content */}
      <div className="p-4">
        {activeTab === 'entries' && (
          <div className="space-y-4">
            {filteredEntries.length === 0 ? (
              <div className="text-center text-gray-500 py-8">
                <FileText className="w-12 h-12 mx-auto mb-3 text-gray-300" />
                <p className="text-sm">No entries found</p>
              </div>
            ) : (
              filteredEntries.map((entry) => (
                <div key={entry.id} className="bg-white rounded-lg border border-gray-200 p-4">
                  <div className="flex items-start justify-between">
                    <div className="flex-1 min-w-0">
                      <h3 className="text-sm font-medium text-gray-900 truncate">
                        {entry.name}
                      </h3>
                      <p className="text-xs text-gray-600 mt-1 truncate">
                        {entry.path}
                      </p>
                      
                      <div className="mt-2 flex items-center space-x-2">
                        <span className={`inline-flex items-center px-2 py-1 rounded-full text-xs font-medium ${getClassificationColor(entry.classification)}`}>
                          {entry.classification}
                        </span>
                        
                        {entry.legalHold && (
                          <span className="inline-flex items-center px-2 py-1 rounded-full text-xs font-medium bg-red-100 text-red-800">
                            <Lock className="w-3 h-3 mr-1" />
                            Legal Hold
                          </span>
                        )}
                        
                        {entry.retentionUntil && (
                          <span className="inline-flex items-center px-2 py-1 rounded-full text-xs font-medium bg-yellow-100 text-yellow-800">
                            <Clock className="w-3 h-3 mr-1" />
                            Retain until {new Date(entry.retentionUntil).toLocaleDateString()}
                          </span>
                        )}
                      </div>
                      
                      <div className="mt-2 flex items-center space-x-4 text-xs text-gray-500">
                        <span className="flex items-center space-x-1">
                          <User className="w-3 h-3" />
                          <span>{entry.author}</span>
                        </span>
                        <span className="flex items-center space-x-1">
                          <Calendar className="w-3 h-3" />
                          <span>{formatDate(entry.lastModified)}</span>
                        </span>
                        <span>{formatFileSize(entry.size)}</span>
                      </div>
                    </div>
                    
                    <div className="flex items-center space-x-1 ml-2">
                      <button className="p-2 text-gray-400 hover:text-gray-600 transition-colors">
                        <Settings className="w-4 h-4" />
                      </button>
                    </div>
                  </div>
                </div>
              ))
            )}
          </div>
        )}

        {activeTab === 'audit' && (
          <div className="space-y-4">
            {auditLogs.length === 0 ? (
              <div className="text-center text-gray-500 py-8">
                <Shield className="w-12 h-12 mx-auto mb-3 text-gray-300" />
                <p className="text-sm">No audit logs found</p>
              </div>
            ) : (
              auditLogs.map((log) => (
                <div key={log.id} className="bg-white rounded-lg border border-gray-200 p-4">
                  <div className="flex items-start justify-between">
                    <div className="flex-1 min-w-0">
                      <div className="flex items-center space-x-2">
                        <span className={`inline-flex items-center px-2 py-1 rounded-full text-xs font-medium ${getActionColor(log.action)}`}>
                          {log.action}
                        </span>
                        <span className="text-sm font-medium text-gray-900">
                          {log.user}
                        </span>
                      </div>
                      
                      <p className="text-sm text-gray-600 mt-1">
                        {log.details}
                      </p>
                      
                      <div className="mt-2 flex items-center space-x-4 text-xs text-gray-500">
                        <span className="flex items-center space-x-1">
                          <Calendar className="w-3 h-3" />
                          <span>{formatDate(log.timestamp)}</span>
                        </span>
                        <span>IP: {log.ipAddress}</span>
                      </div>
                    </div>
                  </div>
                </div>
              ))
            )}
          </div>
        )}

        {activeTab === 'export' && (
          <div className="space-y-4">
            <div className="bg-white rounded-lg border border-gray-200 p-4">
              <h3 className="text-sm font-medium text-gray-900 mb-3">
                Export Compliance Data
              </h3>
              
              <div className="space-y-3">
                <button
                  onClick={() => handleExport('json')}
                  className="w-full flex items-center justify-between p-3 border border-gray-200 rounded-lg hover:bg-gray-50 transition-colors"
                >
                  <div className="flex items-center space-x-3">
                    <FileText className="w-5 h-5 text-blue-500" />
                    <span className="text-sm font-medium text-gray-900">JSON Export</span>
                  </div>
                  <Download className="w-4 h-4 text-gray-400" />
                </button>
                
                <button
                  onClick={() => handleExport('csv')}
                  className="w-full flex items-center justify-between p-3 border border-gray-200 rounded-lg hover:bg-gray-50 transition-colors"
                >
                  <div className="flex items-center space-x-3">
                    <FileText className="w-5 h-5 text-green-500" />
                    <span className="text-sm font-medium text-gray-900">CSV Export</span>
                  </div>
                  <Download className="w-4 h-4 text-gray-400" />
                </button>
                
                <button
                  onClick={() => handleExport('pdf')}
                  className="w-full flex items-center justify-between p-3 border border-gray-200 rounded-lg hover:bg-gray-50 transition-colors"
                >
                  <div className="flex items-center space-x-3">
                    <FileText className="w-5 h-5 text-red-500" />
                    <span className="text-sm font-medium text-gray-900">PDF Report</span>
                  </div>
                  <Download className="w-4 h-4 text-gray-400" />
                </button>
              </div>
            </div>
            
            <div className="bg-yellow-50 border border-yellow-200 rounded-lg p-4">
              <div className="flex items-start space-x-3">
                <AlertTriangle className="w-5 h-5 text-yellow-600 flex-shrink-0 mt-0.5" />
                <div>
                  <h4 className="text-sm font-medium text-yellow-800">
                    Export Notice
                  </h4>
                  <p className="text-sm text-yellow-700 mt-1">
                    Exported data may contain sensitive information. Ensure proper handling and storage according to your organization's data protection policies.
                  </p>
                </div>
              </div>
            </div>
          </div>
        )}
      </div>
    </div>
  )
}

export default MobileCompliancePage
