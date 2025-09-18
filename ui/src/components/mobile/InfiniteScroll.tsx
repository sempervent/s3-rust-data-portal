// Infinite scroll component for mobile
// Week 8: Mobile/responsive UX with PWA support

import React, { useState, useRef, useEffect, useCallback } from 'react'
import { Loader2 } from 'lucide-react'

interface InfiniteScrollProps {
  children: React.ReactNode
  onLoadMore: () => Promise<void>
  hasMore: boolean
  loading: boolean
  className?: string
  threshold?: number
  rootMargin?: string
}

export const InfiniteScroll: React.FC<InfiniteScrollProps> = ({
  children,
  onLoadMore,
  hasMore,
  loading,
  className = '',
  threshold = 0.1,
  rootMargin = '100px'
}) => {
  const [isIntersecting, setIsIntersecting] = useState(false)
  const sentinelRef = useRef<HTMLDivElement>(null)
  const observerRef = useRef<IntersectionObserver | null>(null)

  // Create intersection observer
  useEffect(() => {
    if (!sentinelRef.current) return

    observerRef.current = new IntersectionObserver(
      (entries) => {
        const [entry] = entries
        setIsIntersecting(entry.isIntersecting)
      },
      {
        threshold,
        rootMargin
      }
    )

    observerRef.current.observe(sentinelRef.current)

    return () => {
      if (observerRef.current) {
        observerRef.current.disconnect()
      }
    }
  }, [threshold, rootMargin])

  // Handle load more when intersecting
  useEffect(() => {
    if (isIntersecting && hasMore && !loading) {
      onLoadMore()
    }
  }, [isIntersecting, hasMore, loading, onLoadMore])

  return (
    <div className={className}>
      {children}
      
      {/* Sentinel element */}
      <div ref={sentinelRef} className="h-1" />
      
      {/* Loading indicator */}
      {loading && (
        <div className="flex items-center justify-center py-8">
          <div className="flex items-center space-x-3">
            <Loader2 className="w-5 h-5 text-blue-500 animate-spin" />
            <span className="text-sm text-gray-600">Loading more...</span>
          </div>
        </div>
      )}
      
      {/* End of content indicator */}
      {!hasMore && !loading && (
        <div className="flex items-center justify-center py-8">
          <div className="text-sm text-gray-500">
            You've reached the end
          </div>
        </div>
      )}
    </div>
  )
}

export default InfiniteScroll
