// Mobile-optimized repository page
// Week 8: Mobile/responsive UX with PWA support

import React, { useState, useEffect, useCallback } from 'react'
import { useParams, useNavigate } from 'react-router-dom'
import MobileRepositoryBrowser from '@/components/mobile/MobileRepositoryBrowser'
import { RepositoryEntry } from '@/types/repository'

const MobileRepositoryPage: React.FC = () => {
  const { repoId, path = '/' } = useParams<{ repoId: string; path?: string }>()
  const navigate = useNavigate()
  const [entries, setEntries] = useState<RepositoryEntry[]>([])
  const [loading, setLoading] = useState(true)
  const [currentPath, setCurrentPath] = useState(path)

  // Load repository entries
  useEffect(() => {
    const loadEntries = async () => {
      if (!repoId) return
      
      setLoading(true)
      try {
        // Replace with actual API call
        const response = await fetch(`/api/v1/repos/${repo}/refs/${ref}/tree`, {
          method: 'GET',
          headers: {
            'Authorization': `Bearer ${localStorage.getItem('token')}`,
            'Content-Type': 'application/json'
          }
        })
        
        if (!response.ok) {
          throw new Error(`Repository API call failed: ${response.status}`)
        }
        
        const data = await response.json()
        setEntries(data.entries)
      } catch (error) {
        console.error('Failed to load repository entries:', error)
      } finally {
        setLoading(false)
      }
    }

    loadEntries()
  }, [repoId, currentPath])

  // Handle navigation
  const handleNavigate = useCallback((newPath: string) => {
    setCurrentPath(newPath)
    // Update URL
    navigate(`/repositories/${repoId}${newPath}`)
  }, [navigate, repoId])

  // Handle entry click
  const handleEntryClick = useCallback((entry: RepositoryEntry) => {
    if (entry.type === 'directory') {
      handleNavigate(entry.path)
    } else {
      // Open file or show details
      // Implement file viewer or download
      handleEntryAction(entry, 'view')
    }
  }, [handleNavigate, handleEntryAction])

  // Handle entry action
  const handleEntryAction = useCallback(async (entry: RepositoryEntry, action: string) => {
    switch (action) {
      case 'view':
        // Implement file viewer
        try {
          const response = await fetch(`/api/v1/repos/${repo}/refs/${ref}/blobs/${entry.path}/view`, {
            method: 'GET',
            headers: {
              'Authorization': `Bearer ${localStorage.getItem('token')}`,
              'Content-Type': 'application/json'
            }
          })
          
          if (!response.ok) {
            throw new Error(`File view failed: ${response.status}`)
          }
          
          const fileData = await response.json()
          // Open file viewer modal or navigate to viewer page
          console.log('File content:', fileData)
        } catch (error) {
          console.error('Failed to view file:', error)
        }
        break
      case 'download':
        // Implement download
        try {
          const response = await fetch(`/api/v1/repos/${repo}/refs/${ref}/blobs/${entry.path}/download`, {
            method: 'GET',
            headers: {
              'Authorization': `Bearer ${localStorage.getItem('token')}`,
              'Content-Type': 'application/json'
            }
          })
          
          if (!response.ok) {
            throw new Error(`Download failed: ${response.status}`)
          }
          
          const blob = await response.blob()
          const url = window.URL.createObjectURL(blob)
          const a = document.createElement('a')
          a.href = url
          a.download = entry.name
          document.body.appendChild(a)
          a.click()
          window.URL.revokeObjectURL(url)
          document.body.removeChild(a)
        } catch (error) {
          console.error('Failed to download file:', error)
        }
        break
      case 'share':
        // Implement sharing
        try {
          const response = await fetch(`/api/v1/repos/${repo}/refs/${ref}/blobs/${entry.path}/share`, {
            method: 'POST',
            headers: {
              'Authorization': `Bearer ${localStorage.getItem('token')}`,
              'Content-Type': 'application/json'
            }
          })
          
          if (!response.ok) {
            throw new Error(`Share failed: ${response.status}`)
          }
          
          const shareData = await response.json()
          await navigator.clipboard.writeText(shareData.shareUrl)
          console.log('Share URL copied to clipboard')
        } catch (error) {
          console.error('Failed to share file:', error)
        }
        break
      case 'favorite':
        // Implement favorites
        try {
          const response = await fetch(`/api/v1/repos/${repo}/refs/${ref}/blobs/${entry.path}/favorite`, {
            method: 'POST',
            headers: {
              'Authorization': `Bearer ${localStorage.getItem('token')}`,
              'Content-Type': 'application/json'
            }
          })
          
          if (!response.ok) {
            throw new Error(`Favorite failed: ${response.status}`)
          }
          
          console.log('Added to favorites:', entry)
        } catch (error) {
          console.error('Failed to add to favorites:', error)
        }
        break
      default:
        console.log('Unknown action:', action, entry)
    }
  }, [])

  if (loading) {
    return (
      <div className="min-h-screen bg-gray-50 flex items-center justify-center">
        <div className="text-center">
          <div className="w-8 h-8 border-2 border-blue-500 border-t-transparent rounded-full animate-spin mx-auto mb-4"></div>
          <p className="text-sm text-gray-600">Loading repository...</p>
        </div>
      </div>
    )
  }

  return (
    <div className="min-h-screen bg-gray-50">
      {/* Repository Header */}
      <div className="bg-white border-b border-gray-200 px-4 py-3">
        <div className="flex items-center justify-between">
          <div>
            <h1 className="text-lg font-semibold text-gray-900">
              Repository {repoId}
            </h1>
            <p className="text-sm text-gray-600">
              {currentPath === '/' ? 'Root directory' : currentPath}
            </p>
          </div>
          <div className="flex items-center space-x-2">
            <button className="p-2 text-gray-600 hover:text-gray-900 hover:bg-gray-100 rounded-md transition-colors">
              <Share className="w-5 h-5" />
            </button>
            <button className="p-2 text-gray-600 hover:text-gray-900 hover:bg-gray-100 rounded-md transition-colors">
              <Star className="w-5 h-5" />
            </button>
          </div>
        </div>
      </div>

      {/* Repository Browser */}
      <MobileRepositoryBrowser
        entries={entries}
        currentPath={currentPath}
        onNavigate={handleNavigate}
        onEntryClick={handleEntryClick}
        onEntryAction={handleEntryAction}
      />
    </div>
  )
}

export default MobileRepositoryPage
