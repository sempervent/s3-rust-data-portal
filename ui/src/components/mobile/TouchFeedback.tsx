// Touch feedback component for better mobile UX
// Week 8: Mobile/responsive UX with PWA support

import React, { useState, useRef, useEffect } from 'react'

interface TouchFeedbackProps {
  children: React.ReactNode
  className?: string
  feedbackColor?: string
  feedbackOpacity?: number
  disabled?: boolean
  onClick?: () => void
  onLongPress?: () => void
  longPressDelay?: number
}

export const TouchFeedback: React.FC<TouchFeedbackProps> = ({
  children,
  className = '',
  feedbackColor = 'rgba(0, 0, 0, 0.1)',
  feedbackOpacity = 0.1,
  disabled = false,
  onClick,
  onLongPress,
  longPressDelay = 500
}) => {
  const [isPressed, setIsPressed] = useState(false)
  const [ripple, setRipple] = useState<{ x: number; y: number; id: number } | null>(null)
  const elementRef = useRef<HTMLDivElement>(null)
  const longPressTimer = useRef<NodeJS.Timeout | null>(null)
  const rippleId = useRef(0)

  // Handle touch start
  const handleTouchStart = (e: React.TouchEvent) => {
    if (disabled) return

    setIsPressed(true)
    
    // Create ripple effect
    if (elementRef.current) {
      const rect = elementRef.current.getBoundingClientRect()
      const touch = e.touches[0]
      const x = touch.clientX - rect.left
      const y = touch.clientY - rect.top
      
      setRipple({ x, y, id: rippleId.current++ })
    }

    // Start long press timer
    if (onLongPress) {
      longPressTimer.current = setTimeout(() => {
        onLongPress()
        setIsPressed(false)
        setRipple(null)
      }, longPressDelay)
    }
  }

  // Handle touch end
  const handleTouchEnd = (e: React.TouchEvent) => {
    if (disabled) return

    // Clear long press timer
    if (longPressTimer.current) {
      clearTimeout(longPressTimer.current)
      longPressTimer.current = null
    }

    // Handle click if not long press
    if (onClick && !ripple) {
      onClick()
    }

    setIsPressed(false)
    
    // Clear ripple after animation
    setTimeout(() => {
      setRipple(null)
    }, 300)
  }

  // Handle touch cancel
  const handleTouchCancel = () => {
    if (disabled) return

    if (longPressTimer.current) {
      clearTimeout(longPressTimer.current)
      longPressTimer.current = null
    }

    setIsPressed(false)
    setRipple(null)
  }

  // Handle mouse events for desktop
  const handleMouseDown = (e: React.MouseEvent) => {
    if (disabled) return

    setIsPressed(true)
    
    // Create ripple effect
    if (elementRef.current) {
      const rect = elementRef.current.getBoundingClientRect()
      const x = e.clientX - rect.left
      const y = e.clientY - rect.top
      
      setRipple({ x, y, id: rippleId.current++ })
    }
  }

  const handleMouseUp = () => {
    if (disabled) return

    setIsPressed(false)
    
    // Clear ripple after animation
    setTimeout(() => {
      setRipple(null)
    }, 300)
  }

  const handleMouseLeave = () => {
    if (disabled) return

    setIsPressed(false)
    setRipple(null)
  }

  // Cleanup on unmount
  useEffect(() => {
    return () => {
      if (longPressTimer.current) {
        clearTimeout(longPressTimer.current)
      }
    }
  }, [])

  return (
    <div
      ref={elementRef}
      className={`relative overflow-hidden ${className} ${disabled ? 'opacity-50 cursor-not-allowed' : 'cursor-pointer'}`}
      onTouchStart={handleTouchStart}
      onTouchEnd={handleTouchEnd}
      onTouchCancel={handleTouchCancel}
      onMouseDown={handleMouseDown}
      onMouseUp={handleMouseUp}
      onMouseLeave={handleMouseLeave}
      style={{
        WebkitTapHighlightColor: 'transparent',
        WebkitTouchCallout: 'none',
        WebkitUserSelect: 'none',
        userSelect: 'none'
      }}
    >
      {/* Background feedback */}
      <div
        className={`absolute inset-0 transition-opacity duration-150 ${
          isPressed ? 'opacity-100' : 'opacity-0'
        }`}
        style={{
          backgroundColor: feedbackColor,
          opacity: isPressed ? feedbackOpacity : 0
        }}
      />

      {/* Ripple effect */}
      {ripple && (
        <div
          key={ripple.id}
          className="absolute pointer-events-none animate-ping"
          style={{
            left: ripple.x - 10,
            top: ripple.y - 10,
            width: 20,
            height: 20,
            borderRadius: '50%',
            backgroundColor: feedbackColor,
            opacity: feedbackOpacity * 2
          }}
        />
      )}

      {/* Content */}
      <div className="relative z-10">
        {children}
      </div>
    </div>
  )
}

export default TouchFeedback
