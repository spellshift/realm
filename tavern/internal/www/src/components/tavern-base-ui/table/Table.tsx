import React, { Fragment } from 'react'

import {
  useReactTable,
  getCoreRowModel,
  ColumnDef,
  flexRender,
  getSortedRowModel,
  Row,
  getExpandedRowModel
} from '@tanstack/react-table'
import { TriangleDownIcon, TriangleUpIcon } from '@chakra-ui/icons'

export type TableSorting = {
  key: string,
  ascending: boolean,
}

type TableProps<TData> = {
  data: TData[],
  columns: ColumnDef<TData>[],
  onRowClick?: (e: any, event?: any) => void,
  renderSubComponent?: (props: { row: Row<TData> }) => React.ReactElement,
  getRowCanExpand?: (row: Row<TData>) => boolean
}

export const Table = ({
  data,
  columns,
  renderSubComponent,
  getRowCanExpand,
  onRowClick,
}: TableProps<any>): JSX.Element => {
  const table = useReactTable<any>({
    data,
    columns,
    getRowCanExpand,
    getCoreRowModel: getCoreRowModel(),
    getSortedRowModel: getSortedRowModel(),
    getExpandedRowModel: getExpandedRowModel(),
  })

  const tbodyRef = React.useRef<HTMLTableSectionElement>(null);
  // Function to handle key press on a row
  const handleKeyDown = (event: any, row: any) => {
    event.stopPropagation();
    if (onRowClick && event.key === "Enter") {
      onRowClick(row);
    }
  };

  return (
    <div className="overflow-x-auto -mx-4 sm:-mx-6 xl:-mx-8">
      <div className="inline-block min-w-full align-middle">
        <table className="w-full divide-y divide-gray-200 table-fixed">
          <thead className="bg-gray-50 sticky top-0 z-5">
            {table.getHeaderGroups().map(headerGroup => (
              <tr key={headerGroup.id}>
                {headerGroup.headers.map(header => {
                  return (
                    <th
                      key={header.id}
                      colSpan={header.colSpan}
                      scope="col"
                      className={`px-4 sm:px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider ${header.column.getCanSort() && "cursor-pointer"}`}
                      onClick={header.column.getToggleSortingHandler()}
                      style={{
                        width: header.getSize() !== 0 ? header.getSize() : undefined,
                      }}
                    >
                      {header.isPlaceholder ? null : (
                        <div
                          {...{
                            className: header.column.getCanSort()
                              ? 'cursor-pointer select-none'
                              : '',
                            onClick: header.column.getToggleSortingHandler(),
                          }}
                        >
                          {flexRender(
                            header.column.columnDef.header,
                            header.getContext()
                          )}
                          {{
                            asc: <TriangleUpIcon w={4} />,
                            desc: <TriangleDownIcon w={4} />
                          }[header.column.getIsSorted() as string] ?? null}
                        </div>
                      )}
                    </th>
                  )
                })}
              </tr>
            ))}
          </thead>
          <tbody className="bg-white divide-y divide-gray-200" ref={tbodyRef}>
            {table.getRowModel().rows.map(row => {
              return (
                <Fragment key={row.id}>
                  <tr onClick={(event) => onRowClick && onRowClick(row, event)} tabIndex={0} onKeyDown={(e) => handleKeyDown(e, row)} className={onRowClick && `hover:cursor-pointer hover:bg-gray-100`}>
                    {/* first row is a normal row */}
                    {row.getVisibleCells().map(cell => {
                      return (
                        <td key={cell.id} className="px-4 sm:px-6 py-4" style={{
                          width: cell.column.getSize(),
                        }}>
                          {flexRender(
                            cell.column.columnDef.cell,
                            cell.getContext()
                          )}
                        </td>
                      )
                    })}
                  </tr>
                  {row.getIsExpanded() && (
                    <tr>
                      {/* 2nd row is a custom 1 cell row */}
                      <td colSpan={row.getVisibleCells().length}>
                        {renderSubComponent && renderSubComponent({ row })}
                      </td>
                    </tr>
                  )}
                </Fragment>
              )
            })}
          </tbody>
        </table>
      </div>
    </div>
  )
}
export default Table;
