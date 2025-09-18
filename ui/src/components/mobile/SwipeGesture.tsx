// Swipe gesture component for mobile interactions
// Week 8: Mobile/responsive UX with PWA support

import React, { useState, useRef, useEffect } from 'react'

interface SwipeGestureProps {
  children: React.ReactNode
  className?: string
  onSwipeLeft?: () => void
  onSwipeRight?: () => void
  onSwipeUp?: () => void
  onSwipeDown?: () => void
  threshold?: number
  disabled?: boolean
}

export const SwipeGesture: React.FC<SwipeGestureProps> = ({
  children,
  className = '',
  onSwipeLeft,
  onSwipeRight,
  onSwipeUp,
  onSwipeDown,
  threshold = 50,
  disabled = false
}) => {
  const [startX, setStartX] = useState(0)
  const [startY, setStartY] = useState(0)
  const [currentX, setCurrentX] = useState(0)
  const [currentY, setCurrentY] = useState(0)
  const [isDragging, setIsDragging] = useState(false)
  const elementRef = useRef<HTMLDivElement>(null)

  // Handle touch start
  const handleTouchStart = (e: React.TouchEvent) => {
    if (disabled) return

    const touch = e.touches[0]
    setStartX(touch.clientX)
    setStartY(touch.clientY)
    setCurrentX(touch.clientX)
    setCurrentY(touch.clientY)
    setIsDragging(true)
  }

  // Handle touch move
  const handleTouchMove = (e: React.TouchEvent) => {
    if (disabled || !isDragging) return

    const touch = e.touches[0]
    setCurrentX(touch.clientX)
    setCurrentY(touch.clientY)
  }

  // Handle touch end
  const handleTouchEnd = () => {
    if (disabled || !isDragging) return

    const deltaX = currentX - startX
    const deltaY = currentY - startY
    const absDeltaX = Math.abs(deltaX)
    const absDeltaY = Math.abs(deltaY)

    // Determine if it's a swipe (not just a tap)
    if (absDeltaX > threshold || absDeltaY > threshold) {
      // Determine primary direction
      if (absDeltaX > absDeltaY) {
        // Horizontal swipe
        if (deltaX > 0) {
          onSwipeRight?.()
        } else {
          onSwipeLeft?.()
        }
      } else {
        // Vertical swipe
        if (deltaY > 0) {
          onSwipeDown?.()
        } else {
          onSwipeUp?.()
        }
      }
    }

    setIsDragging(false)
    setStartX(0)
    setStartY(0)
    setCurrentX(0)
    setCurrentY(0)
  }

  // Handle touch cancel
  const handleTouchCancel = () => {
    setIsDragging(false)
    setStartX(0)
    setStartY(0)
    setCurrentX(0)
    setCurrentY(0)
  }

  // Calculate transform for visual feedback
  const deltaX = currentX - startX
  const deltaY = currentY - startY
  const transform = isDragging ? `translate(${deltaX * 0.1}px, ${deltaY * 0.1}px)` : 'translate(0, 0)'

  return (
    <div
      ref={elementRef}
      className={`relative ${className}`}
      onTouchStart={handleTouchStart}
      onTouchMove={handleTouchMove}
      onTouchEnd={handleTouchEnd}
      onTouchCancel={handleTouchCancel}
      style={{
        transform,
        transition: isDragging ? 'none' : 'transform 0.2s ease-out',
        WebkitTouchCallout: 'none',
        WebkitUserSelect: 'none',
        userSelect: 'none'
      }}
    >
      {children}
    </div>
  )
}

export default SwipeGesture
