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
        // TODO: Replace with actual API call
        const mockEntries: RepositoryEntry[] = [
          {
            id: '1',
            path: '/datasets',
            name: 'datasets',
            type: 'directory',
            size: 0,
            lastModified: new Date().toISOString(),
            author: 'John Doe',
            tags: ['data', 'datasets']
          },
          {
            id: '2',
            path: '/documents',
            name: 'documents',
            type: 'directory',
            size: 0,
            lastModified: new Date().toISOString(),
            author: 'Jane Smith',
            tags: ['docs', 'documentation']
          },
          {
            id: '3',
            path: '/sales_data.csv',
            name: 'sales_data.csv',
            type: 'file',
            size: 1024000,
            lastModified: new Date().toISOString(),
            author: 'John Doe',
            tags: ['csv', 'sales', 'data']
          },
          {
            id: '4',
            path: '/customer_analytics.json',
            name: 'customer_analytics.json',
            type: 'file',
            size: 512000,
            lastModified: new Date().toISOString(),
            author: 'Jane Smith',
            tags: ['json', 'analytics', 'customers']
          }
        ]
        
        setEntries(mockEntries)
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
      // TODO: Implement file viewer or download
      console.log('Opening file:', entry)
    }
  }, [handleNavigate])

  // Handle entry action
  const handleEntryAction = useCallback((entry: RepositoryEntry, action: string) => {
    switch (action) {
      case 'view':
        // TODO: Implement file viewer
        console.log('Viewing:', entry)
        break
      case 'download':
        // TODO: Implement download
        console.log('Downloading:', entry)
        break
      case 'share':
        // TODO: Implement sharing
        console.log('Sharing:', entry)
        break
      case 'favorite':
        // TODO: Implement favorites
        console.log('Adding to favorites:', entry)
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
