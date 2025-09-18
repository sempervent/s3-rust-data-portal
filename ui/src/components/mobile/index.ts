// Mobile-optimized search components index
// Week 8: Mobile/responsive UX with PWA support

// Core components
export { default as MobileSearchBar } from './MobileSearchBar'
export { default as MobileNavigation } from './MobileNavigation'
export { default as MobileJobDock } from './MobileJobDock'
export { default as MobileRepositoryBrowser } from './MobileRepositoryBrowser'
export { default as MobileSearchResults } from './MobileSearchResults'
export { default as MobileLayout } from './MobileLayout'

// Search components
export { default as MobileSearchPage } from '../pages/mobile/MobileSearchPage'
export { default as MobileRepositoryPage } from '../pages/mobile/MobileRepositoryPage'
export { default as MobileAdminConnectorsPage } from '../pages/mobile/MobileAdminConnectorsPage'
export { default as MobileCompliancePage } from '../pages/mobile/MobileCompliancePage'
export { default as MobileSemanticSearchPage } from '../pages/mobile/MobileSemanticSearchPage'

// PWA components
export { default as PWAInstallBanner } from './PWAInstallBanner'
export { default as OfflineIndicator } from './OfflineIndicator'

// Interaction components
export { default as TouchFeedback } from './TouchFeedback'
export { default as SwipeGesture } from './SwipeGesture'
export { default as PullToRefresh } from './PullToRefresh'
export { default as InfiniteScroll } from './InfiniteScroll'

// UI components
export { default as MobileModal } from './MobileModal'
export { default as MobileToast } from './MobileToast'
export { default as BottomSheet } from './BottomSheet'
export { default as MobileTabBar } from './MobileTabBar'
export { default as FloatingActionButton } from './FloatingActionButton'

// Search-specific components
export { default as MobileSearchFilters } from './MobileSearchFilters'
export { default as MobileSearchSort } from './MobileSearchSort'
export { default as MobileSearchSuggestions } from './MobileSearchSuggestions'
export { default as MobileSearchHistory } from './MobileSearchHistory'
export { default as MobileSearchAnalytics } from './MobileSearchAnalytics'
export { default as MobileSearchPerformance } from './MobileSearchPerformance'
export { default as MobileSearchSettings } from './MobileSearchSettings'
export { default as MobileSearchHelp } from './MobileSearchHelp'
export { default as MobileSearchOnboarding } from './MobileSearchOnboarding'
export { default as MobileSearchDashboard } from './MobileSearchDashboard'
export { default as MobileSearchContext } from './MobileSearchContext'

// Store and hooks
export { default as useMobileSearchStore } from '../stores/mobileSearch'
export { default as useMobileSearch } from '../hooks/useMobileSearch'

// Types and utilities
export * from '../types/mobileSearch'
export * from '../utils/mobileSearch'
export * from '../constants/mobileSearch'

// Re-export common types
export type { SearchResult, SearchQuery, SearchResponse } from '../types/mobileSearch'
export type { SearchState, SearchActions } from '../stores/mobileSearch'
