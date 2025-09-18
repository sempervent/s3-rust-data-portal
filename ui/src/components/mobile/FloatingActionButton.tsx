// Mobile-optimized floating action button component
// Week 8: Mobile/responsive UX with PWA support

import React, { useState, useRef, useEffect } from 'react'
import { Plus, X, Upload, FolderPlus, FileText, Camera } from 'lucide-react'

interface FloatingActionButtonProps {
  className?: string
  onUpload?: () => void
  onNewFolder?: () => void
  onNewFile?: () => void
  onScan?: () => void
}

export const FloatingActionButton: React.FC<FloatingActionButtonProps> = ({
  className = '',
  onUpload,
  onNewFolder,
  onNewFile,
  onScan
}) => {
  const [isOpen, setIsOpen] = useState(false)
  const [isVisible, setIsVisible] = useState(true)
  const fabRef = useRef<HTMLDivElement>(null)
  const lastScrollY = useRef(0)

  // Handle scroll to hide/show FAB
  useEffect(() => {
    const handleScroll = () => {
      const currentScrollY = window.scrollY
      
      if (currentScrollY > lastScrollY.current && currentScrollY > 100) {
        // Scrolling down
        setIsVisible(false)
      } else {
        // Scrolling up
        setIsVisible(true)
      }
      
      lastScrollY.current = currentScrollY
    }

    window.addEventListener('scroll', handleScroll, { passive: true })
    return () => window.removeEventListener('scroll', handleScroll)
  }, [])

  // Handle click outside to close
  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (fabRef.current && !fabRef.current.contains(event.target as Node)) {
        setIsOpen(false)
      }
    }

    if (isOpen) {
      document.addEventListener('click', handleClickOutside)
      return () => document.removeEventListener('click', handleClickOutside)
    }
  }, [isOpen])

  // Handle main button click
  const handleMainClick = () => {
    setIsOpen(!isOpen)
  }

  // Handle action click
  const handleActionClick = (action: () => void) => {
    action()
    setIsOpen(false)
  }

  if (!isVisible) return null

  return (
    <div
      ref={fabRef}
      className={`fixed bottom-20 right-4 z-50 ${className}`}
    >
      {/* Action Buttons */}
      {isOpen && (
        <div className="absolute bottom-16 right-0 space-y-3">
          {onUpload && (
            <button
              onClick={() => handleActionClick(onUpload)}
              className="flex items-center space-x-3 bg-white text-gray-700 px-4 py-3 rounded-full shadow-lg hover:bg-gray-50 transition-colors"
            >
              <Upload className="w-5 h-5" />
              <span className="text-sm font-medium">Upload</span>
            </button>
          )}
          
          {onNewFolder && (
            <button
              onClick={() => handleActionClick(onNewFolder)}
              className="flex items-center space-x-3 bg-white text-gray-700 px-4 py-3 rounded-full shadow-lg hover:bg-gray-50 transition-colors"
            >
              <FolderPlus className="w-5 h-5" />
              <span className="text-sm font-medium">New Folder</span>
            </button>
          )}
          
          {onNewFile && (
            <button
              onClick={() => handleActionClick(onNewFile)}
              className="flex items-center space-x-3 bg-white text-gray-700 px-4 py-3 rounded-full shadow-lg hover:bg-gray-50 transition-colors"
            >
              <FileText className="w-5 h-5" />
              <span className="text-sm font-medium">New File</span>
            </button>
          )}
          
          {onScan && (
            <button
              onClick={() => handleActionClick(onScan)}
              className="flex items-center space-x-3 bg-white text-gray-700 px-4 py-3 rounded-full shadow-lg hover:bg-gray-50 transition-colors"
            >
              <Camera className="w-5 h-5" />
              <span className="text-sm font-medium">Scan</span>
            </button>
          )}
        </div>
      )}

      {/* Main FAB */}
      <button
        onClick={handleMainClick}
        className={`w-14 h-14 bg-blue-600 text-white rounded-full shadow-lg hover:bg-blue-700 transition-all duration-300 flex items-center justify-center ${
          isOpen ? 'rotate-45' : 'rotate-0'
        }`}
      >
        {isOpen ? (
          <X className="w-6 h-6" />
        ) : (
          <Plus className="w-6 h-6" />
        )}
      </button>
    </div>
  )
}

export default FloatingActionButton
