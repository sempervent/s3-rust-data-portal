// Pull-to-refresh component for mobile
// Week 8: Mobile/responsive UX with PWA support

import React, { useState, useRef, useEffect } from 'react'
import { RefreshCw, ArrowDown } from 'lucide-react'

interface PullToRefreshProps {
  children: React.ReactNode
  onRefresh: () => Promise<void>
  className?: string
  threshold?: number
  disabled?: boolean
}

export const PullToRefresh: React.FC<PullToRefreshProps> = ({
  children,
  onRefresh,
  className = '',
  threshold = 80,
  disabled = false
}) => {
  const [isRefreshing, setIsRefreshing] = useState(false)
  const [pullDistance, setPullDistance] = useState(0)
  const [isPulling, setIsPulling] = useState(false)
  const [startY, setStartY] = useState(0)
  const [currentY, setCurrentY] = useState(0)
  const elementRef = useRef<HTMLDivElement>(null)

  // Handle touch start
  const handleTouchStart = (e: React.TouchEvent) => {
    if (disabled || isRefreshing) return

    const touch = e.touches[0]
    setStartY(touch.clientY)
    setCurrentY(touch.clientY)
  }

  // Handle touch move
  const handleTouchMove = (e: React.TouchEvent) => {
    if (disabled || isRefreshing) return

    const touch = e.touches[0]
    const deltaY = touch.clientY - startY

    // Only allow pull down when at the top
    if (deltaY > 0 && elementRef.current?.scrollTop === 0) {
      e.preventDefault()
      
      const distance = Math.min(deltaY * 0.5, threshold * 1.5) // Dampen the pull
      setPullDistance(distance)
      setIsPulling(distance > 0)
    }
  }

  // Handle touch end
  const handleTouchEnd = async () => {
    if (disabled || isRefreshing) return

    if (pullDistance > threshold) {
      setIsRefreshing(true)
      setPullDistance(0)
      setIsPulling(false)
      
      try {
        await onRefresh()
      } catch (error) {
        console.error('Refresh failed:', error)
      } finally {
        setIsRefreshing(false)
      }
    } else {
      setPullDistance(0)
      setIsPulling(false)
    }
  }

  // Calculate rotation for arrow
  const rotation = Math.min((pullDistance / threshold) * 180, 180)

  return (
    <div
      ref={elementRef}
      className={`relative overflow-auto ${className}`}
      onTouchStart={handleTouchStart}
      onTouchMove={handleTouchMove}
      onTouchEnd={handleTouchEnd}
      style={{
        WebkitOverflowScrolling: 'touch'
      }}
    >
      {/* Pull indicator */}
      <div
        className="absolute top-0 left-0 right-0 flex items-center justify-center transition-all duration-200"
        style={{
          height: Math.max(pullDistance, 0),
          transform: `translateY(${Math.max(pullDistance - 60, -60)}px)`,
          opacity: isPulling ? 1 : 0
        }}
      >
        <div className="flex flex-col items-center space-y-2">
          <div
            className="transition-transform duration-200"
            style={{
              transform: `rotate(${rotation}deg)`
            }}
          >
            {pullDistance > threshold ? (
              <RefreshCw className="w-6 h-6 text-blue-500" />
            ) : (
              <ArrowDown className="w-6 h-6 text-gray-400" />
            )}
          </div>
          <p className="text-sm text-gray-500">
            {pullDistance > threshold ? 'Release to refresh' : 'Pull to refresh'}
          </p>
        </div>
      </div>

      {/* Content */}
      <div
        className="transition-transform duration-200"
        style={{
          transform: `translateY(${Math.max(pullDistance, 0)}px)`
        }}
      >
        {children}
      </div>

      {/* Loading overlay */}
      {isRefreshing && (
        <div className="fixed top-0 left-0 right-0 bg-white bg-opacity-90 backdrop-blur-sm z-50">
          <div className="flex items-center justify-center py-4">
            <div className="flex items-center space-x-3">
              <RefreshCw className="w-5 h-5 text-blue-500 animate-spin" />
              <span className="text-sm text-gray-700">Refreshing...</span>
            </div>
          </div>
        </div>
      )}
    </div>
  )
}

export default PullToRefresh
