// Mobile-optimized search constants
// Week 8: Mobile/responsive UX with PWA support

// Search modes
export const SEARCH_MODES = {
  KEYWORD: 'keyword',
  SEMANTIC: 'semantic',
  HYBRID: 'hybrid'
} as const

// Search sort options
export const SEARCH_SORT_OPTIONS = [
  { value: 'relevance', label: 'Relevance', direction: 'desc' },
  { value: 'name_asc', label: 'Name A-Z', direction: 'asc' },
  { value: 'name_desc', label: 'Name Z-A', direction: 'desc' },
  { value: 'date_desc', label: 'Date Modified', direction: 'desc' },
  { value: 'size_desc', label: 'Size', direction: 'desc' }
] as const

// Search view modes
export const SEARCH_VIEW_MODES = {
  LIST: 'list',
  GRID: 'grid'
} as const

// Search result types
export const SEARCH_RESULT_TYPES = {
  FILE: 'file',
  DIRECTORY: 'directory'
} as const

// Search filter types
export const SEARCH_FILTER_TYPES = {
  TEXT: 'text',
  SELECT: 'select',
  MULTISELECT: 'multiselect',
  DATE: 'date',
  RANGE: 'range'
} as const

// Search suggestion types
export const SEARCH_SUGGESTION_TYPES = {
  RECENT: 'recent',
  TRENDING: 'trending',
  TAG: 'tag',
  FILE: 'file',
  FOLDER: 'folder'
} as const

// Search notification types
export const SEARCH_NOTIFICATION_TYPES = {
  NEW_RESULTS: 'new_results',
  QUERY_ALERT: 'query_alert',
  SYSTEM_UPDATE: 'system_update'
} as const

// Search alert types
export const SEARCH_ALERT_TYPES = {
  WARNING: 'warning',
  ERROR: 'error',
  INFO: 'info'
} as const

// Search export formats
export const SEARCH_EXPORT_FORMATS = {
  JSON: 'json',
  CSV: 'csv',
  XLSX: 'xlsx',
  PDF: 'pdf'
} as const

// Search share platforms
export const SEARCH_SHARE_PLATFORMS = {
  EMAIL: 'email',
  SOCIAL: 'social',
  LINK: 'link'
} as const

// Search performance thresholds
export const SEARCH_PERFORMANCE_THRESHOLDS = {
  RESPONSE_TIME: {
    GOOD: 100,
    WARNING: 500,
    ERROR: 1000
  },
  ERROR_RATE: {
    GOOD: 0.01,
    WARNING: 0.05,
    ERROR: 0.1
  },
  CACHE_HIT_RATE: {
    GOOD: 0.8,
    WARNING: 0.6,
    ERROR: 0.4
  }
} as const

// Search pagination
export const SEARCH_PAGINATION = {
  DEFAULT_PAGE_SIZE: 25,
  MAX_PAGE_SIZE: 100,
  MIN_PAGE_SIZE: 10
} as const

// Search debounce
export const SEARCH_DEBOUNCE = {
  QUERY: 300,
  FILTERS: 500,
  SORT: 100
} as const

// Search throttle
export const SEARCH_THROTTLE = {
  API_CALLS: 1000,
  ANALYTICS: 5000,
  CACHE_UPDATES: 2000
} as const

// Search cache
export const SEARCH_CACHE = {
  TTL: 300000, // 5 minutes
  MAX_SIZE: 100,
  CLEANUP_INTERVAL: 60000 // 1 minute
} as const

// Search history
export const SEARCH_HISTORY = {
  MAX_ITEMS: 50,
  RECENT_ITEMS: 10,
  POPULAR_ITEMS: 20
} as const

// Search suggestions
export const SEARCH_SUGGESTIONS = {
  MAX_ITEMS: 10,
  MIN_QUERY_LENGTH: 2,
  MAX_QUERY_LENGTH: 100
} as const

// Search analytics
export const SEARCH_ANALYTICS = {
  TREND_DAYS: 30,
  TOP_QUERIES_LIMIT: 10,
  METRICS_RETENTION: 90
} as const

// Search settings
export const SEARCH_SETTINGS = {
  DEFAULT_RESULTS_PER_PAGE: 25,
  DEFAULT_SEARCH_MODE: 'hybrid',
  DEFAULT_SORT: 'relevance',
  DEFAULT_VIEW_MODE: 'list'
} as const

// Search UI
export const SEARCH_UI = {
  MOBILE_BREAKPOINT: 768,
  TABLET_BREAKPOINT: 1024,
  DESKTOP_BREAKPOINT: 1280
} as const

