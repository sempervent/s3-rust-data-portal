import * as React from "react";
import {
  ColumnDef,
  flexRender,
  getCoreRowModel,
  getFilteredRowModel,
  getPaginationRowModel,
  getSortedRowModel,
  useReactTable,
  SortingState,
  ColumnFiltersState,
  VisibilityState,
} from "@tanstack/react-table";
import { ChevronDown, ChevronUp, ArrowUpDown } from "lucide-react";
import { cn } from "@/utils/cn";
import { Button } from "./button";

interface DataTableProps<TData, TValue> {
  columns: ColumnDef<TData, TValue>[];
  data: TData[];
  loading?: boolean;
  error?: string;
  selectable?: boolean;
  multiSelect?: boolean;
  selectedRows?: Set<string>;
  onSelectionChange?: (selected: Set<string>) => void;
  sortable?: boolean;
  filterable?: boolean;
  paginated?: boolean;
  virtualized?: boolean;
  className?: string;
}

export function DataTable<TData, TValue>({
  columns,
  data,
  loading = false,
  error,
  selectable = false,
  multiSelect = false,
  selectedRows = new Set(),
  onSelectionChange,
  sortable = true,
  filterable = false,
  paginated = false,
  virtualized = false,
  className,
}: DataTableProps<TData, TValue>) {
  const [sorting, setSorting] = React.useState<SortingState>([]);
  const [columnFilters, setColumnFilters] = React.useState<ColumnFiltersState>([]);
  const [columnVisibility, setColumnVisibility] = React.useState<VisibilityState>({});
  const [rowSelection, setRowSelection] = React.useState<Record<string, boolean>>({});

  // Add selection column if selectable
  const selectColumn: ColumnDef<TData, TValue> = React.useMemo(
    () => ({
      id: "select",
      header: ({ table }) => (
        <input
          type="checkbox"
          checked={table.getIsAllPageRowsSelected()}
          onChange={(e) => table.toggleAllPageRowsSelected(e.target.checked)}
          className="rounded border-gray-300"
          aria-label="Select all rows"
        />
      ),
      cell: ({ row }) => (
        <input
          type="checkbox"
          checked={row.getIsSelected()}
          onChange={(e) => row.toggleSelected(e.target.checked)}
          className="rounded border-gray-300"
          aria-label={`Select row ${row.index}`}
        />
      ),
      enableSorting: false,
      enableHiding: false,
    }),
    []
  );

  const tableColumns = React.useMemo(() => {
    if (selectable) {
      return [selectColumn, ...columns];
    }
    return columns;
  }, [selectable, selectColumn, columns]);

  const table = useReactTable({
    data,
    columns: tableColumns,
    onSortingChange: setSorting,
    onColumnFiltersChange: setColumnFilters,
    getCoreRowModel: getCoreRowModel(),
    getPaginationRowModel: paginated ? getPaginationRowModel() : undefined,
    getSortedRowModel: sortable ? getSortedRowModel() : undefined,
    getFilteredRowModel: filterable ? getFilteredRowModel() : undefined,
    onColumnVisibilityChange: setColumnVisibility,
    onRowSelectionChange: setRowSelection,
    state: {
      sorting,
      columnFilters,
      columnVisibility,
      rowSelection,
    },
  });

  // Update external selection state
  React.useEffect(() => {
    if (onSelectionChange) {
      const selected = new Set(
        Object.keys(rowSelection).filter(key => rowSelection[key])
      );
      onSelectionChange(selected);
    }
  }, [rowSelection, onSelectionChange]);

  if (loading) {
    return (
      <div className="flex items-center justify-center h-32">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-primary"></div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="flex items-center justify-center h-32 text-red-500">
        <p>{error}</p>
      </div>
    );
  }

  return (
    <div className={cn("w-full", className)}>
      <div className="rounded-md border">
        <table className="w-full">
          <thead>
            {table.getHeaderGroups().map((headerGroup) => (
              <tr key={headerGroup.id} className="border-b bg-muted/50">
                {headerGroup.headers.map((header) => (
                  <th
                    key={header.id}
                    className={cn(
                      "px-4 py-2 text-left text-sm font-medium text-muted-foreground",
                      header.column.getCanSort() && "cursor-pointer select-none hover:bg-muted/70"
                    )}
                    onClick={header.column.getToggleSortingHandler()}
                  >
                    <div className="flex items-center gap-2">
                      {header.isPlaceholder
                        ? null
                        : flexRender(
                            header.column.columnDef.header,
                            header.getContext()
                          )}
                      {header.column.getCanSort() && (
                        <div className="flex flex-col">
                          {header.column.getIsSorted() === "asc" ? (
                            <ChevronUp className="h-3 w-3" />
                          ) : header.column.getIsSorted() === "desc" ? (
                            <ChevronDown className="h-3 w-3" />
                          ) : (
                            <ArrowUpDown className="h-3 w-3 opacity-50" />
                          )}
                        </div>
                      )}
                    </div>
                  </th>
                ))}
              </tr>
            ))}
          </thead>
          <tbody>
            {table.getRowModel().rows?.length ? (
              table.getRowModel().rows.map((row) => (
                <tr
                  key={row.id}
                  className={cn(
                    "border-b transition-colors hover:bg-muted/50",
                    row.getIsSelected() && "bg-muted"
                  )}
                >
                  {row.getVisibleCells().map((cell) => (
                    <td key={cell.id} className="px-4 py-2 text-sm">
                      {flexRender(
                        cell.column.columnDef.cell,
                        cell.getContext()
                      )}
                    </td>
                  ))}
                </tr>
              ))
            ) : (
              <tr>
                <td
                  colSpan={columns.length}
                  className="h-24 text-center text-muted-foreground"
                >
                  No results.
                </td>
              </tr>
            )}
          </tbody>
        </table>
      </div>

      {paginated && (
        <div className="flex items-center justify-between space-x-2 py-4">
          <div className="flex-1 text-sm text-muted-foreground">
            {table.getFilteredSelectedRowModel().rows.length} of{" "}
            {table.getFilteredRowModel().rows.length} row(s) selected.
          </div>
          <div className="flex items-center space-x-2">
            <Button
              variant="outline"
              size="sm"
              onClick={() => table.previousPage()}
              disabled={!table.getCanPreviousPage()}
            >
              Previous
            </Button>
            <Button
              variant="outline"
              size="sm"
              onClick={() => table.nextPage()}
              disabled={!table.getCanNextPage()}
            >
              Next
            </Button>
          </div>
        </div>
      )}
    </div>
  );
}

