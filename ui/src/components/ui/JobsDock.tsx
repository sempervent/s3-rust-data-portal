import React, { useState, useEffect, useRef } from 'react'
import { Button } from './button'
import { useAppStore } from '@/app/store'

export interface JobEvent {
  id: string
  type: 'sampling' | 'rdf' | 'antivirus' | 'upload' | 'metadata'
  status: 'pending' | 'running' | 'completed' | 'failed'
  repo: string
  path?: string
  progress?: number
  message?: string
  created_at: string
  updated_at: string
  result?: any
  error?: string
}

interface JobsDockProps {
  className?: string
}

export const JobsDock: React.FC<JobsDockProps> = ({ className = "" }) => {
  const [isExpanded, setIsExpanded] = useState(false)
  const [jobs, setJobs] = useState<JobEvent[]>([])
  const [isConnected, setIsConnected] = useState(false)
  const [usePolling, setUsePolling] = useState(false)
  const eventSourceRef = useRef<EventSource | null>(null)
  const pollingIntervalRef = useRef<NodeJS.Timeout | null>(null)
  const { addToast } = useAppStore()

  // Check for SSE support and feature flag
  const supportsSSE = typeof EventSource !== 'undefined'
  const enableSSE = supportsSSE && !usePolling

  useEffect(() => {
    if (enableSSE) {
      connectSSE()
    } else {
      startPolling()
    }

    // Listen for simulated job events
    const handleCustomJobEvent = (event: any) => {
      handleJobUpdate(event.detail)
    }
    
    window.addEventListener('job-update', handleCustomJobEvent)

    return () => {
      disconnectSSE()
      stopPolling()
      window.removeEventListener('job-update', handleCustomJobEvent)
    }
  }, [enableSSE])

  const connectSSE = () => {
    try {
      // In a real implementation, this would include auth headers via URL params or custom implementation
      const eventSource = new EventSource('/v1/jobs/stream')
      
      eventSource.onopen = () => {
        setIsConnected(true)
        console.log('Jobs SSE connected')
      }

      eventSource.onmessage = (event) => {
        try {
          const jobEvent: JobEvent = JSON.parse(event.data)
          handleJobUpdate(jobEvent)
        } catch (error) {
          console.error('Failed to parse job event:', error)
        }
      }

      eventSource.onerror = () => {
        setIsConnected(false)
        console.log('Jobs SSE disconnected')
        
        // Fallback to polling
        eventSource.close()
        setUsePolling(true)
      }

      eventSourceRef.current = eventSource
    } catch (error) {
      console.error('Failed to connect SSE:', error)
      setUsePolling(true)
    }
  }

  const disconnectSSE = () => {
    if (eventSourceRef.current) {
      eventSourceRef.current.close()
      eventSourceRef.current = null
      setIsConnected(false)
    }
  }

  const startPolling = () => {
    const poll = async () => {
      try {
        // Simulated polling - in real implementation, this would call GET /v1/jobs
        const response = await fetch('/v1/jobs?since=' + new Date(Date.now() - 60000).toISOString())
        const data = await response.json()
        
        if (Array.isArray(data)) {
          data.forEach(handleJobUpdate)
        }
        setIsConnected(true)
      } catch (error) {
        console.error('Polling failed:', error)
        setIsConnected(false)
      }
    }

    poll() // Initial poll
    pollingIntervalRef.current = setInterval(poll, 5000) // Poll every 5 seconds
  }

  const stopPolling = () => {
    if (pollingIntervalRef.current) {
      clearInterval(pollingIntervalRef.current)
      pollingIntervalRef.current = null
    }
  }

  const handleJobUpdate = (jobEvent: JobEvent) => {
    setJobs(prev => {
      const existingIndex = prev.findIndex(job => job.id === jobEvent.id)
      
      if (existingIndex >= 0) {
        // Update existing job
        const updated = [...prev]
        updated[existingIndex] = jobEvent
        return updated
      } else {
        // Add new job
        return [jobEvent, ...prev.slice(0, 99)] // Keep only last 100 jobs
      }
    })

    // Show notification for completed/failed jobs
    if (jobEvent.status === 'completed') {
      addToast(`${getJobTypeLabel(jobEvent.type)} completed for ${jobEvent.path || jobEvent.repo}`, 'success')
    } else if (jobEvent.status === 'failed') {
      addToast(`${getJobTypeLabel(jobEvent.type)} failed for ${jobEvent.path || jobEvent.repo}`, 'error')
    }
  }

  const getJobTypeLabel = (type: string): string => {
    switch (type) {
      case 'sampling': return 'Data Sampling'
      case 'rdf': return 'RDF Generation'
      case 'antivirus': return 'Antivirus Scan'
      case 'upload': return 'File Upload'
      case 'metadata': return 'Metadata Processing'
      default: return 'Job'
    }
  }

  const getJobIcon = (type: string): string => {
    switch (type) {
      case 'sampling': return '📊'
      case 'rdf': return '🔗'
      case 'antivirus': return '🛡️'
      case 'upload': return '📤'
      case 'metadata': return '📝'
      default: return '⚙️'
    }
  }

  const getStatusIcon = (status: string): string => {
    switch (status) {
      case 'pending': return '⏸️'
      case 'running': return '⏳'
      case 'completed': return '✅'
      case 'failed': return '❌'
      default: return '❓'
    }
  }

  const formatTimeAgo = (timestamp: string): string => {
    const now = new Date()
    const time = new Date(timestamp)
    const diffMs = now.getTime() - time.getTime()
    const diffMins = Math.floor(diffMs / 60000)
    
    if (diffMins < 1) return 'just now'
    if (diffMins < 60) return `${diffMins}m ago`
    if (diffMins < 1440) return `${Math.floor(diffMins / 60)}h ago`
    return `${Math.floor(diffMins / 1440)}d ago`
  }

  const runningJobs = jobs.filter(job => job.status === 'running')
  const recentJobs = jobs.slice(0, 10)

  // Don't render if no jobs and not expanded
  if (jobs.length === 0 && !isExpanded) {
    return null
  }

  return (
    <div className={`fixed bottom-4 right-4 z-40 ${className}`}>
      {/* Dock Header */}
      <div
        className={`bg-card border border-border rounded-lg shadow-lg transition-all duration-300 ${
          isExpanded ? 'w-80' : 'w-auto'
        }`}
      >
        <div
          className="flex items-center justify-between p-3 cursor-pointer"
          onClick={() => setIsExpanded(!isExpanded)}
        >
          <div className="flex items-center space-x-2">
            <div className={`w-2 h-2 rounded-full ${isConnected ? 'bg-green-500' : 'bg-red-500'}`} />
            <span className="font-medium text-sm">
              Jobs {runningJobs.length > 0 && `(${runningJobs.length})`}
            </span>
          </div>
          
          <div className="flex items-center space-x-2">
            {runningJobs.length > 0 && (
              <div className="flex space-x-1">
                {runningJobs.slice(0, 3).map(job => (
                  <span key={job.id} className="text-xs">
                    {getJobIcon(job.type)}
                  </span>
                ))}
              </div>
            )}
            <span className="text-xs transform transition-transform duration-200" style={{ 
              transform: isExpanded ? 'rotate(180deg)' : 'rotate(0deg)' 
            }}>
              ▼
            </span>
          </div>
        </div>

        {/* Expanded Content */}
        {isExpanded && (
          <div className="border-t border-border">
            <div className="p-3 space-y-2 max-h-96 overflow-y-auto">
              {recentJobs.length === 0 ? (
                <p className="text-xs text-muted-foreground text-center py-4">
                  No recent jobs
                </p>
              ) : (
                recentJobs.map(job => (
                  <div
                    key={job.id}
                    className={`p-2 rounded text-xs border transition-colors ${
                      job.status === 'running' 
                        ? 'bg-primary/5 border-primary/20' 
                        : 'bg-muted/50 border-border/50'
                    }`}
                  >
                    <div className="flex items-start justify-between">
                      <div className="flex items-center space-x-2 min-w-0 flex-1">
                        <span>{getJobIcon(job.type)}</span>
                        <div className="min-w-0 flex-1">
                          <p className="font-medium truncate">
                            {getJobTypeLabel(job.type)}
                          </p>
                          <p className="text-muted-foreground truncate">
                            {job.path ? `${job.repo}/${job.path}` : job.repo}
                          </p>
                        </div>
                      </div>
                      
                      <div className="flex items-center space-x-1 ml-2">
                        <span>{getStatusIcon(job.status)}</span>
                        <span className="text-muted-foreground">
                          {formatTimeAgo(job.updated_at)}
                        </span>
                      </div>
                    </div>
                    
                    {job.progress !== undefined && job.status === 'running' && (
                      <div className="mt-1">
                        <div className="w-full bg-muted rounded-full h-1">
                          <div
                            className="bg-primary h-1 rounded-full transition-all duration-300"
                            style={{ width: `${job.progress}%` }}
                          />
                        </div>
                      </div>
                    )}
                    
                    {job.message && (
                      <p className="text-muted-foreground mt-1 truncate">
                        {job.message}
                      </p>
                    )}
                    
                    {job.error && (
                      <p className="text-destructive mt-1 truncate">
                        {job.error}
                      </p>
                    )}
                  </div>
                ))
              )}
            </div>
            
            <div className="border-t border-border p-2">
              <div className="flex items-center justify-between text-xs text-muted-foreground">
                <span>
                  {enableSSE ? 'Live updates' : 'Polling every 5s'}
                </span>
                <Button
                  variant="ghost"
                  size="sm"
                  className="text-xs h-6"
                  onClick={() => setJobs([])}
                >
                  Clear
                </Button>
              </div>
            </div>
          </div>
        )}
      </div>
    </div>
  )
}

