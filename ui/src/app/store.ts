import { create } from 'zustand'
import { devtools } from 'zustand/middleware'

interface Toast {
  id: string
  message: string
  type: 'success' | 'error' | 'warning' | 'info'
}

interface AppState {
  // Current context
  currentRepo: string | null
  currentBranch: string
  currentPath: string
  
  // UI state
  selections: string[]
  
  // Toasts
  toasts: Toast[]
  
  // Actions
  setCurrentRepo: (repo: string | null) => void
  setCurrentBranch: (branch: string) => void
  setCurrentPath: (path: string) => void
  setSelections: (selections: string[]) => void
  addSelection: (selection: string) => void
  removeSelection: (selection: string) => void
  clearSelections: () => void
  addToast: (message: string, type?: Toast['type']) => void
  removeToast: (id: string) => void
  clearToasts: () => void
}

export const useAppStore = create<AppState>()(
  devtools(
    (set, get) => ({
      // Initial state
      currentRepo: null,
      currentBranch: 'main',
      currentPath: '',
      selections: [],
      toasts: [],
      
      // Actions
      setCurrentRepo: (repo) => set({ currentRepo: repo }),
      setCurrentBranch: (branch) => set({ currentBranch: branch }),
      setCurrentPath: (path) => set({ currentPath: path }),
      
      setSelections: (selections) => set({ selections }),
      addSelection: (selection) => {
        const { selections } = get()
        if (!selections.includes(selection)) {
          set({ selections: [...selections, selection] })
        }
      },
      removeSelection: (selection) => {
        const { selections } = get()
        set({ selections: selections.filter(s => s !== selection) })
      },
      clearSelections: () => set({ selections: [] }),
      
      addToast: (message, type = 'info') => {
        const id = Date.now().toString()
        const toast: Toast = { id, message, type }
        set(state => ({ toasts: [...state.toasts, toast] }))
        
        // Auto-remove toast after 5 seconds
        setTimeout(() => {
          set(state => ({ toasts: state.toasts.filter(t => t.id !== id) }))
        }, 5000)
      },
      
      removeToast: (id) => {
        set(state => ({ toasts: state.toasts.filter(t => t.id !== id) }))
      },
      
      clearToasts: () => set({ toasts: [] }),
    }),
    {
      name: 'app-store',
    }
  )
)