// Enhanced column helper
export function createColumn<T>(
  id: string,
  header: string,
  accessorKey?: keyof T,
  options?: {
    sortable?: boolean;
    filterable?: boolean;
    width?: number;
    minWidth?: number;
    maxWidth?: number;
    align?: 'left' | 'center' | 'right';
    render?: (value: any, row: T) => React.ReactNode;
  }
): ColumnDef<T> {
  return {
    id,
    accessorKey: accessorKey as string,
    header,
    cell: ({ getValue, row }) => {
      const value = getValue();
      if (options?.render) {
        return options.render(value, row.original);
      }
      return value;
    },
    enableSorting: options?.sortable ?? true,
    enableColumnFilter: options?.filterable ?? false,
    size: options?.width,
    minSize: options?.minWidth,
    maxSize: options?.maxWidth,
  };
}

// Utility to create action column
export function createActionColumn<T>(
  id: string,
  header: string,
  actions: Array<{
    label: string;
    onClick: (row: T) => void;
    icon?: React.ReactNode;
    variant?: 'default' | 'destructive' | 'outline' | 'secondary' | 'ghost' | 'link';
  }>
): ColumnDef<T> {
  return {
    id,
    header,
    cell: ({ row }) => (
      <div className="flex items-center gap-2">
        {actions.map((action, index) => (
          <Button
            key={index}
            variant={action.variant || 'ghost'}
            size="sm"
            onClick={() => action.onClick(row.original)}
          >
            {action.icon}
            {action.label}
          </Button>
        ))}
      </div>
    ),
    enableSorting: false,
    enableColumnFilter: false,
  };
}
