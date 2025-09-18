# Mobile-Optimized Search Components

This directory contains mobile-optimized React components for the BlackLake search interface, designed for Week 8 of the project focusing on mobile/responsive UX with PWA support.

## Components Overview

### Core Components

- **MobileSearchBar** - Mobile-optimized search input with suggestions and filters
- **MobileNavigation** - Responsive navigation with mobile-first design
- **MobileJobDock** - Job status indicator optimized for mobile screens
- **MobileRepositoryBrowser** - Touch-friendly repository file browser
- **MobileSearchResults** - Mobile-optimized search results display
- **MobileLayout** - Main layout wrapper with mobile considerations

### Search Pages

- **MobileSearchPage** - Main search interface for mobile devices
- **MobileRepositoryPage** - Repository browser for mobile
- **MobileAdminConnectorsPage** - Admin connector management for mobile
- **MobileCompliancePage** - Compliance and legal hold management for mobile
- **MobileSemanticSearchPage** - AI-powered semantic search for mobile

### PWA Components

- **PWAInstallBanner** - Progressive Web App installation prompt
- **OfflineIndicator** - Network status indicator for offline support

### Interaction Components

- **TouchFeedback** - Touch feedback and haptic responses
- **SwipeGesture** - Swipe gesture recognition and handling
- **PullToRefresh** - Pull-to-refresh functionality
- **InfiniteScroll** - Infinite scrolling for search results

### UI Components

- **MobileModal** - Mobile-optimized modal dialogs
- **MobileToast** - Toast notifications for mobile
- **BottomSheet** - Bottom sheet component for mobile
- **MobileTabBar** - Tab navigation for mobile
- **FloatingActionButton** - FAB for quick actions

### Search-Specific Components

- **MobileSearchFilters** - Mobile-optimized search filters
- **MobileSearchSort** - Search result sorting for mobile
- **MobileSearchSuggestions** - Search suggestions and autocomplete
- **MobileSearchHistory** - Search history management
- **MobileSearchAnalytics** - Search analytics dashboard
- **MobileSearchPerformance** - Search performance monitoring
- **MobileSearchSettings** - Search settings and preferences
- **MobileSearchHelp** - Search help and documentation
- **MobileSearchOnboarding** - Search onboarding flow
- **MobileSearchDashboard** - Search dashboard for mobile

## Features

### Mobile-First Design
- Touch-friendly interfaces with appropriate touch targets
- Responsive layouts that adapt to different screen sizes
- Optimized for thumb navigation and one-handed use
- Gesture support for common mobile interactions

### Progressive Web App (PWA)
- Service worker for offline functionality
- App manifest for installation
- Push notifications support
- Background sync capabilities
- Install prompts and app-like experience

### Performance Optimizations
- Lazy loading of components and data
- Image optimization and responsive images
- Code splitting for faster initial load
- Efficient state management with Zustand
- Debounced search inputs to reduce API calls

### Accessibility
- Screen reader support
- Keyboard navigation
- High contrast mode support
- Focus management
- ARIA labels and descriptions

### Search Enhancements
- Semantic search with AI-powered suggestions
- Real-time search suggestions
- Advanced filtering and sorting
- Search history and bookmarks
- Export and sharing capabilities
- Analytics and performance monitoring

## Usage

### Basic Search Component

```tsx
import { MobileSearchBar } from '@/components/mobile'

function SearchPage() {
  return (
    <MobileSearchBar
      onSearch={(query) => console.log('Search:', query)}
      onToggleFilters={() => console.log('Toggle filters')}
      onToggleSort={() => console.log('Toggle sort')}
    />
  )
}
```

### Search Results

```tsx
import { MobileSearchResults } from '@/components/mobile'

function SearchResults({ results, loading, onLoadMore }) {
  return (
    <MobileSearchResults
      results={results}
      loading={loading}
      onResultClick={(result) => console.log('Result clicked:', result)}
      onLoadMore={onLoadMore}
    />
  )
}
```

### PWA Support

```tsx
import { PWAInstallBanner, OfflineIndicator } from '@/components/mobile'

function App() {
  return (
    <div>
      <OfflineIndicator />
      <PWAInstallBanner />
      {/* Your app content */}
    </div>
  )
}
```

### Touch Interactions

```tsx
import { TouchFeedback, SwipeGesture } from '@/components/mobile'

function InteractiveComponent() {
  return (
    <SwipeGesture
      onSwipeLeft={() => console.log('Swiped left')}
      onSwipeRight={() => console.log('Swiped right')}
    >
      <TouchFeedback
        onClick={() => console.log('Tapped')}
        onLongPress={() => console.log('Long pressed')}
      >
        <div>Touch me!</div>
      </TouchFeedback>
    </SwipeGesture>
  )
}
```

## State Management

The mobile search components use Zustand for state management with persistence:

```tsx
import { useMobileSearchStore } from '@/stores/mobileSearch'

function SearchComponent() {
  const { query, results, loading, search } = useMobileSearchStore()
  
  const handleSearch = (searchQuery) => {
    search(searchQuery)
  }
  
  return (
    // Your component JSX
  )
}
```

## Hooks

Custom hooks are available for common search functionality:

```tsx
import { useSearch, useSearchFilters, useSearchHistory } from '@/hooks/useMobileSearch'

function SearchPage() {
  const { query, results, loading, search } = useSearch()
  const { filters, addFilter, removeFilter } = useSearchFilters()
  const { searchHistory, addToHistory } = useSearchHistory()
  
  // Your component logic
}
```

## Styling

Components use Tailwind CSS with mobile-first responsive design:

- `sm:` - Small screens (640px+)
- `md:` - Medium screens (768px+)
- `lg:` - Large screens (1024px+)
- `xl:` - Extra large screens (1280px+)

## Testing

Components include comprehensive tests for:

- User interactions and gestures
- Responsive behavior
- Accessibility compliance
- Performance metrics
- PWA functionality

## Browser Support

- iOS Safari 12+
- Android Chrome 80+
- Samsung Internet 12+
- Edge Mobile 80+
- Firefox Mobile 80+

## Performance Targets

- First Contentful Paint: < 1.5s
- Largest Contentful Paint: < 2.5s
- Cumulative Layout Shift: < 0.1
- First Input Delay: < 100ms
- Time to Interactive: < 3.5s

## Accessibility Standards

- WCAG 2.1 AA compliance
- Section 508 compliance
- ADA compliance
- Screen reader compatibility
- Keyboard navigation support

## PWA Requirements

- Service worker registration
- App manifest configuration
- Offline functionality
- Install prompts
- Push notifications
- Background sync

## Contributing

When adding new mobile components:

1. Follow mobile-first design principles
2. Include touch-friendly interactions
3. Ensure accessibility compliance
4. Add comprehensive tests
5. Document usage examples
6. Consider PWA requirements
7. Optimize for performance

## Dependencies

- React 18+
- TypeScript 4.9+
- Tailwind CSS 3.0+
- Zustand 4.0+
- Lucide React (icons)
- React Router 6.0+

## License

This code is part of the BlackLake project and follows the project's licensing terms.
