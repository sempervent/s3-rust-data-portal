// Mobile-optimized search performance component
// Week 8: Mobile/responsive UX with PWA support

import React, { useState, useCallback } from 'react'
import { 
  Zap, 
  Clock, 
  Database, 
  Cpu, 
  Memory,
  Network,
  TrendingUp,
  AlertTriangle,
  CheckCircle,
  XCircle
} from 'lucide-react'

interface PerformanceMetrics {
  averageResponseTime: number
  p95ResponseTime: number
  p99ResponseTime: number
  throughput: number
  errorRate: number
  cacheHitRate: number
  indexSize: number
  queryComplexity: number
  memoryUsage: number
  cpuUsage: number
}

interface PerformanceAlert {
  id: string
  type: 'warning' | 'error' | 'info'
  message: string
  timestamp: string
  resolved: boolean
}

interface MobileSearchPerformanceProps {
  metrics: PerformanceMetrics
  alerts: PerformanceAlert[]
  className?: string
}

export const MobileSearchPerformance: React.FC<MobileSearchPerformanceProps> = ({
  metrics,
  alerts,
  className = ''
}) => {
  const [activeTab, setActiveTab] = useState<'metrics' | 'alerts' | 'trends'>('metrics')

  // Format number
  const formatNumber = (num: number) => {
    if (num >= 1000000) {
      return (num / 1000000).toFixed(1) + 'M'
    } else if (num >= 1000) {
      return (num / 1000).toFixed(1) + 'K'
    }
    return num.toString()
  }

  // Format bytes
  const formatBytes = (bytes: number) => {
    if (bytes === 0) return '0 B'
    const k = 1024
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB']
    const i = Math.floor(Math.log(bytes) / Math.log(k))
    return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i]
  }

  // Format percentage
  const formatPercentage = (num: number) => {
    return Math.round(num * 100) + '%'
  }

  // Get performance status
  const getPerformanceStatus = (metric: string, value: number) => {
    switch (metric) {
      case 'averageResponseTime':
        if (value < 100) return 'good'
        if (value < 500) return 'warning'
        return 'error'
      case 'errorRate':
        if (value < 0.01) return 'good'
        if (value < 0.05) return 'warning'
        return 'error'
      case 'cacheHitRate':
        if (value > 0.8) return 'good'
        if (value > 0.6) return 'warning'
        return 'error'
      default:
        return 'good'
    }
  }

  // Get status icon
  const getStatusIcon = (status: string) => {
    switch (status) {
      case 'good':
        return <CheckCircle className="w-4 h-4 text-green-500" />
      case 'warning':
        return <AlertTriangle className="w-4 h-4 text-yellow-500" />
      case 'error':
        return <XCircle className="w-4 h-4 text-red-500" />
      default:
        return <CheckCircle className="w-4 h-4 text-gray-500" />
    }
  }

  // Get alert icon
  const getAlertIcon = (type: string) => {
    switch (type) {
      case 'error':
        return <XCircle className="w-4 h-4 text-red-500" />
      case 'warning':
        return <AlertTriangle className="w-4 h-4 text-yellow-500" />
      case 'info':
        return <CheckCircle className="w-4 h-4 text-blue-500" />
      default:
        return <CheckCircle className="w-4 h-4 text-gray-500" />
    }
  }

  // Format date
  const formatDate = (date: string) => {
    const d = new Date(date)
    return d.toLocaleDateString() + ' ' + d.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })
  }

  return (
    <div className={`bg-white ${className}`}>
      {/* Header */}
      <div className="p-4 border-b border-gray-200">
        <div className="flex items-center justify-between">
          <div>
            <h2 className="text-lg font-semibold text-gray-900">
              Search Performance
            </h2>
            <p className="text-sm text-gray-600">
              Monitor search system health and performance
            </p>
          </div>
          
          <div className="flex items-center space-x-2">
            <div className="flex items-center space-x-1">
              {getStatusIcon(getPerformanceStatus('averageResponseTime', metrics.averageResponseTime))}
              <span className="text-sm text-gray-600">System Health</span>
            </div>
          </div>
        </div>
      </div>

      {/* Tabs */}
      <div className="flex border-b border-gray-200">
        {[
          { id: 'metrics', label: 'Metrics', icon: BarChart3 },
          { id: 'alerts', label: 'Alerts', icon: AlertTriangle },
          { id: 'trends', label: 'Trends', icon: TrendingUp },
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
        {activeTab === 'metrics' && (
          <div className="space-y-6">
            {/* Response Time Metrics */}
            <div>
              <h3 className="text-sm font-medium text-gray-900 mb-3">Response Times</h3>
              <div className="grid grid-cols-1 gap-3">
                <div className="flex items-center justify-between p-3 bg-gray-50 rounded-lg">
                  <div className="flex items-center space-x-3">
                    <Clock className="w-5 h-5 text-blue-600" />
                    <span className="text-sm font-medium text-gray-900">Average</span>
                  </div>
                  <div className="flex items-center space-x-2">
                    <span className="text-lg font-bold text-gray-900">
                      {metrics.averageResponseTime}ms
                    </span>
                    {getStatusIcon(getPerformanceStatus('averageResponseTime', metrics.averageResponseTime))}
                  </div>
                </div>
                
                <div className="flex items-center justify-between p-3 bg-gray-50 rounded-lg">
                  <div className="flex items-center space-x-3">
                    <TrendingUp className="w-5 h-5 text-green-600" />
                    <span className="text-sm font-medium text-gray-900">P95</span>
                  </div>
                  <span className="text-lg font-bold text-gray-900">
                    {metrics.p95ResponseTime}ms
                  </span>
                </div>
                
                <div className="flex items-center justify-between p-3 bg-gray-50 rounded-lg">
                  <div className="flex items-center space-x-3">
                    <TrendingUp className="w-5 h-5 text-red-600" />
                    <span className="text-sm font-medium text-gray-900">P99</span>
                  </div>
                  <span className="text-lg font-bold text-gray-900">
                    {metrics.p99ResponseTime}ms
                  </span>
                </div>
              </div>
            </div>

            {/* System Metrics */}
            <div>
              <h3 className="text-sm font-medium text-gray-900 mb-3">System Metrics</h3>
              <div className="grid grid-cols-2 gap-3">
                <div className="p-3 bg-gray-50 rounded-lg">
                  <div className="flex items-center space-x-2 mb-2">
                    <Database className="w-4 h-4 text-blue-600" />
                    <span className="text-sm font-medium text-gray-900">Throughput</span>
                  </div>
                  <p className="text-lg font-bold text-gray-900">
                    {formatNumber(metrics.throughput)}/s
                  </p>
                </div>
                
                <div className="p-3 bg-gray-50 rounded-lg">
                  <div className="flex items-center space-x-2 mb-2">
                    <XCircle className="w-4 h-4 text-red-600" />
                    <span className="text-sm font-medium text-gray-900">Error Rate</span>
                  </div>
                  <p className="text-lg font-bold text-gray-900">
                    {formatPercentage(metrics.errorRate)}
                  </p>
                </div>
                
                <div className="p-3 bg-gray-50 rounded-lg">
                  <div className="flex items-center space-x-2 mb-2">
                    <Zap className="w-4 h-4 text-yellow-600" />
                    <span className="text-sm font-medium text-gray-900">Cache Hit</span>
                  </div>
                  <p className="text-lg font-bold text-gray-900">
                    {formatPercentage(metrics.cacheHitRate)}
                  </p>
                </div>
                
                <div className="p-3 bg-gray-50 rounded-lg">
                  <div className="flex items-center space-x-2 mb-2">
                    <Database className="w-4 h-4 text-purple-600" />
                    <span className="text-sm font-medium text-gray-900">Index Size</span>
                  </div>
                  <p className="text-lg font-bold text-gray-900">
                    {formatBytes(metrics.indexSize)}
                  </p>
                </div>
              </div>
            </div>

            {/* Resource Usage */}
            <div>
              <h3 className="text-sm font-medium text-gray-900 mb-3">Resource Usage</h3>
              <div className="space-y-3">
                <div className="flex items-center justify-between">
                  <div className="flex items-center space-x-2">
                    <Cpu className="w-4 h-4 text-blue-600" />
                    <span className="text-sm text-gray-700">CPU Usage</span>
                  </div>
                  <div className="flex items-center space-x-2">
                    <div className="w-20 bg-gray-200 rounded-full h-2">
                      <div
                        className="bg-blue-600 h-2 rounded-full"
                        style={{ width: `${metrics.cpuUsage * 100}%` }}
                      />
                    </div>
                    <span className="text-sm text-gray-500 w-12 text-right">
                      {formatPercentage(metrics.cpuUsage)}
                    </span>
                  </div>
                </div>
                
                <div className="flex items-center justify-between">
                  <div className="flex items-center space-x-2">
                    <Memory className="w-4 h-4 text-green-600" />
                    <span className="text-sm text-gray-700">Memory Usage</span>
                  </div>
                  <div className="flex items-center space-x-2">
                    <div className="w-20 bg-gray-200 rounded-full h-2">
                      <div
                        className="bg-green-600 h-2 rounded-full"
                        style={{ width: `${metrics.memoryUsage * 100}%` }}
                      />
                    </div>
                    <span className="text-sm text-gray-500 w-12 text-right">
                      {formatPercentage(metrics.memoryUsage)}
                    </span>
                  </div>
                </div>
              </div>
            </div>
          </div>
        )}

        {activeTab === 'alerts' && (
          <div className="space-y-4">
            <h3 className="text-sm font-medium text-gray-900">Performance Alerts</h3>
            {alerts.length === 0 ? (
              <div className="text-center py-8">
                <CheckCircle className="w-12 h-12 mx-auto mb-3 text-green-500" />
                <p className="text-sm text-gray-500">No active alerts</p>
                <p className="text-xs text-gray-400 mt-1">
                  System is running smoothly
                </p>
              </div>
            ) : (
              <div className="space-y-3">
                {alerts.map((alert) => (
                  <div
                    key={alert.id}
                    className={`p-3 rounded-lg border ${
                      alert.type === 'error'
                        ? 'bg-red-50 border-red-200'
                        : alert.type === 'warning'
                        ? 'bg-yellow-50 border-yellow-200'
                        : 'bg-blue-50 border-blue-200'
                    }`}
                  >
                    <div className="flex items-start space-x-3">
                      {getAlertIcon(alert.type)}
                      <div className="flex-1 min-w-0">
                        <p className="text-sm font-medium text-gray-900">
                          {alert.message}
                        </p>
                        <p className="text-xs text-gray-500 mt-1">
                          {formatDate(alert.timestamp)}
                        </p>
                      </div>
                      {alert.resolved && (
                        <span className="inline-flex items-center px-2 py-1 rounded-full text-xs font-medium bg-green-100 text-green-800">
                          Resolved
                        </span>
                      )}
                    </div>
                  </div>
                ))}
              </div>
            )}
          </div>
        )}

        {activeTab === 'trends' && (
          <div className="space-y-4">
            <h3 className="text-sm font-medium text-gray-900">Performance Trends</h3>
            <div className="text-center py-8">
              <TrendingUp className="w-12 h-12 mx-auto mb-3 text-gray-300" />
              <p className="text-sm text-gray-500">Trends coming soon</p>
              <p className="text-xs text-gray-400 mt-1">
                Historical performance data will be available here
              </p>
            </div>
          </div>
        )}
      </div>
    </div>
  )
}

export default MobileSearchPerformance
