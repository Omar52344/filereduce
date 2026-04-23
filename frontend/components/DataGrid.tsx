'use client';

import {
  flexRender,
  getCoreRowModel,
  getSortedRowModel,
  useReactTable,
  type ColumnDef,
  type SortingState,
} from '@tanstack/react-table';
import { useState } from 'react';

interface EdifactDocument {
  interchange_id: string;
  sender: string;
  receiver: string;
  document_number: string;
  doc_type: string;
  document_date?: string;
  requested_delivery_date?: string;
  buyer?: string;
  seller?: string;
  currency: string;
  line_count_check?: number;
  lines: Array<{
    line_no: number;
    sku: string;
    qty?: number;
    uom?: string;
    amount?: number;
    extra?: Record<string, string>;
  }>;
  extra?: Record<string, string>;
}

interface DataGridProps {
  data: EdifactDocument[];
}

export default function DataGrid({ data }: DataGridProps) {
  const [sorting, setSorting] = useState<SortingState>([]);

  const columns: ColumnDef<EdifactDocument>[] = [
    {
      header: 'Doc #',
      accessorKey: 'document_number',
      cell: ({ row }) => (
        <div className="font-medium">{row.original.document_number}</div>
      ),
    },
    {
      header: 'Type',
      accessorKey: 'doc_type',
      cell: ({ row }) => (
        <span className="px-2 py-1 bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200 text-xs rounded-full">
          {row.original.doc_type}
        </span>
      ),
    },
    {
      header: 'Sender',
      accessorKey: 'sender',
    },
    {
      header: 'Receiver',
      accessorKey: 'receiver',
    },
    {
      header: 'Date',
      accessorKey: 'document_date',
      cell: ({ row }) => row.original.document_date || '\u2014',
    },
    {
      header: 'Lines',
      accessorKey: 'lines',
      cell: ({ row }) => (
        <div className="text-center">
          <span className="font-semibold">{row.original.lines?.length ?? 0}</span>
          {row.original.line_count_check && (
            <span className="text-xs text-gray-500 ml-1">
              ({row.original.line_count_check})
            </span>
          )}
        </div>
      ),
    },
    {
      header: 'Buyer',
      accessorKey: 'buyer',
      cell: ({ row }) => row.original.buyer || '\u2014',
    },
    {
      header: 'Seller',
      accessorKey: 'seller',
      cell: ({ row }) => row.original.seller || '\u2014',
    },
    {
      header: 'Currency',
      accessorKey: 'currency',
      cell: ({ row }) => (
        <span className="font-mono">{row.original.currency}</span>
      ),
    },
  ];

  const table = useReactTable({
    data,
    columns,
    state: {
      sorting,
    },
    onSortingChange: setSorting,
    getCoreRowModel: getCoreRowModel(),
    getSortedRowModel: getSortedRowModel(),
  });

  if (data.length === 0) {
    return (
      <div className="text-center py-12 text-gray-500 dark:text-gray-400">
        No data to display
      </div>
    );
  }

  return (
    <div className="rounded-lg border border-gray-200 dark:border-gray-800 overflow-hidden">
      <div className="overflow-x-auto">
        <table className="min-w-full divide-y divide-gray-200 dark:divide-gray-800">
          <thead className="bg-gray-50 dark:bg-gray-900">
            {table.getHeaderGroups().map((headerGroup) => (
              <tr key={headerGroup.id}>
                {headerGroup.headers.map((header) => (
                  <th
                    key={header.id}
                    className="px-4 py-3 text-left text-xs font-medium text-gray-700 dark:text-gray-300 uppercase tracking-wider"
                  >
                    {header.isPlaceholder
                      ? null
                      : flexRender(
                          header.column.columnDef.header,
                          header.getContext()
                        )}
                  </th>
                ))}
              </tr>
            ))}
          </thead>
          <tbody className="bg-white dark:bg-gray-950 divide-y divide-gray-200 dark:divide-gray-800">
            {table.getRowModel().rows.map((row) => (
              <tr
                key={row.id}
                className="hover:bg-gray-50 dark:hover:bg-gray-900 transition-colors"
              >
                {row.getVisibleCells().map((cell) => (
                  <td
                    key={cell.id}
                    className="px-4 py-3 text-sm text-gray-800 dark:text-gray-200"
                  >
                    {flexRender(cell.column.columnDef.cell, cell.getContext())}
                  </td>
                ))}
              </tr>
            ))}
          </tbody>
        </table>
      </div>
      <div className="px-4 py-3 border-t border-gray-200 dark:border-gray-800 text-xs text-gray-500 dark:text-gray-400">
        Showing {data.length} document{data.length !== 1 ? 's' : ''}
      </div>
    </div>
  );
}