// Search accessibility
export const SEARCH_ACCESSIBILITY = {
  ARIA_LABELS: {
    SEARCH_INPUT: 'Search for files and data',
    SEARCH_BUTTON: 'Search',
    FILTER_BUTTON: 'Filter results',
    SORT_BUTTON: 'Sort results',
    VIEW_TOGGLE: 'Toggle view mode',
    LOAD_MORE: 'Load more results',
    CLEAR_FILTERS: 'Clear all filters',
    EXPORT_RESULTS: 'Export search results',
    SHARE_RESULTS: 'Share search results',
    BOOKMARK_SEARCH: 'Bookmark this search',
    NOTIFICATION_TOGGLE: 'Toggle notifications'
  },
  KEYBOARD_SHORTCUTS: {
    SEARCH: 'Ctrl+K',
    FILTERS: 'Ctrl+F',
    SORT: 'Ctrl+S',
    VIEW_TOGGLE: 'Ctrl+V',
    CLEAR: 'Escape',
    LOAD_MORE: 'Ctrl+M',
    EXPORT: 'Ctrl+E',
    SHARE: 'Ctrl+Shift+S',
    BOOKMARK: 'Ctrl+B',
    HELP: 'F1'
  }
} as const

// Search error codes
export const SEARCH_ERROR_CODES = {
  INVALID_QUERY: 'INVALID_QUERY',
  INVALID_FILTERS: 'INVALID_FILTERS',
  INVALID_SORT: 'INVALID_SORT',
  SEARCH_FAILED: 'SEARCH_FAILED',
  LOAD_MORE_FAILED: 'LOAD_MORE_FAILED',
  EXPORT_FAILED: 'EXPORT_FAILED',
  SHARE_FAILED: 'SHARE_FAILED',
  BOOKMARK_FAILED: 'BOOKMARK_FAILED',
  NOTIFICATION_FAILED: 'NOTIFICATION_FAILED',
  ANALYTICS_FAILED: 'ANALYTICS_FAILED',
  SETTINGS_FAILED: 'SETTINGS_FAILED',
  CACHE_FAILED: 'CACHE_FAILED',
  NETWORK_ERROR: 'NETWORK_ERROR',
  TIMEOUT_ERROR: 'TIMEOUT_ERROR',
  UNAUTHORIZED: 'UNAUTHORIZED',
  FORBIDDEN: 'FORBIDDEN',
  NOT_FOUND: 'NOT_FOUND',
  INTERNAL_ERROR: 'INTERNAL_ERROR'
} as const

// Search error messages
export const SEARCH_ERROR_MESSAGES = {
  [SEARCH_ERROR_CODES.INVALID_QUERY]: 'Please enter a valid search query',
  [SEARCH_ERROR_CODES.INVALID_FILTERS]: 'Invalid filter values provided',
  [SEARCH_ERROR_CODES.INVALID_SORT]: 'Invalid sort option provided',
  [SEARCH_ERROR_CODES.SEARCH_FAILED]: 'Search failed. Please try again.',
  [SEARCH_ERROR_CODES.LOAD_MORE_FAILED]: 'Failed to load more results',
  [SEARCH_ERROR_CODES.EXPORT_FAILED]: 'Failed to export results',
  [SEARCH_ERROR_CODES.SHARE_FAILED]: 'Failed to share results',
  [SEARCH_ERROR_CODES.BOOKMARK_FAILED]: 'Failed to bookmark search',
  [SEARCH_ERROR_CODES.NOTIFICATION_FAILED]: 'Failed to send notification',
  [SEARCH_ERROR_CODES.ANALYTICS_FAILED]: 'Failed to update analytics',
  [SEARCH_ERROR_CODES.SETTINGS_FAILED]: 'Failed to update settings',
  [SEARCH_ERROR_CODES.CACHE_FAILED]: 'Failed to update cache',
  [SEARCH_ERROR_CODES.NETWORK_ERROR]: 'Network error. Please check your connection.',
  [SEARCH_ERROR_CODES.TIMEOUT_ERROR]: 'Request timed out. Please try again.',
  [SEARCH_ERROR_CODES.UNAUTHORIZED]: 'You are not authorized to perform this action',
  [SEARCH_ERROR_CODES.FORBIDDEN]: 'Access denied',
  [SEARCH_ERROR_CODES.NOT_FOUND]: 'Resource not found',
  [SEARCH_ERROR_CODES.INTERNAL_ERROR]: 'Internal server error. Please try again later.'
} as const

// Search success messages
export const SEARCH_SUCCESS_MESSAGES = {
  SEARCH_COMPLETED: 'Search completed successfully',
  RESULTS_LOADED: 'Results loaded successfully',
  FILTERS_APPLIED: 'Filters applied successfully',
  SORT_APPLIED: 'Sort applied successfully',
  VIEW_MODE_CHANGED: 'View mode changed successfully',
  EXPORT_COMPLETED: 'Export completed successfully',
  SHARE_COMPLETED: 'Share completed successfully',
  BOOKMARK_CREATED: 'Bookmark created successfully',
  BOOKMARK_DELETED: 'Bookmark deleted successfully',
  NOTIFICATION_SENT: 'Notification sent successfully',
  SETTINGS_SAVED: 'Settings saved successfully',
  CACHE_CLEARED: 'Cache cleared successfully',
  HISTORY_CLEARED: 'History cleared successfully'
} as const

