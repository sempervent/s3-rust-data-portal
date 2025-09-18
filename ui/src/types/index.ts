// Re-export API types
export type {
  Repo,
  Branch,
  TreeEntry,
  TreeResponse,
  Commit,
  SearchResult,
  SearchResponse,
  UploadInitRequest,
  UploadInitResponse,
  CommitRequest,
  CommitResponse,
  JobEvent,
  DiffResponse,
  LineageResponse,
  ApiError,
} from '@/lib/api';

// UI-specific types
export interface User {
  id: string;
  email: string;
  name?: string;
  picture?: string;
}

export interface BreadcrumbItem {
  label: string;
  href?: string;
  current?: boolean;
}

export interface Column {
  id: string;
  label: string;
  sortable?: boolean;
  filterable?: boolean;
  width?: number;
  minWidth?: number;
  maxWidth?: number;
  align?: 'left' | 'center' | 'right';
  render?: (value: any, row: any) => React.ReactNode;
}

export interface Filter {
  id: string;
  label: string;
  type: 'text' | 'select' | 'multiselect' | 'date' | 'daterange' | 'number' | 'numberrange';
  options?: Array<{ label: string; value: any }>;
  placeholder?: string;
  multiple?: boolean;
}

export interface SortConfig {
  column: string;
  direction: 'asc' | 'desc';
}

export interface PaginationConfig {
  page: number;
  perPage: number;
  total: number;
  hasNext: boolean;
  hasPrev: boolean;
}

export interface TableConfig {
  columns: Column[];
  filters: Filter[];
  sort?: SortConfig;
  pagination?: PaginationConfig;
  selectable?: boolean;
  multiSelect?: boolean;
  virtualized?: boolean;
}

export interface FileUpload {
  file: File;
  path: string;
  metadata?: Record<string, any>;
  status: 'pending' | 'uploading' | 'completed' | 'error';
  progress?: number;
  error?: string;
  uploadUrl?: string;
  sha256?: string;
}

export interface UploadBatch {
  id: string;
  files: FileUpload[];
  metadata: Record<string, any>;
  status: 'pending' | 'uploading' | 'completed' | 'error';
  progress?: number;
  error?: string;
  commitId?: string;
}

export interface ModelInfo {
  framework: 'ONNX' | 'PyTorch' | 'TensorFlow' | 'Unknown';
  version?: string;
  opset?: number;
  inputShapes?: Array<{
    name: string;
    shape: number[];
    type: string;
  }>;
  outputShapes?: Array<{
    name: string;
    shape: number[];
    type: string;
  }>;
  size: number;
  sha256: string;
}

export interface PreviewData {
  type: 'text' | 'json' | 'csv' | 'parquet' | 'image' | 'model' | 'binary';
  content?: any;
  metadata?: Record<string, any>;
  error?: string;
}

export interface LineageNode {
  id: string;
  path: string;
  type: 'blob' | 'tree';
  metadata?: Record<string, any>;
  position?: { x: number; y: number };
}

export interface LineageEdge {
  id: string;
  source: string;
  target: string;
  type: 'derived' | 'parent' | 'reference';
  label?: string;
}

export interface LineageGraph {
  nodes: LineageNode[];
  edges: LineageEdge[];
}

export interface SavedView {
  id: string;
  name: string;
  filters: Record<string, any>;
  columns: string[];
  sort?: SortConfig;
  createdAt: string;
  updatedAt?: string;
}

export interface Notification {
  id: string;
  type: 'success' | 'error' | 'warning' | 'info';
  title: string;
  message?: string;
  duration?: number;
  action?: {
    label: string;
    onClick: () => void;
  };
  timestamp: string;
}

export interface Theme {
  name: 'light' | 'dark' | 'system';
  colors: {
    primary: string;
    secondary: string;
    background: string;
    foreground: string;
    muted: string;
    accent: string;
    destructive: string;
    border: string;
    input: string;
    ring: string;
  };
}

export interface AppSettings {
  theme: Theme;
  language: string;
  timezone: string;
  dateFormat: string;
  itemsPerPage: number;
  autoRefresh: boolean;
  refreshInterval: number;
  notifications: {
    enabled: boolean;
    sound: boolean;
    desktop: boolean;
  };
  shortcuts: Record<string, string>;
}

