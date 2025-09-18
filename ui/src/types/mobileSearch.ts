// Mobile-optimized search types
// Week 8: Mobile/responsive UX with PWA support

export interface SearchResult {
  id: string
  name: string
  path: string
  description?: string
  type: 'file' | 'directory'
  size?: number
  lastModified: string
  author?: string
  tags?: string[]
  repository?: string
  similarity?: number
  suggestedTags?: string[]
}

export interface SearchQuery {
  query: string
  filters?: Record<string, string[]>
  sort?: string
  semantic?: boolean
  page?: number
  limit?: number
}

export interface SearchResponse {
  results: SearchResult[]
  totalCount: number
  page: number
  limit: number
  facets?: Record<string, Array<{
    value: string
    count: number
  }>>
  suggestions?: string[]
  query?: string
  responseTime?: number
}

export interface SearchFilter {
  id: string
  label: string
  type: 'text' | 'select' | 'multiselect' | 'date' | 'range'
  options?: Array<{
    value: string
    label: string
    count?: number
  }>
  value?: any
  placeholder?: string
}

export interface SearchSort {
  id: string
  label: string
  direction: 'asc' | 'desc'
  icon?: string
}

export interface SearchSuggestion {
  id: string
  text: string
  type: 'recent' | 'trending' | 'tag' | 'file' | 'folder'
  count?: number
  lastUsed?: string
  confidence?: number
}

export interface SearchHistoryItem {
  id: string
  query: string
  timestamp: string
  resultCount: number
  filters?: string[]
  sort?: string
  success: boolean
}

export interface SearchAnalytics {
  totalSearches: number
  uniqueUsers: number
  averageResponseTime: number
  topQueries: Array<{
    query: string
    count: number
    trend: 'up' | 'down' | 'stable'
  }>
  searchTrends: Array<{
    date: string
    searches: number
    users: number
  }>
  popularFilters: Array<{
    filter: string
    count: number
    percentage: number
  }>
  searchSources: Array<{
    source: string
    count: number
    percentage: number
  }>
}

export interface SearchSettings {
  enableSemanticSearch: boolean
  enableAutoComplete: boolean
  enableSearchHistory: boolean
  enableSearchSuggestions: boolean
  enableSearchAnalytics: boolean
  defaultSearchMode: 'keyword' | 'semantic' | 'hybrid'
  resultsPerPage: number
  enableSearchFilters: boolean
  enableSearchSort: boolean
  enableSearchExport: boolean
  enableSearchSharing: boolean
  enableSearchBookmarks: boolean
  enableSearchNotifications: boolean
  enableSearchPrivacy: boolean
  enableSearchSecurity: boolean
}

export interface SearchPerformance {
  averageResponseTime: number
  p95ResponseTime: number
  p99ResponseTime: number
  throughput: number
  errorRate: number
  cacheHitRate: number
  indexSize: number
  queryComplexity: number
  memoryUsage: number
  cpuUsage: number
}

export interface SearchAlert {
  id: string
  type: 'warning' | 'error' | 'info'
  message: string
  timestamp: string
  resolved: boolean
}

export interface SearchFacet {
  field: string
  label: string
  type: 'string' | 'number' | 'date' | 'boolean'
  values: Array<{
    value: string
    count: number
    selected?: boolean
  }>
}

export interface SearchHighlight {
  field: string
  fragments: string[]
  score?: number
}

export interface SearchSpellCheck {
  query: string
  suggestions: Array<{
    suggestion: string
    frequency: number
  }>
  collation?: string
}

export interface SearchAutoComplete {
  query: string
  suggestions: Array<{
    text: string
    type: 'query' | 'field' | 'value'
    count?: number
  }>
}

export interface SearchSemantic {
  query: string
  embedding: number[]
  similarQueries: Array<{
    query: string
    similarity: number
  }>
  suggestedTags: Array<{
    tag: string
    confidence: number
    source: 'ner' | 'ml' | 'user'
  }>
}

export interface SearchExport {
  format: 'json' | 'csv' | 'xlsx' | 'pdf'
  results: SearchResult[]
  query: string
  filters?: Record<string, string[]>
  timestamp: string
  totalCount: number
}

export interface SearchShare {
  url: string
  title: string
  description: string
  results: SearchResult[]
  query: string
  filters?: Record<string, string[]>
  timestamp: string
}

