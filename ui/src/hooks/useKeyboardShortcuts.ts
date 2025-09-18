// BlackLake Keyboard Shortcuts
// Week 4: Global keyboard shortcuts and user preferences

import { useEffect, useCallback, useRef } from 'react';

export interface KeyboardShortcut {
  key: string;
  ctrl?: boolean;
  shift?: boolean;
  alt?: boolean;
  meta?: boolean;
  action: () => void;
  description: string;
  category: string;
}

export interface ShortcutPreferences {
  enabled: boolean;
  shortcuts: Record<string, KeyboardShortcut>;
}

const DEFAULT_SHORTCUTS: Record<string, KeyboardShortcut> = {
  search: {
    key: '/',
    action: () => {
      const searchInput = document.querySelector('input[placeholder*="search" i], input[type="search"]') as HTMLInputElement;
      if (searchInput) {
        searchInput.focus();
        searchInput.select();
      }
    },
    description: 'Focus search',
    category: 'Navigation',
  },
  upload: {
    key: 'u',
    ctrl: true,
    action: () => {
      const uploadButton = document.querySelector('[data-shortcut="upload"]') as HTMLElement;
      if (uploadButton) {
        uploadButton.click();
      }
    },
    description: 'Open upload dialog',
    category: 'Actions',
  },
  refresh: {
    key: '.',
    action: () => {
      const refreshButton = document.querySelector('[data-shortcut="refresh"]') as HTMLElement;
      if (refreshButton) {
        refreshButton.click();
      }
    },
    description: 'Refresh current view',
    category: 'Actions',
  },
  help: {
    key: '?',
    action: () => {
      const helpButton = document.querySelector('[data-shortcut="help"]') as HTMLElement;
      if (helpButton) {
        helpButton.click();
      }
    },
    description: 'Show help',
    category: 'Navigation',
  },
  settings: {
    key: ',',
    ctrl: true,
    action: () => {
      const settingsButton = document.querySelector('[data-shortcut="settings"]') as HTMLElement;
      if (settingsButton) {
        settingsButton.click();
      }
    },
    description: 'Open settings',
    category: 'Navigation',
  },
  newRepo: {
    key: 'n',
    ctrl: true,
    shift: true,
    action: () => {
      const newRepoButton = document.querySelector('[data-shortcut="new-repo"]') as HTMLElement;
      if (newRepoButton) {
        newRepoButton.click();
      }
    },
    description: 'Create new repository',
    category: 'Actions',
  },
  toggleTheme: {
    key: 't',
    ctrl: true,
    action: () => {
      const themeButton = document.querySelector('[data-shortcut="theme"]') as HTMLElement;
      if (themeButton) {
        themeButton.click();
      }
    },
    description: 'Toggle theme',
    category: 'Preferences',
  },
  export: {
    key: 'e',
    ctrl: true,
    action: () => {
      const exportButton = document.querySelector('[data-shortcut="export"]') as HTMLElement;
      if (exportButton) {
        exportButton.click();
      }
    },
    description: 'Export selected items',
    category: 'Actions',
  },
  selectAll: {
    key: 'a',
    ctrl: true,
    action: () => {
      const selectAllButton = document.querySelector('[data-shortcut="select-all"]') as HTMLElement;
      if (selectAllButton) {
        selectAllButton.click();
      }
    },
    description: 'Select all items',
    category: 'Selection',
  },
  deselectAll: {
    key: 'Escape',
    action: () => {
      const deselectButton = document.querySelector('[data-shortcut="deselect-all"]') as HTMLElement;
      if (deselectButton) {
        deselectButton.click();
      }
    },
    description: 'Deselect all items',
    category: 'Selection',
  },
};

