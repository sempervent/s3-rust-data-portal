// Offline indicator component
// Week 8: Mobile/responsive UX with PWA support

import React, { useState, useEffect } from 'react'
import { Wifi, WifiOff, RefreshCw } from 'lucide-react'

interface OfflineIndicatorProps {
  className?: string
}

export const OfflineIndicator: React.FC<OfflineIndicatorProps> = ({ className = '' }) => {
  const [isOnline, setIsOnline] = useState(navigator.onLine)
  const [showReconnect, setShowReconnect] = useState(false)
  const [reconnectAttempts, setReconnectAttempts] = useState(0)

  useEffect(() => {
    const handleOnline = () => {
      setIsOnline(true)
      setShowReconnect(false)
      setReconnectAttempts(0)
    }

    const handleOffline = () => {
      setIsOnline(false)
      setShowReconnect(true)
    }

    window.addEventListener('online', handleOnline)
    window.addEventListener('offline', handleOffline)

    return () => {
      window.removeEventListener('online', handleOnline)
      window.removeEventListener('offline', handleOffline)
    }
  }, [])

  // Handle reconnect attempt
  const handleReconnect = async () => {
    setReconnectAttempts(prev => prev + 1)
    
    try {
      // Try to fetch a small resource to test connectivity
      const response = await fetch('/api/health', { 
        method: 'HEAD',
        cache: 'no-cache'
      })
      
      if (response.ok) {
        setIsOnline(true)
        setShowReconnect(false)
        setReconnectAttempts(0)
      }
    } catch (error) {
      console.error('Reconnect attempt failed:', error)
    }
  }

  // Don't show indicator if online
  if (isOnline) {
    return null
  }

  return (
    <div className={`fixed top-0 left-0 right-0 bg-yellow-100 border-b border-yellow-200 z-50 ${className}`}>
      <div className="px-4 py-3">
        <div className="flex items-center justify-between">
          <div className="flex items-center space-x-3">
            <WifiOff className="w-5 h-5 text-yellow-600" />
            <div>
              <p className="text-sm font-medium text-yellow-800">
                You're offline
              </p>
              <p className="text-xs text-yellow-700">
                Some features may be limited. Check your connection.
              </p>
            </div>
          </div>
          
          {showReconnect && (
            <button
              onClick={handleReconnect}
              disabled={reconnectAttempts >= 3}
              className="flex items-center space-x-2 px-3 py-1 bg-yellow-200 text-yellow-800 text-sm font-medium rounded-md hover:bg-yellow-300 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
            >
              <RefreshCw className={`w-4 h-4 ${reconnectAttempts > 0 ? 'animate-spin' : ''}`} />
              <span>
                {reconnectAttempts >= 3 ? 'Failed' : 'Retry'}
              </span>
            </button>
          )}
        </div>
      </div>
    </div>
  )
}

export default OfflineIndicator