export interface SearchBookmark {
  id: string
  name: string
  query: string
  filters?: Record<string, string[]>
  sort?: string
  description?: string
  tags?: string[]
  createdAt: string
  updatedAt: string
}

export interface SearchNotification {
  id: string
  type: 'new_results' | 'query_alert' | 'system_update'
  title: string
  message: string
  query?: string
  filters?: Record<string, string[]>
  timestamp: string
  read: boolean
  actionUrl?: string
}

export interface SearchContext {
  user: {
    id: string
    name: string
    email: string
    role: string
    permissions: string[]
  }
  session: {
    id: string
    startTime: string
    lastActivity: string
    ipAddress: string
    userAgent: string
  }
  environment: {
    version: string
    build: string
    environment: 'development' | 'staging' | 'production'
    features: string[]
  }
}

export interface SearchError {
  code: string
  message: string
  details?: any
  timestamp: string
  requestId?: string
}

export interface SearchMetrics {
  query: string
  responseTime: number
  resultCount: number
  filters?: Record<string, string[]>
  sort?: string
  semantic?: boolean
  timestamp: string
  userId?: string
  sessionId?: string
}

export interface SearchCache {
  key: string
  value: any
  expiresAt: string
  createdAt: string
  hits: number
  misses: number
}

export interface SearchIndex {
  name: string
  type: 'solr' | 'elasticsearch' | 'custom'
  status: 'active' | 'inactive' | 'error'
  documentCount: number
  size: number
  lastUpdated: string
  health: 'healthy' | 'warning' | 'critical'
}

export interface SearchCluster {
  name: string
  nodes: Array<{
    id: string
    host: string
    port: number
    status: 'active' | 'inactive' | 'error'
    role: 'master' | 'data' | 'coordinator'
  }>
  health: 'healthy' | 'warning' | 'critical'
  version: string
  uptime: number
}

export interface SearchBackup {
  id: string
  name: string
  type: 'full' | 'incremental'
  status: 'pending' | 'running' | 'completed' | 'failed'
  size: number
  createdAt: string
  completedAt?: string
  error?: string
}

export interface SearchRestore {
  id: string
  backupId: string
  status: 'pending' | 'running' | 'completed' | 'failed'
  createdAt: string
  completedAt?: string
  error?: string
}

export interface SearchReplication {
  id: string
  source: string
  target: string
  status: 'active' | 'inactive' | 'error'
  lag: number
  lastSync: string
  nextSync?: string
}

export interface SearchSecurity {
  enabled: boolean
  authentication: {
    type: 'none' | 'basic' | 'oauth' | 'saml'
    provider?: string
    config?: any
  }
  authorization: {
    type: 'none' | 'rbac' | 'abac'
    policies?: any[]
  }
  encryption: {
    inTransit: boolean
    atRest: boolean
    algorithm?: string
  }
  audit: {
    enabled: boolean
    level: 'none' | 'basic' | 'detailed'
    retention: number
  }
}

export interface SearchCompliance {
  enabled: boolean
  dataRetention: {
    enabled: boolean
    period: number
    unit: 'days' | 'months' | 'years'
  }
  legalHold: {
    enabled: boolean
    entries: string[]
  }
  audit: {
    enabled: boolean
    level: 'none' | 'basic' | 'detailed'
    retention: number
  }
  privacy: {
    enabled: boolean
    anonymization: boolean
    consent: boolean
  }
}

export interface SearchGovernance {
  enabled: boolean
  policies: Array<{
    id: string
    name: string
    description: string
    rules: any[]
    enforcement: 'warn' | 'block' | 'audit'
  }>
  classifications: Array<{
    id: string
    name: string
    level: number
    description: string
    color: string
  }>
  tags: Array<{
    id: string
    name: string
    description: string
    category: string
    required: boolean
  }>
}

export interface SearchQuality {
  enabled: boolean
  metrics: {
    precision: number
    recall: number
    f1Score: number
    ndcg: number
  }
  feedback: {
    enabled: boolean
    collection: 'implicit' | 'explicit' | 'both'
    retention: number
  }
  tuning: {
    enabled: boolean
    algorithm: 'manual' | 'automatic' | 'hybrid'
    frequency: 'daily' | 'weekly' | 'monthly'
  }
}

