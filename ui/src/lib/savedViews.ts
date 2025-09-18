// Local storage utilities for saved search views

export interface SavedView {
  id: string
  name: string
  filters: Record<string, any>
  columns: string[]
  sorting?: {
    field: string
    direction: 'asc' | 'desc'
  }
  created_at: string
  updated_at: string
}

const STORAGE_KEY = 'blacklake_saved_views'

export function getSavedViews(): SavedView[] {
  try {
    const stored = localStorage.getItem(STORAGE_KEY)
    return stored ? JSON.parse(stored) : []
  } catch (error) {
    console.error('Failed to load saved views:', error)
    return []
  }
}

export function saveView(view: Omit<SavedView, 'id' | 'created_at' | 'updated_at'>): SavedView {
  const views = getSavedViews()
  const now = new Date().toISOString()
  
  const newView: SavedView = {
    ...view,
    id: generateId(),
    created_at: now,
    updated_at: now
  }
  
  views.push(newView)
  
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(views))
  } catch (error) {
    console.error('Failed to save view:', error)
    throw new Error('Failed to save view. Storage may be full.')
  }
  
  return newView
}

export function updateView(id: string, updates: Partial<SavedView>): SavedView | null {
  const views = getSavedViews()
  const index = views.findIndex(view => view.id === id)
  
  if (index === -1) return null
  
  const updatedView: SavedView = {
    ...views[index],
    ...updates,
    updated_at: new Date().toISOString()
  }
  
  views[index] = updatedView
  
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(views))
  } catch (error) {
    console.error('Failed to update view:', error)
    throw new Error('Failed to update view.')
  }
  
  return updatedView
}

export function deleteView(id: string): boolean {
  const views = getSavedViews()
  const filteredViews = views.filter(view => view.id !== id)
  
  if (filteredViews.length === views.length) return false
  
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(filteredViews))
    return true
  } catch (error) {
    console.error('Failed to delete view:', error)
    return false
  }
}

export function exportViews(): string {
  const views = getSavedViews()
  return JSON.stringify(views, null, 2)
}

export function importViews(jsonString: string): number {
  try {
    const importedViews: SavedView[] = JSON.parse(jsonString)
    
    // Validate the imported data
    if (!Array.isArray(importedViews)) {
      throw new Error('Invalid format: expected an array of views')
    }
    
    const existingViews = getSavedViews()
    const mergedViews = [...existingViews]
    
    for (const view of importedViews) {
      // Check if view already exists
      const existingIndex = mergedViews.findIndex(existing => existing.id === view.id)
      
      if (existingIndex >= 0) {
        // Update existing view
        mergedViews[existingIndex] = {
          ...view,
          updated_at: new Date().toISOString()
        }
      } else {
        // Add new view
        mergedViews.push(view)
      }
    }
    
    localStorage.setItem(STORAGE_KEY, JSON.stringify(mergedViews))
    return importedViews.length
  } catch (error) {
    console.error('Failed to import views:', error)
    throw new Error('Failed to import views. Invalid format.')
  }
}

function generateId(): string {
  return Date.now().toString(36) + Math.random().toString(36).substr(2)
}
