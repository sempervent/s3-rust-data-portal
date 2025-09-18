// Mobile-optimized tab bar component
// Week 8: Mobile/responsive UX with PWA support

import React from 'react'
import { Link, useLocation } from 'react-router-dom'
import { 
  Home, 
  Search, 
  Database, 
  Settings, 
  User,
  Plus,
  Bell
} from 'lucide-react'

interface TabItem {
  id: string
  label: string
  icon: React.ComponentType<{ className?: string }>
  path: string
  badge?: number
}

interface MobileTabBarProps {
  className?: string
}

export const MobileTabBar: React.FC<MobileTabBarProps> = ({ className = '' }) => {
  const location = useLocation()

  const tabs: TabItem[] = [
    {
      id: 'home',
      label: 'Home',
      icon: Home,
      path: '/'
    },
    {
      id: 'search',
      label: 'Search',
      icon: Search,
      path: '/search'
    },
    {
      id: 'repositories',
      label: 'Repos',
      icon: Database,
      path: '/repositories'
    },
    {
      id: 'notifications',
      label: 'Alerts',
      icon: Bell,
      path: '/notifications',
      badge: 3
    },
    {
      id: 'profile',
      label: 'Profile',
      icon: User,
      path: '/profile'
    }
  ]

  const isActive = (path: string) => {
    if (path === '/') {
      return location.pathname === '/'
    }
    return location.pathname.startsWith(path)
  }

  return (
    <div className={`fixed bottom-0 left-0 right-0 bg-white border-t border-gray-200 z-40 ${className}`}>
      <div className="flex items-center justify-around py-2">
        {tabs.map((tab) => {
          const Icon = tab.icon
          const active = isActive(tab.path)
          
          return (
            <Link
              key={tab.id}
              to={tab.path}
              className={`flex flex-col items-center space-y-1 px-3 py-2 rounded-lg transition-colors ${
                active
                  ? 'text-blue-600 bg-blue-50'
                  : 'text-gray-500 hover:text-gray-700 hover:bg-gray-50'
              }`}
            >
              <div className="relative">
                <Icon className="w-5 h-5" />
                {tab.badge && tab.badge > 0 && (
                  <span className="absolute -top-2 -right-2 w-4 h-4 bg-red-500 text-white text-xs rounded-full flex items-center justify-center">
                    {tab.badge > 9 ? '9+' : tab.badge}
                  </span>
                )}
              </div>
              <span className="text-xs font-medium">{tab.label}</span>
            </Link>
          )
        })}
      </div>
    </div>
  )
}

export default MobileTabBar
