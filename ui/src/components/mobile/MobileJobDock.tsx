// Mobile-optimized job dock component
// Week 8: Mobile/responsive UX with PWA support

import React, { useState, useEffect } from 'react'
import { 
  Clock, 
  CheckCircle, 
  XCircle, 
  AlertCircle, 
  ChevronUp, 
  ChevronDown,
  Trash2,
  Eye,
  RefreshCw
} from 'lucide-react'
import { useJobStore } from '@/stores/jobs'

interface MobileJobDockProps {
  className?: string
}

export const MobileJobDock: React.FC<MobileJobDockProps> = ({ className = '' }) => {
  const { jobs, clearCompleted, retryJob } = useJobStore()
  const [isExpanded, setIsExpanded] = useState(false)
  const [isVisible, setIsVisible] = useState(false)

  // Show dock if there are active jobs
  useEffect(() => {
    const hasActiveJobs = jobs.some(job => job.status === 'running' || job.status === 'pending')
    setIsVisible(hasActiveJobs || jobs.length > 0)
  }, [jobs])

  // Auto-expand if there are running jobs
  useEffect(() => {
    const hasRunningJobs = jobs.some(job => job.status === 'running')
    if (hasRunningJobs) {
      setIsExpanded(true)
    }
  }, [jobs])

  if (!isVisible) {
    return null
  }

  const getStatusIcon = (status: string) => {
    switch (status) {
      case 'completed':
        return <CheckCircle className="w-4 h-4 text-green-500" />
      case 'failed':
        return <XCircle className="w-4 h-4 text-red-500" />
      case 'running':
        return <RefreshCw className="w-4 h-4 text-blue-500 animate-spin" />
      case 'pending':
        return <Clock className="w-4 h-4 text-yellow-500" />
      default:
        return <AlertCircle className="w-4 h-4 text-gray-500" />
    }
  }

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'completed':
        return 'bg-green-50 border-green-200'
      case 'failed':
        return 'bg-red-50 border-red-200'
      case 'running':
        return 'bg-blue-50 border-blue-200'
      case 'pending':
        return 'bg-yellow-50 border-yellow-200'
      default:
        return 'bg-gray-50 border-gray-200'
    }
  }

  const formatDuration = (startTime: Date, endTime?: Date) => {
    const end = endTime || new Date()
    const duration = end.getTime() - startTime.getTime()
    const seconds = Math.floor(duration / 1000)
    
    if (seconds < 60) {
      return `${seconds}s`
    } else if (seconds < 3600) {
      return `${Math.floor(seconds / 60)}m ${seconds % 60}s`
    } else {
      return `${Math.floor(seconds / 3600)}h ${Math.floor((seconds % 3600) / 60)}m`
    }
  }

  const formatFileSize = (bytes: number) => {
    if (bytes === 0) return '0 B'
    const k = 1024
    const sizes = ['B', 'KB', 'MB', 'GB']
    const i = Math.floor(Math.log(bytes) / Math.log(k))
    return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i]
  }

  const activeJobs = jobs.filter(job => job.status === 'running' || job.status === 'pending')
  const completedJobs = jobs.filter(job => job.status === 'completed' || job.status === 'failed')

  return (
    <div className={`fixed bottom-0 left-0 right-0 bg-white border-t border-gray-200 shadow-lg z-30 ${className}`}>
      {/* Header */}
      <div 
        className="flex items-center justify-between p-3 cursor-pointer"
        onClick={() => setIsExpanded(!isExpanded)}
      >
        <div className="flex items-center space-x-2">
          <div className="flex items-center space-x-1">
            {activeJobs.length > 0 && (
              <div className="w-2 h-2 bg-blue-500 rounded-full animate-pulse"></div>
            )}
            <span className="text-sm font-medium text-gray-900">
              Jobs ({jobs.length})
            </span>
          </div>
          {activeJobs.length > 0 && (
            <span className="text-xs text-blue-600 bg-blue-100 px-2 py-1 rounded-full">
              {activeJobs.length} active
            </span>
          )}
        </div>
        
        <div className="flex items-center space-x-2">
          {completedJobs.length > 0 && (
            <button
              onClick={(e) => {
                e.stopPropagation()
                clearCompleted()
              }}
              className="text-xs text-gray-500 hover:text-gray-700 transition-colors"
            >
              Clear
            </button>
          )}
          {isExpanded ? (
            <ChevronDown className="w-4 h-4 text-gray-500" />
          ) : (
            <ChevronUp className="w-4 h-4 text-gray-500" />
          )}
        </div>
      </div>

      {/* Job List */}
      {isExpanded && (
        <div className="max-h-64 overflow-y-auto border-t border-gray-100">
          {jobs.length === 0 ? (
            <div className="p-4 text-center text-gray-500 text-sm">
              No jobs running
            </div>
          ) : (
            <div className="divide-y divide-gray-100">
              {jobs.map((job) => (
                <div
                  key={job.id}
                  className={`p-3 ${getStatusColor(job.status)}`}
                >
                  <div className="flex items-start justify-between">
                    <div className="flex items-start space-x-3 flex-1 min-w-0">
                      {getStatusIcon(job.status)}
                      <div className="flex-1 min-w-0">
                        <div className="text-sm font-medium text-gray-900 truncate">
                          {job.type}
                        </div>
                        <div className="text-xs text-gray-600 truncate">
                          {job.description}
                        </div>
                        {job.progress !== undefined && (
                          <div className="mt-1">
                            <div className="flex items-center justify-between text-xs text-gray-500 mb-1">
                              <span>Progress</span>
                              <span>{job.progress}%</span>
                            </div>
                            <div className="w-full bg-gray-200 rounded-full h-1.5">
                              <div
                                className="bg-blue-600 h-1.5 rounded-full transition-all duration-300"
                                style={{ width: `${job.progress}%` }}
                              ></div>
                            </div>
                          </div>
                        )}
                        <div className="flex items-center justify-between mt-2 text-xs text-gray-500">
                          <span>
                            {formatDuration(job.startTime, job.endTime)}
                          </span>
                          {job.fileSize && (
                            <span>{formatFileSize(job.fileSize)}</span>
                          )}
                        </div>
                      </div>
                    </div>
                    
                    <div className="flex items-center space-x-1 ml-2">
                      {job.status === 'failed' && (
                        <button
                          onClick={() => retryJob(job.id)}
                          className="p-1 text-gray-400 hover:text-gray-600 transition-colors"
                          title="Retry job"
                        >
                          <RefreshCw className="w-3 h-3" />
                        </button>
                      )}
                      {job.status === 'completed' && (
                        <button
                          onClick={() => {
                            // TODO: View job details
                          }}
                          className="p-1 text-gray-400 hover:text-gray-600 transition-colors"
                          title="View details"
                        >
                          <Eye className="w-3 h-3" />
                        </button>
                      )}
                    </div>
                  </div>
                  
                  {/* Error Message */}
                  {job.error && (
                    <div className="mt-2 p-2 bg-red-50 border border-red-200 rounded text-xs text-red-700">
                      {job.error}
                    </div>
                  )}
                </div>
              ))}
            </div>
          )}
        </div>
      )}
    </div>
  )
}

export default MobileJobDock