export interface SearchScalability {
  enabled: boolean
  horizontal: {
    enabled: boolean
    minNodes: number
    maxNodes: number
    scaleUpThreshold: number
    scaleDownThreshold: number
  }
  vertical: {
    enabled: boolean
    minCpu: number
    maxCpu: number
    minMemory: number
    maxMemory: number
  }
  caching: {
    enabled: boolean
    type: 'memory' | 'disk' | 'distributed'
    size: number
    ttl: number
  }
}

export interface SearchReliability {
  enabled: boolean
  redundancy: {
    enabled: boolean
    replicas: number
    shards: number
  }
  backup: {
    enabled: boolean
    frequency: 'hourly' | 'daily' | 'weekly'
    retention: number
  }
  monitoring: {
    enabled: boolean
    healthChecks: boolean
    alerts: boolean
    metrics: boolean
  }
  recovery: {
    enabled: boolean
    rto: number
    rpo: number
    strategy: 'backup' | 'replication' | 'both'
  }
}

export interface SearchIntegration {
  enabled: boolean
  apis: Array<{
    name: string
    version: string
    status: 'active' | 'deprecated' | 'disabled'
    endpoints: string[]
  }>
  webhooks: Array<{
    name: string
    url: string
    events: string[]
    status: 'active' | 'inactive' | 'error'
  }>
  connectors: Array<{
    name: string
    type: string
    status: 'active' | 'inactive' | 'error'
    config: any
  }>
  sdk: {
    enabled: boolean
    languages: string[]
    versions: Record<string, string>
  }
}

export interface SearchCustomization {
  enabled: boolean
  themes: Array<{
    id: string
    name: string
    primary: string
    secondary: string
    accent: string
    dark: boolean
  }>
  layouts: Array<{
    id: string
    name: string
    components: string[]
    responsive: boolean
  }>
  widgets: Array<{
    id: string
    name: string
    type: string
    config: any
    position: string
  }>
  branding: {
    logo?: string
    favicon?: string
    title?: string
    description?: string
  }
}

export interface SearchAccessibility {
  enabled: boolean
  standards: string[]
  features: {
    keyboardNavigation: boolean
    screenReader: boolean
    highContrast: boolean
    largeText: boolean
    colorBlind: boolean
  }
  testing: {
    enabled: boolean
    tools: string[]
    frequency: 'manual' | 'automated' | 'both'
  }
  compliance: {
    wcag: string
    section508: boolean
    ada: boolean
  }
}

export interface SearchInternationalization {
  enabled: boolean
  defaultLanguage: string
  supportedLanguages: string[]
  fallbackLanguage: string
  rtl: boolean
  dateFormat: string
  timeFormat: string
  numberFormat: string
  currency: string
  timezone: string
}

export interface SearchMobile {
  enabled: boolean
  responsive: boolean
  pwa: boolean
  touch: boolean
  gestures: boolean
  offline: boolean
  performance: {
    lazyLoading: boolean
    imageOptimization: boolean
    codeSplitting: boolean
    caching: boolean
  }
  features: {
    pushNotifications: boolean
    backgroundSync: boolean
    installPrompt: boolean
    shareTarget: boolean
  }
}

export interface SearchAnalytics {
  enabled: boolean
  tracking: {
    pageViews: boolean
    userInteractions: boolean
    searchQueries: boolean
    performance: boolean
  }
  privacy: {
    anonymize: boolean
    consent: boolean
    retention: number
  }
  providers: Array<{
    name: string
    type: string
    config: any
    status: 'active' | 'inactive' | 'error'
  }>
  dashboards: Array<{
    id: string
    name: string
    widgets: string[]
    refresh: number
  }>
}

export interface SearchMonitoring {
  enabled: boolean
  metrics: {
    system: boolean
    application: boolean
    business: boolean
    user: boolean
  }
  alerts: Array<{
    id: string
    name: string
    condition: string
    threshold: number
    action: string
    status: 'active' | 'inactive' | 'error'
  }>
  logging: {
    enabled: boolean
    level: 'debug' | 'info' | 'warn' | 'error'
    retention: number
    format: 'json' | 'text'
  }
  tracing: {
    enabled: boolean
    sampling: number
    retention: number
    exporter: string
  }
}