// Simulate some job events for demo purposes
export const useJobSimulation = () => {
  const [isSimulating, setIsSimulating] = useState(false)

  const simulateJob = (type: JobEvent['type'], repo: string, path?: string) => {
    const jobId = Date.now().toString() + Math.random().toString(36).substr(2)
    
    // Dispatch custom events to simulate SSE
    const dispatch = (job: JobEvent) => {
      window.dispatchEvent(new CustomEvent('job-update', { detail: job }))
    }

    const baseJob: JobEvent = {
      id: jobId,
      type,
      status: 'pending',
      repo,
      path,
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString()
    }

    // Start job
    dispatch(baseJob)

    // Running
    setTimeout(() => {
      dispatch({
        ...baseJob,
        status: 'running',
        progress: 0,
        message: 'Processing...',
        updated_at: new Date().toISOString()
      })
    }, 500)

    // Progress updates
    let progress = 0
    const progressInterval = setInterval(() => {
      progress += Math.random() * 30
      if (progress < 100) {
        dispatch({
          ...baseJob,
          status: 'running',
          progress: Math.min(progress, 99),
          message: `${Math.floor(progress)}% complete`,
          updated_at: new Date().toISOString()
        })
      } else {
        clearInterval(progressInterval)
        
        // Complete or fail randomly
        const success = Math.random() > 0.2 // 80% success rate
        dispatch({
          ...baseJob,
          status: success ? 'completed' : 'failed',
          progress: success ? 100 : undefined,
          message: success ? 'Job completed successfully' : undefined,
          error: success ? undefined : 'Job failed due to timeout',
          updated_at: new Date().toISOString()
        })
      }
    }, 200)
  }

  return { simulateJob, isSimulating, setIsSimulating }
}
