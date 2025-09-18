import { useVirtualizer, Virtualizer } from '@tanstack/react-virtual';
import { RefObject, useMemo } from 'react';

// Virtual list configuration
export interface VirtualListConfig {
  count: number;
  getScrollElement: () => HTMLElement | null;
  estimateSize?: (index: number) => number;
  overscan?: number;
  enabled?: boolean;
}

// Hook for virtualized lists
export function useVirtualList<T>({
  count,
  getScrollElement,
  estimateSize = () => 50,
  overscan = 5,
  enabled = true,
}: VirtualListConfig): Virtualizer<HTMLElement, Element> {
  return useVirtualizer({
    count,
    getScrollElement,
    estimateSize,
    overscan,
    enabled,
  });
}

// Hook for virtualized tables
export function useVirtualTable<T>({
  count,
  getScrollElement,
  estimateSize = () => 40,
  overscan = 10,
  enabled = true,
}: VirtualListConfig): Virtualizer<HTMLElement, Element> {
  return useVirtualizer({
    count,
    getScrollElement,
    estimateSize,
    overscan,
    enabled,
  });
}

// Hook for virtualized trees
export function useVirtualTree<T>({
  count,
  getScrollElement,
  estimateSize = () => 32,
  overscan = 5,
  enabled = true,
}: VirtualListConfig): Virtualizer<HTMLElement, Element> {
  return useVirtualizer({
    count,
    getScrollElement,
    estimateSize,
    overscan,
    enabled,
  });
}

// Utility to calculate item size based on content
export function calculateItemSize<T>(
  item: T,
  type: 'list' | 'table' | 'tree' = 'list'
): number {
  switch (type) {
    case 'list':
      // Base size for list items
      return 60;
    case 'table':
      // Base size for table rows
      return 40;
    case 'tree':
      // Base size for tree nodes
      return 32;
    default:
      return 50;
  }
}

// Utility to get scroll element from ref
export function getScrollElementFromRef(
  ref: RefObject<HTMLElement>
): () => HTMLElement | null {
  return () => ref.current;
}

// Utility to create virtual list props
export function createVirtualListProps<T>(
  virtualizer: Virtualizer<HTMLElement, Element>,
  items: T[],
  renderItem: (item: T, index: number) => React.ReactNode
) {
  return {
    style: {
      height: `${virtualizer.getTotalSize()}px`,
      width: '100%',
      position: 'relative' as const,
    },
    children: virtualizer.getVirtualItems().map((virtualItem) => (
      <div
        key={virtualItem.key}
        style={{
          position: 'absolute' as const,
          top: 0,
          left: 0,
          width: '100%',
          height: `${virtualItem.size}px`,
          transform: `translateY(${virtualItem.start}px)`,
        }}
      >
        {renderItem(items[virtualItem.index], virtualItem.index)}
      </div>
    )),
  };
}

// Utility to create virtual table props
export function createVirtualTableProps<T>(
  virtualizer: Virtualizer<HTMLElement, Element>,
  items: T[],
  renderRow: (item: T, index: number) => React.ReactNode
) {
  return {
    style: {
      height: `${virtualizer.getTotalSize()}px`,
      width: '100%',
      position: 'relative' as const,
    },
    children: virtualizer.getVirtualItems().map((virtualItem) => (
      <tr
        key={virtualItem.key}
        style={{
          position: 'absolute' as const,
          top: 0,
          left: 0,
          width: '100%',
          height: `${virtualItem.size}px`,
          transform: `translateY(${virtualItem.start}px)`,
        }}
      >
        {renderRow(items[virtualItem.index], virtualItem.index)}
      </tr>
    )),
  };
}

// Utility to create virtual tree props
export function createVirtualTreeProps<T>(
  virtualizer: Virtualizer<HTMLElement, Element>,
  items: T[],
  renderNode: (item: T, index: number) => React.ReactNode
) {
  return {
    style: {
      height: `${virtualizer.getTotalSize()}px`,
      width: '100%',
      position: 'relative' as const,
    },
    children: virtualizer.getVirtualItems().map((virtualItem) => (
      <div
        key={virtualItem.key}
        style={{
          position: 'absolute' as const,
          top: 0,
          left: 0,
          width: '100%',
          height: `${virtualItem.size}px`,
          transform: `translateY(${virtualItem.start}px)`,
        }}
      >
        {renderNode(items[virtualItem.index], virtualItem.index)}
      </div>
    )),
  };
}

// Hook for infinite scrolling
export function useInfiniteScroll<T>(
  items: T[],
  hasNextPage: boolean,
  isFetching: boolean,
  fetchNextPage: () => void,
  threshold = 5
) {
  const virtualizer = useVirtualList({
    count: items.length,
    getScrollElement: () => document.getElementById('scroll-container'),
    estimateSize: () => 50,
    overscan: 10,
  });

  const virtualItems = virtualizer.getVirtualItems();
  const lastItem = virtualItems[virtualItems.length - 1];

  // Trigger fetch when approaching the end
  if (
    lastItem &&
    lastItem.index >= items.length - threshold &&
    hasNextPage &&
    !isFetching
  ) {
    fetchNextPage();
  }

  return virtualizer;
}

// Utility to handle scroll restoration
export function useScrollRestoration(
  key: string,
  getScrollElement: () => HTMLElement | null
) {
  const scrollKey = `scroll-${key}`;
  
  const saveScrollPosition = () => {
    const element = getScrollElement();
    if (element) {
      sessionStorage.setItem(scrollKey, element.scrollTop.toString());
    }
  };

  const restoreScrollPosition = () => {
    const element = getScrollElement();
    const savedPosition = sessionStorage.getItem(scrollKey);
    if (element && savedPosition) {
      element.scrollTop = parseInt(savedPosition, 10);
    }
  };

  const clearScrollPosition = () => {
    sessionStorage.removeItem(scrollKey);
  };

  return {
    saveScrollPosition,
    restoreScrollPosition,
    clearScrollPosition,
  };
}

// Utility to handle keyboard navigation
export function useKeyboardNavigation<T>(
  items: T[],
  selectedIndex: number,
  onSelect: (index: number) => void,
  onActivate?: (index: number) => void
) {
  const handleKeyDown = (event: React.KeyboardEvent) => {
    switch (event.key) {
      case 'ArrowDown':
        event.preventDefault();
        if (selectedIndex < items.length - 1) {
          onSelect(selectedIndex + 1);
        }
        break;
      case 'ArrowUp':
        event.preventDefault();
        if (selectedIndex > 0) {
          onSelect(selectedIndex - 1);
        }
        break;
      case 'Home':
        event.preventDefault();
        onSelect(0);
        break;
      case 'End':
        event.preventDefault();
        onSelect(items.length - 1);
        break;
      case 'Enter':
      case ' ':
        event.preventDefault();
        if (onActivate) {
          onActivate(selectedIndex);
        }
        break;
    }
  };

  return { handleKeyDown };
}
