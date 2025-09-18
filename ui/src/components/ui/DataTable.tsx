import React, { useState } from 'react'
import { Button } from './button'

interface Column {
  key: string
  header: string
  width?: string
  sortable?: boolean
  render?: (value: any, row: any) => React.ReactNode
}

interface DataTableProps {
  columns: Column[]
  data: any[]
  loading?: boolean
  pagination?: {
    page: number
    totalPages: number
    onPageChange: (page: number) => void
  }
  sorting?: {
    field: string | null
    direction: 'asc' | 'desc' | null
    onSort: (field: string) => void
  }
  selection?: {
    selectedRows: string[]
    onSelectionChange: (selectedRows: string[]) => void
    rowKey: string
  }
  className?: string
}

export const DataTable: React.FC<DataTableProps> = ({
  columns,
  data,
  loading = false,
  pagination,
  sorting,
  selection,
  className = ""
}) => {
  const handleSort = (field: string) => {
    if (sorting && columns.find(col => col.key === field)?.sortable) {
      sorting.onSort(field)
    }
  }

  const handleSelectAll = (checked: boolean) => {
    if (selection) {
      if (checked) {
        const allKeys = data.map(row => row[selection.rowKey])
        selection.onSelectionChange(allKeys)
      } else {
        selection.onSelectionChange([])
      }
    }
  }

  const handleSelectRow = (rowKey: string, checked: boolean) => {
    if (selection) {
      if (checked) {
        selection.onSelectionChange([...selection.selectedRows, rowKey])
      } else {
        selection.onSelectionChange(selection.selectedRows.filter(key => key !== rowKey))
      }
    }
  }

  const getSortIcon = (field: string) => {
    if (!sorting || sorting.field !== field) return '↕️'
    return sorting.direction === 'asc' ? '↑' : '↓'
  }

  return (
    <div className={`space-y-4 ${className}`}>
      {/* Table */}
      <div className="border border-border rounded-lg overflow-hidden">
        <div className="overflow-x-auto">
          <table className="w-full">
            <thead className="bg-muted">
              <tr>
                {selection && (
                  <th className="p-4 w-12">
                    <input
                      type="checkbox"
                      checked={selection.selectedRows.length === data.length && data.length > 0}
                      onChange={(e) => handleSelectAll(e.target.checked)}
                      className="rounded border-input"
                    />
                  </th>
                )}
                {columns.map((column) => (
                  <th
                    key={column.key}
                    className={`text-left p-4 ${column.width || ''} ${
                      column.sortable ? 'cursor-pointer hover:bg-muted/80' : ''
                    }`}
                    onClick={() => column.sortable && handleSort(column.key)}
                  >
                    <div className="flex items-center space-x-1">
                      <span>{column.header}</span>
                      {column.sortable && (
                        <span className="text-xs">{getSortIcon(column.key)}</span>
                      )}
                    </div>
                  </th>
                ))}
              </tr>
            </thead>
            <tbody>
              {loading ? (
                <tr>
                  <td colSpan={columns.length + (selection ? 1 : 0)} className="p-8 text-center">
                    <div className="loading-spinner h-6 w-6 mx-auto mb-2"></div>
                    <p className="text-muted-foreground">Loading...</p>
                  </td>
                </tr>
              ) : data.length === 0 ? (
                <tr>
                  <td colSpan={columns.length + (selection ? 1 : 0)} className="p-8 text-center">
                    <p className="text-muted-foreground">No data available</p>
                  </td>
                </tr>
              ) : (
                data.map((row, index) => {
                  const rowKey = selection ? row[selection.rowKey] : index
                  const isSelected = selection ? selection.selectedRows.includes(rowKey) : false
                  
                  return (
                    <tr
                      key={rowKey}
                      className={`border-t border-border hover:bg-accent ${
                        isSelected ? 'bg-primary/5' : ''
                      }`}
                    >
                      {selection && (
                        <td className="p-4">
                          <input
                            type="checkbox"
                            checked={isSelected}
                            onChange={(e) => handleSelectRow(rowKey, e.target.checked)}
                            className="rounded border-input"
                          />
                        </td>
                      )}
                      {columns.map((column) => (
                        <td key={column.key} className="p-4">
                          {column.render
                            ? column.render(row[column.key], row)
                            : row[column.key]
                          }
                        </td>
                      ))}
                    </tr>
                  )
                })
              )}
            </tbody>
          </table>
        </div>
      </div>

      {/* Pagination */}
      {pagination && pagination.totalPages > 1 && (
        <div className="flex items-center justify-between">
          <p className="text-sm text-muted-foreground">
            Page {pagination.page} of {pagination.totalPages}
          </p>
          <div className="flex space-x-2">
            <Button
              variant="outline"
              size="sm"
              disabled={pagination.page <= 1}
              onClick={() => pagination.onPageChange(pagination.page - 1)}
            >
              Previous
            </Button>
            <Button
              variant="outline"
              size="sm"
              disabled={pagination.page >= pagination.totalPages}
              onClick={() => pagination.onPageChange(pagination.page + 1)}
            >
              Next
            </Button>
          </div>
        </div>
      )}

      {/* Selection Info */}
      {selection && selection.selectedRows.length > 0 && (
        <div className="flex items-center justify-between p-3 bg-primary/10 border border-primary/20 rounded-lg">
          <p className="text-sm">
            {selection.selectedRows.length} item(s) selected
          </p>
          <Button
            variant="outline"
            size="sm"
            onClick={() => selection.onSelectionChange([])}
          >
            Clear Selection
          </Button>
        </div>
      )}
    </div>
  )
}
