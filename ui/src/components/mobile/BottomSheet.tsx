// Mobile-optimized bottom sheet component
// Week 8: Mobile/responsive UX with PWA support

import React, { useState, useEffect, useRef } from 'react'
import { X, ChevronDown } from 'lucide-react'

interface BottomSheetProps {
  isOpen: boolean
  onClose: () => void
  title?: string
  children: React.ReactNode
  className?: string
  showCloseButton?: boolean
  snapPoints?: number[]
  defaultSnapPoint?: number
}

export const BottomSheet: React.FC<BottomSheetProps> = ({
  isOpen,
  onClose,
  title,
  children,
  className = '',
  showCloseButton = true,
  snapPoints = [0.25, 0.5, 0.75, 1],
  defaultSnapPoint = 0.5
}) => {
  const [currentSnapPoint, setCurrentSnapPoint] = useState(defaultSnapPoint)
  const [isDragging, setIsDragging] = useState(false)
  const [startY, setStartY] = useState(0)
  const [currentY, setCurrentY] = useState(0)
  const [translateY, setTranslateY] = useState(0)
  const sheetRef = useRef<HTMLDivElement>(null)

  // Handle touch start
  const handleTouchStart = (e: React.TouchEvent) => {
    const touch = e.touches[0]
    setStartY(touch.clientY)
    setCurrentY(touch.clientY)
    setIsDragging(true)
  }

  // Handle touch move
  const handleTouchMove = (e: React.TouchEvent) => {
    if (!isDragging) return

    const touch = e.touches[0]
    const deltaY = touch.clientY - startY
    const newTranslateY = Math.max(0, deltaY)
    setTranslateY(newTranslateY)
  }

  // Handle touch end
  const handleTouchEnd = () => {
    if (!isDragging) return

    setIsDragging(false)
    
    // Determine snap point based on drag distance
    const dragDistance = translateY
    const sheetHeight = sheetRef.current?.offsetHeight || 0
    const dragPercentage = dragDistance / sheetHeight

    // Find closest snap point
    let closestSnapPoint = snapPoints[0]
    let minDistance = Math.abs(dragPercentage - snapPoints[0])

    for (const snapPoint of snapPoints) {
      const distance = Math.abs(dragPercentage - snapPoint)
      if (distance < minDistance) {
        minDistance = distance
        closestSnapPoint = snapPoint
      }
    }

    // If dragged down significantly, close the sheet
    if (dragPercentage > 0.3) {
      onClose()
    } else {
      setCurrentSnapPoint(closestSnapPoint)
      setTranslateY(0)
    }
  }

  // Handle backdrop click
  const handleBackdropClick = (e: React.MouseEvent) => {
    if (e.target === e.currentTarget) {
      onClose()
    }
  }

  // Prevent body scroll when sheet is open
  useEffect(() => {
    if (isOpen) {
      document.body.style.overflow = 'hidden'
    } else {
      document.body.style.overflow = 'unset'
    }

    return () => {
      document.body.style.overflow = 'unset'
    }
  }, [isOpen])

  // Calculate sheet height based on snap point
  const sheetHeight = `${currentSnapPoint * 100}%`

  if (!isOpen) return null

  return (
    <div className="fixed inset-0 z-50 flex items-end">
      {/* Backdrop */}
      <div
        className="absolute inset-0 bg-black bg-opacity-50 transition-opacity duration-300"
        onClick={handleBackdropClick}
      />

      {/* Bottom Sheet */}
      <div
        ref={sheetRef}
        className={`relative w-full bg-white rounded-t-lg shadow-xl transform transition-all duration-300 ${className}`}
        style={{
          height: sheetHeight,
          transform: `translateY(${translateY}px)`,
          transition: isDragging ? 'none' : 'transform 0.3s ease-out'
        }}
        onTouchStart={handleTouchStart}
        onTouchMove={handleTouchMove}
        onTouchEnd={handleTouchEnd}
      >
        {/* Handle */}
        <div className="flex justify-center pt-3 pb-2">
          <div className="w-12 h-1 bg-gray-300 rounded-full" />
        </div>

        {/* Header */}
        {(title || showCloseButton) && (
          <div className="flex items-center justify-between px-4 py-3 border-b border-gray-200">
            {title && (
              <h3 className="text-lg font-semibold text-gray-900">
                {title}
              </h3>
            )}
            {showCloseButton && (
              <button
                onClick={onClose}
                className="p-2 text-gray-400 hover:text-gray-600 hover:bg-gray-100 rounded-md transition-colors"
              >
                <X className="w-5 h-5" />
              </button>
            )}
          </div>
        )}

        {/* Content */}
        <div className="flex-1 overflow-y-auto">
          {children}
        </div>
      </div>
    </div>
  )
}

export default BottomSheet
