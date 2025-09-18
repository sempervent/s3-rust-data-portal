// Mobile-optimized layout component
// Week 8: Mobile/responsive UX with PWA support

import React, { useState, useEffect } from 'react'
import { Outlet, useLocation } from 'react-router-dom'
import MobileNavigation from './MobileNavigation'
import MobileJobDock from './MobileJobDock'
import { useJobStore } from '@/stores/jobs'

interface MobileLayoutProps {
  className?: string
}

export const MobileLayout: React.FC<MobileLayoutProps> = ({ className = '' }) => {
  const location = useLocation()
  const { jobs } = useJobStore()
  const [isOnline, setIsOnline] = useState(navigator.onLine)

  // Handle online/offline status
  useEffect(() => {
    const handleOnline = () => setIsOnline(true)
    const handleOffline = () => setIsOnline(false)

    window.addEventListener('online', handleOnline)
    window.addEventListener('offline', handleOffline)

    return () => {
      window.removeEventListener('online', handleOnline)
      window.removeEventListener('offline', handleOffline)
    }
  }, [])

  // Handle PWA install prompt
  useEffect(() => {
    let deferredPrompt: any

    const handleBeforeInstallPrompt = (e: Event) => {
      e.preventDefault()
      deferredPrompt = e
      
      // Show install button or banner
      // This would be handled by a separate component
    }

    window.addEventListener('beforeinstallprompt', handleBeforeInstallPrompt)

    return () => {
      window.removeEventListener('beforeinstallprompt', handleBeforeInstallPrompt)
    }
  }, [])

  // Handle PWA app installed
  useEffect(() => {
    const handleAppInstalled = () => {
      // Track PWA installation
      console.log('PWA was installed')
    }

    window.addEventListener('appinstalled', handleAppInstalled)

    return () => {
      window.removeEventListener('appinstalled', handleAppInstalled)
    }
  }, [])

  // Check if current route should show job dock
  const shouldShowJobDock = () => {
    const routesWithJobDock = ['/search', '/repositories', '/upload', '/admin']
    return routesWithJobDock.some(route => location.pathname.startsWith(route))
  }

  return (
    <div className={`min-h-screen bg-gray-50 ${className}`}>
      {/* Offline Indicator */}
      {!isOnline && (
        <div className="bg-yellow-100 border-b border-yellow-200 px-4 py-2">
          <div className="flex items-center justify-center space-x-2 text-sm text-yellow-800">
            <div className="w-2 h-2 bg-yellow-500 rounded-full"></div>
            <span>You're offline. Some features may be limited.</span>
          </div>
        </div>
      )}

      {/* Navigation */}
      <MobileNavigation />

      {/* Main Content */}
      <main className="pb-20"> {/* Bottom padding for job dock */}
        <Outlet />
      </main>

      {/* Job Dock */}
      {shouldShowJobDock() && <MobileJobDock />}

      {/* PWA Install Banner */}
      {/* This would be a separate component that shows when PWA can be installed */}
      
      {/* Touch Feedback */}
      <style jsx>{`
        /* Add touch feedback for better mobile experience */
        button, a, [role="button"] {
          -webkit-tap-highlight-color: rgba(0, 0, 0, 0.1);
        }
        
        /* Prevent zoom on input focus */
        input, textarea, select {
          font-size: 16px;
        }
        
        /* Smooth scrolling */
        html {
          scroll-behavior: smooth;
        }
        
        /* Better text rendering on mobile */
        body {
          -webkit-font-smoothing: antialiased;
          -moz-osx-font-smoothing: grayscale;
        }
      `}</style>
    </div>
  )
}

export default MobileLayout