export interface SearchTesting {
  enabled: boolean
  unit: {
    enabled: boolean
    coverage: number
    framework: string
  }
  integration: {
    enabled: boolean
    coverage: number
    framework: string
  }
  e2e: {
    enabled: boolean
    coverage: number
    framework: string
  }
  performance: {
    enabled: boolean
    load: boolean
    stress: boolean
    volume: boolean
  }
  security: {
    enabled: boolean
    vulnerability: boolean
    penetration: boolean
    compliance: boolean
  }
}

export interface SearchDocumentation {
  enabled: boolean
  api: {
    enabled: boolean
    format: 'openapi' | 'swagger' | 'raml'
    version: string
  }
  user: {
    enabled: boolean
    format: 'markdown' | 'html' | 'pdf'
    language: string
  }
  developer: {
    enabled: boolean
    format: 'markdown' | 'html' | 'pdf'
    language: string
  }
  admin: {
    enabled: boolean
    format: 'markdown' | 'html' | 'pdf'
    language: string
  }
}

export interface SearchSupport {
  enabled: boolean
  channels: Array<{
    type: 'email' | 'chat' | 'phone' | 'ticket'
    address: string
    hours: string
    status: 'active' | 'inactive' | 'error'
  }>
  knowledge: {
    enabled: boolean
    base: string
    search: boolean
    categories: string[]
  }
  community: {
    enabled: boolean
    forum: string
    slack: string
    github: string
  }
  training: {
    enabled: boolean
    materials: string[]
    workshops: boolean
    certification: boolean
  }
}

export interface SearchFeedback {
  enabled: boolean
  collection: {
    type: 'implicit' | 'explicit' | 'both'
    frequency: 'always' | 'periodic' | 'manual'
  }
  analysis: {
    enabled: boolean
    sentiment: boolean
    categorization: boolean
    trends: boolean
  }
  response: {
    enabled: boolean
    acknowledgment: boolean
    followUp: boolean
    resolution: boolean
  }
  reporting: {
    enabled: boolean
    frequency: 'daily' | 'weekly' | 'monthly'
    format: 'email' | 'dashboard' | 'api'
  }
}

export interface SearchInnovation {
  enabled: boolean
  research: {
    enabled: boolean
    areas: string[]
    partnerships: string[]
  }
  experimentation: {
    enabled: boolean
    a_b: boolean
    multivariate: boolean
    feature: boolean
  }
  ai: {
    enabled: boolean
    ml: boolean
    nlp: boolean
    computerVision: boolean
  }
  emerging: {
    enabled: boolean
    technologies: string[]
    trends: string[]
    opportunities: string[]
  }
}

export interface SearchFuture {
  roadmap: Array<{
    id: string
    title: string
    description: string
    priority: 'low' | 'medium' | 'high' | 'critical'
    status: 'planned' | 'in_progress' | 'completed' | 'cancelled'
    targetDate: string
    dependencies: string[]
  }>
  vision: {
    shortTerm: string
    mediumTerm: string
    longTerm: string
  }
  goals: Array<{
    id: string
    title: string
    description: string
    metrics: string[]
    target: number
    deadline: string
  }>
  challenges: Array<{
    id: string
    title: string
    description: string
    impact: 'low' | 'medium' | 'high' | 'critical'
    likelihood: 'low' | 'medium' | 'high' | 'certain'
    mitigation: string
  }>
}

export default {
  SearchResult,
  SearchQuery,
  SearchResponse,
  SearchFilter,
  SearchSort,
  SearchSuggestion,
  SearchHistoryItem,
  SearchAnalytics,
  SearchSettings,
  SearchPerformance,
  SearchAlert,
  SearchFacet,
  SearchHighlight,
  SearchSpellCheck,
  SearchAutoComplete,
  SearchSemantic,
  SearchExport,
  SearchShare,
  SearchBookmark,
  SearchNotification,
  SearchContext,
  SearchError,
  SearchMetrics,
  SearchCache,
  SearchIndex,
  SearchCluster,
  SearchBackup,
  SearchRestore,
  SearchReplication,
  SearchSecurity,
  SearchCompliance,
  SearchGovernance,
  SearchQuality,
  SearchScalability,
  SearchReliability,
  SearchIntegration,
  SearchCustomization,
  SearchAccessibility,
  SearchInternationalization,
  SearchMobile,
  SearchAnalytics,
  SearchMonitoring,
  SearchTesting,
  SearchDocumentation,
  SearchSupport,
  SearchFeedback,
  SearchInnovation,
  SearchFuture
}