export interface SearchFilters {
  query?: string;
  fileType?: string;
  org?: string;
  tags?: string[];
  dateRange?: {
    from: string;
    to: string;
  };
  sizeRange?: {
    min: number;
    max: number;
  };
  sortBy?: string;
  sortOrder?: 'asc' | 'desc';
  page?: number;
  perPage?: number;
}

export interface TreeItem {
  id: string;
  name: string;
  path: string;
  type: 'blob' | 'tree';
  size?: number;
  children?: TreeItem[];
  expanded?: boolean;
  selected?: boolean;
  metadata?: Record<string, any>;
}

export interface TabItem {
  id: string;
  label: string;
  content: React.ReactNode;
  icon?: React.ReactNode;
  closable?: boolean;
  disabled?: boolean;
}

export interface ModalProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  title?: string;
  description?: string;
  children: React.ReactNode;
  size?: 'sm' | 'md' | 'lg' | 'xl' | 'full';
  closable?: boolean;
}

export interface DrawerProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  title?: string;
  children: React.ReactNode;
  side?: 'left' | 'right' | 'top' | 'bottom';
  size?: 'sm' | 'md' | 'lg' | 'xl' | 'full';
  closable?: boolean;
}

export interface SplitPaneProps {
  children: [React.ReactNode, React.ReactNode];
  defaultSize?: number;
  minSize?: number;
  maxSize?: number;
  split?: 'vertical' | 'horizontal';
  resizable?: boolean;
  className?: string;
}

export interface ProgressProps {
  value: number;
  max?: number;
  label?: string;
  showValue?: boolean;
  size?: 'sm' | 'md' | 'lg';
  variant?: 'default' | 'success' | 'warning' | 'error';
  className?: string;
}

export interface CodeViewerProps {
  code: string;
  language?: string;
  theme?: 'light' | 'dark';
  readOnly?: boolean;
  showLineNumbers?: boolean;
  wrapLines?: boolean;
  maxHeight?: string;
  className?: string;
}

export interface JsonSchemaFormProps {
  schema: any;
  uiSchema?: any;
  formData?: any;
  onChange?: (data: any) => void;
  onSubmit?: (data: any) => void;
  onError?: (errors: any) => void;
  disabled?: boolean;
  readonly?: boolean;
  liveValidate?: boolean;
  showErrorList?: boolean;
  className?: string;
}

export interface DataTableProps<T> {
  data: T[];
  columns: Column[];
  loading?: boolean;
  error?: string;
  selectable?: boolean;
  multiSelect?: boolean;
  selectedRows?: Set<string>;
  onSelectionChange?: (selected: Set<string>) => void;
  sortable?: boolean;
  sortConfig?: SortConfig;
  onSortChange?: (sort: SortConfig) => void;
  filterable?: boolean;
  filters?: Filter[];
  onFilterChange?: (filters: Record<string, any>) => void;
  paginated?: boolean;
  pagination?: PaginationConfig;
  onPageChange?: (page: number) => void;
  onPerPageChange?: (perPage: number) => void;
  virtualized?: boolean;
  className?: string;
}

export interface FileDropzoneProps {
  onDrop: (files: File[]) => void;
  accept?: string | string[];
  multiple?: boolean;
  maxFiles?: number;
  maxSize?: number;
  disabled?: boolean;
  className?: string;
  children?: React.ReactNode;
}

export interface ToastsProps {
  toasts: Notification[];
  onRemove: (id: string) => void;
  position?: 'top-left' | 'top-right' | 'bottom-left' | 'bottom-right';
  maxToasts?: number;
  className?: string;
}

export interface BreadcrumbsProps {
  items: BreadcrumbItem[];
  separator?: React.ReactNode;
  className?: string;
}

export interface LineageGraphProps {
  graph: LineageGraph;
  onNodeClick?: (node: LineageNode) => void;
  onNodeHover?: (node: LineageNode | null) => void;
  selectedNode?: string;
  className?: string;
}