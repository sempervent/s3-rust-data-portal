// PWA install banner component
// Week 8: Mobile/responsive UX with PWA support

import React, { useState, useEffect } from 'react'
import { Download, X, Smartphone, Monitor } from 'lucide-react'

interface PWAInstallBannerProps {
  className?: string
}

export const PWAInstallBanner: React.FC<PWAInstallBannerProps> = ({ className = '' }) => {
  const [deferredPrompt, setDeferredPrompt] = useState<any>(null)
  const [showBanner, setShowBanner] = useState(false)
  const [isInstalled, setIsInstalled] = useState(false)

  useEffect(() => {
    // Check if PWA is already installed
    const checkIfInstalled = () => {
      // Check if running in standalone mode (PWA)
      if (window.matchMedia('(display-mode: standalone)').matches) {
        setIsInstalled(true)
        return
      }

      // Check if running in fullscreen mode
      if (window.matchMedia('(display-mode: fullscreen)').matches) {
        setIsInstalled(true)
        return
      }

      // Check if running in minimal-ui mode
      if (window.matchMedia('(display-mode: minimal-ui)').matches) {
        setIsInstalled(true)
        return
      }
    }

    checkIfInstalled()

    // Listen for beforeinstallprompt event
    const handleBeforeInstallPrompt = (e: Event) => {
      e.preventDefault()
      setDeferredPrompt(e)
      setShowBanner(true)
    }

    // Listen for appinstalled event
    const handleAppInstalled = () => {
      setIsInstalled(true)
      setShowBanner(false)
      setDeferredPrompt(null)
      
      // Track PWA installation
      console.log('PWA was installed')
      
      // Show success message
      // TODO: Implement toast notification
    }

    window.addEventListener('beforeinstallprompt', handleBeforeInstallPrompt)
    window.addEventListener('appinstalled', handleAppInstalled)

    return () => {
      window.removeEventListener('beforeinstallprompt', handleBeforeInstallPrompt)
      window.removeEventListener('appinstalled', handleAppInstalled)
    }
  }, [])

  // Handle install button click
  const handleInstall = async () => {
    if (!deferredPrompt) return

    try {
      // Show the install prompt
      deferredPrompt.prompt()
      
      // Wait for the user to respond to the prompt
      const { outcome } = await deferredPrompt.userChoice
      
      if (outcome === 'accepted') {
        console.log('User accepted the install prompt')
      } else {
        console.log('User dismissed the install prompt')
      }
      
      // Clear the deferred prompt
      setDeferredPrompt(null)
      setShowBanner(false)
    } catch (error) {
      console.error('Error showing install prompt:', error)
    }
  }

  // Handle dismiss
  const handleDismiss = () => {
    setShowBanner(false)
    // Store dismissal in localStorage to avoid showing again immediately
    localStorage.setItem('pwa-banner-dismissed', Date.now().toString())
  }

  // Check if banner was recently dismissed
  useEffect(() => {
    const dismissed = localStorage.getItem('pwa-banner-dismissed')
    if (dismissed) {
      const dismissedTime = parseInt(dismissed)
      const now = Date.now()
      const daysSinceDismissed = (now - dismissedTime) / (1000 * 60 * 60 * 24)
      
      // Don't show banner if dismissed within last 7 days
      if (daysSinceDismissed < 7) {
        setShowBanner(false)
      }
    }
  }, [])

  // Don't show banner if PWA is already installed
  if (isInstalled || !showBanner || !deferredPrompt) {
    return null
  }

  return (
    <div className={`fixed bottom-0 left-0 right-0 bg-white border-t border-gray-200 shadow-lg z-40 ${className}`}>
      <div className="p-4">
        <div className="flex items-start space-x-3">
          {/* Icon */}
          <div className="flex-shrink-0">
            <div className="w-10 h-10 bg-blue-100 rounded-lg flex items-center justify-center">
              <Download className="w-5 h-5 text-blue-600" />
            </div>
          </div>

          {/* Content */}
          <div className="flex-1 min-w-0">
            <h3 className="text-sm font-semibold text-gray-900">
              Install BlackLake
            </h3>
            <p className="text-xs text-gray-600 mt-1">
              Get quick access to your data portal with our mobile app
            </p>
            
            {/* Benefits */}
            <div className="mt-2 flex items-center space-x-4 text-xs text-gray-500">
              <div className="flex items-center space-x-1">
                <Smartphone className="w-3 h-3" />
                <span>Mobile access</span>
              </div>
              <div className="flex items-center space-x-1">
                <Monitor className="w-3 h-3" />
                <span>Offline support</span>
              </div>
            </div>
          </div>

          {/* Actions */}
          <div className="flex items-center space-x-2">
            <button
              onClick={handleDismiss}
              className="p-1 text-gray-400 hover:text-gray-600 transition-colors"
            >
              <X className="w-4 h-4" />
            </button>
            <button
              onClick={handleInstall}
              className="px-3 py-2 bg-blue-600 text-white text-sm font-medium rounded-md hover:bg-blue-700 transition-colors"
            >
              Install
            </button>
          </div>
        </div>
      </div>
    </div>
  )
}

export default PWAInstallBanner