// Search loading messages
export const SEARCH_LOADING_MESSAGES = {
  SEARCHING: 'Searching...',
  LOADING_MORE: 'Loading more results...',
  APPLYING_FILTERS: 'Applying filters...',
  APPLYING_SORT: 'Applying sort...',
  CHANGING_VIEW: 'Changing view mode...',
  EXPORTING: 'Exporting results...',
  SHARING: 'Sharing results...',
  CREATING_BOOKMARK: 'Creating bookmark...',
  DELETING_BOOKMARK: 'Deleting bookmark...',
  SENDING_NOTIFICATION: 'Sending notification...',
  SAVING_SETTINGS: 'Saving settings...',
  CLEARING_CACHE: 'Clearing cache...',
  CLEARING_HISTORY: 'Clearing history...'
} as const

// Search empty states
export const SEARCH_EMPTY_STATES = {
  NO_QUERY: {
    title: 'Start searching',
    description: 'Enter a search query to find files and data',
    icon: 'search'
  },
  NO_RESULTS: {
    title: 'No results found',
    description: 'Try adjusting your search terms or filters',
    icon: 'file-x'
  },
  NO_HISTORY: {
    title: 'No search history',
    description: 'Your recent searches will appear here',
    icon: 'clock'
  },
  NO_BOOKMARKS: {
    title: 'No bookmarks',
    description: 'Bookmark searches to access them quickly',
    icon: 'bookmark'
  },
  NO_NOTIFICATIONS: {
    title: 'No notifications',
    description: 'You\'ll receive notifications about your searches here',
    icon: 'bell'
  }
} as const

// Search help content
export const SEARCH_HELP_CONTENT = {
  GETTING_STARTED: {
    title: 'Getting Started',
    content: 'Start by typing your search query in the search bar. You can search for file names, content, tags, and metadata.'
  },
  FILTERS: {
    title: 'Using Filters',
    content: 'Use filters to narrow down your search results by file type, date, repository, or other criteria.'
  },
  SORTING: {
    title: 'Sorting Results',
    content: 'Sort your results by relevance, name, date, or size to find what you\'re looking for faster.'
  },
  SEMANTIC_SEARCH: {
    title: 'Semantic Search',
    content: 'Semantic search understands the meaning behind your queries and finds relevant content even if it doesn\'t contain your exact words.'
  },
  SHORTCUTS: {
    title: 'Keyboard Shortcuts',
    content: 'Use keyboard shortcuts to navigate and interact with search results more efficiently.'
  }
} as const

// Search tips
export const SEARCH_TIPS = [
  'Use specific terms for better results',
  'Try different keywords if you don\'t find what you\'re looking for',
  'Use filters to narrow down results',
  'Check your spelling',
  'Search within file contents, not just file names',
  'Use semantic search for natural language queries',
  'Bookmark useful searches for quick access',
  'Export results for offline use',
  'Share interesting findings with your team',
  'Use tags to organize and find content'
] as const

// Search best practices
export const SEARCH_BEST_PRACTICES = [
  'Use descriptive search terms',
  'Combine multiple search terms with AND/OR',
  'Use quotes for exact phrases',
  'Use wildcards (*) for partial matches',
  'Filter by file type when possible',
  'Sort by relevance for best matches',
  'Use semantic search for complex queries',
  'Bookmark frequently used searches',
  'Export results for analysis',
  'Share findings with colleagues'
] as const

export default {
  SEARCH_MODES,
  SEARCH_SORT_OPTIONS,
  SEARCH_VIEW_MODES,
  SEARCH_RESULT_TYPES,
  SEARCH_FILTER_TYPES,
  SEARCH_SUGGESTION_TYPES,
  SEARCH_NOTIFICATION_TYPES,
  SEARCH_ALERT_TYPES,
  SEARCH_EXPORT_FORMATS,
  SEARCH_SHARE_PLATFORMS,
  SEARCH_PERFORMANCE_THRESHOLDS,
  SEARCH_PAGINATION,
  SEARCH_DEBOUNCE,
  SEARCH_THROTTLE,
  SEARCH_CACHE,
  SEARCH_HISTORY,
  SEARCH_SUGGESTIONS,
  SEARCH_ANALYTICS,
  SEARCH_SETTINGS,
  SEARCH_UI,
  SEARCH_ACCESSIBILITY,
  SEARCH_ERROR_CODES,
  SEARCH_ERROR_MESSAGES,
  SEARCH_SUCCESS_MESSAGES,
  SEARCH_LOADING_MESSAGES,
  SEARCH_EMPTY_STATES,
  SEARCH_HELP_CONTENT,
  SEARCH_TIPS,
  SEARCH_BEST_PRACTICES
}