export const useKeyboardShortcuts = () => {
  const shortcutsRef = useRef<Record<string, KeyboardShortcut>>(DEFAULT_SHORTCUTS);
  const enabledRef = useRef(true);

  // Load preferences from localStorage
  useEffect(() => {
    const saved = localStorage.getItem('blacklake-shortcuts');
    if (saved) {
      try {
        const preferences: ShortcutPreferences = JSON.parse(saved);
        enabledRef.current = preferences.enabled;
        shortcutsRef.current = { ...DEFAULT_SHORTCUTS, ...preferences.shortcuts };
      } catch (error) {
        console.error('Failed to load keyboard shortcuts preferences:', error);
      }
    }
  }, []);

  // Save preferences to localStorage
  const savePreferences = useCallback((preferences: ShortcutPreferences) => {
    localStorage.setItem('blacklake-shortcuts', JSON.stringify(preferences));
    enabledRef.current = preferences.enabled;
    shortcutsRef.current = { ...DEFAULT_SHORTCUTS, ...preferences.shortcuts };
  }, []);

  // Handle keyboard events
  const handleKeyDown = useCallback((event: KeyboardEvent) => {
    if (!enabledRef.current) return;

    // Don't trigger shortcuts when typing in inputs
    const target = event.target as HTMLElement;
    if (
      target.tagName === 'INPUT' ||
      target.tagName === 'TEXTAREA' ||
      target.contentEditable === 'true' ||
      target.closest('[contenteditable="true"]')
    ) {
      // Allow some shortcuts even in inputs
      const allowedInInputs = ['Escape', '?'];
      if (!allowedInInputs.includes(event.key)) {
        return;
      }
    }

    // Find matching shortcut
    const matchingShortcut = Object.values(shortcutsRef.current).find(shortcut => {
      return (
        shortcut.key.toLowerCase() === event.key.toLowerCase() &&
        !!shortcut.ctrl === event.ctrlKey &&
        !!shortcut.shift === event.shiftKey &&
        !!shortcut.alt === event.altKey &&
        !!shortcut.meta === event.metaKey
      );
    });

    if (matchingShortcut) {
      event.preventDefault();
      event.stopPropagation();
      matchingShortcut.action();
    }
  }, []);

  // Register global keyboard event listener
  useEffect(() => {
    document.addEventListener('keydown', handleKeyDown);
    return () => {
      document.removeEventListener('keydown', handleKeyDown);
    };
  }, [handleKeyDown]);

  // Get current shortcuts
  const getShortcuts = useCallback(() => {
    return shortcutsRef.current;
  }, []);

  // Get shortcuts by category
  const getShortcutsByCategory = useCallback(() => {
    const shortcuts = shortcutsRef.current;
    const categories: Record<string, KeyboardShortcut[]> = {};
    
    Object.values(shortcuts).forEach(shortcut => {
      if (!categories[shortcut.category]) {
        categories[shortcut.category] = [];
      }
      categories[shortcut.category].push(shortcut);
    });

    return categories;
  }, []);

  // Update a shortcut
  const updateShortcut = useCallback((id: string, shortcut: Partial<KeyboardShortcut>) => {
    const current = shortcutsRef.current;
    if (current[id]) {
      current[id] = { ...current[id], ...shortcut };
      savePreferences({
        enabled: enabledRef.current,
        shortcuts: current,
      });
    }
  }, [savePreferences]);

  // Reset shortcuts to defaults
  const resetShortcuts = useCallback(() => {
    shortcutsRef.current = { ...DEFAULT_SHORTCUTS };
    savePreferences({
      enabled: enabledRef.current,
      shortcuts: shortcutsRef.current,
    });
  }, [savePreferences]);

  // Toggle shortcuts on/off
  const toggleShortcuts = useCallback((enabled: boolean) => {
    enabledRef.current = enabled;
    savePreferences({
      enabled,
      shortcuts: shortcutsRef.current,
    });
  }, [savePreferences]);

  // Format shortcut display
  const formatShortcut = useCallback((shortcut: KeyboardShortcut) => {
    const parts: string[] = [];
    
    if (shortcut.ctrl) parts.push('Ctrl');
    if (shortcut.shift) parts.push('Shift');
    if (shortcut.alt) parts.push('Alt');
    if (shortcut.meta) parts.push('Cmd');
    
    parts.push(shortcut.key);
    
    return parts.join(' + ');
  }, []);

  return {
    shortcuts: shortcutsRef.current,
    enabled: enabledRef.current,
    getShortcuts,
    getShortcutsByCategory,
    updateShortcut,
    resetShortcuts,
    toggleShortcuts,
    formatShortcut,
    savePreferences,
  };
};

// Hook for components that need to register shortcut targets
export const useShortcutTarget = (shortcutId: string, element: HTMLElement | null) => {
  useEffect(() => {
    if (element) {
      element.setAttribute('data-shortcut', shortcutId);
    }
    return () => {
      if (element) {
        element.removeAttribute('data-shortcut');
      }
    };
  }, [shortcutId, element]);
};
