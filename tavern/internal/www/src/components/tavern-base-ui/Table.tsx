import React, { Fragment } from 'react'

import {
  useReactTable,
  getCoreRowModel,
  ColumnDef,
  flexRender,
} from '@tanstack/react-table'

type TableProps<TData> = {
    data: TData[],
    columns: ColumnDef<TData>[],
    onRowClick: (e: any) => void
}

export const Table = ({
    data,
    columns,
    onRowClick
  }: TableProps<any>): JSX.Element => {
    const table = useReactTable<any>({
      data,
      columns,
      getCoreRowModel: getCoreRowModel(),
    })

    return (
      <div className="p-2">
        <div className="h-2" />
        <table className="w-full divide-y divide-gray-200 overflow-scroll table-fixed">
          <thead className="bg-gray-50">
            {table.getHeaderGroups().map(headerGroup => (
              <tr key={headerGroup.id}>
                {headerGroup.headers.map(header => {
                  return (
                    <th 
                    key={header.id} 
                    colSpan={header.colSpan}
                    scope="col"
                    className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider"
                    >
                      {header.isPlaceholder ? null : (
                        <div>
                          {flexRender(
                            header.column.columnDef.header,
                            header.getContext()
                          )}
                        </div>
                      )}
                    </th>
                  )
                })}
              </tr>
            ))}
          </thead>
          <tbody className="bg-white divide-y divide-gray-200">
            {table.getRowModel().rows.map(row => {
              return (
                <Fragment key={row.id}>
                  <tr onClick={() =>onRowClick(row)}>
                    {/* first row is a normal row */}
                    {row.getVisibleCells().map(cell => {
                      return (
                        <td key={cell.id} className="px-6 py-4">
                          {flexRender(
                            cell.column.columnDef.cell,
                            cell.getContext()
                          )}
                        </td>
                      )
                    })}
                  </tr>
                </Fragment>
              )
            })}
          </tbody>
        </table>
      </div>
    )
}
export default Table;