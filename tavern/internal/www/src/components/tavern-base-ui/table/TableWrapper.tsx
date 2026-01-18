import React, { ReactNode } from 'react';
import { EmptyState, EmptyStateType } from '../EmptyState';
import { ApolloError } from '@apollo/client';

type TableWrapperProps = {
  totalItems?: number;
  loading: boolean;
  error: ApolloError | undefined;
  title?: string;
  filterControls?: ReactNode;
  sortingControls?: ReactNode;
  table: ReactNode;
  pagination: ReactNode;
  stickyControls?: boolean;
}

export const TableWrapper: React.FC<TableWrapperProps> = ({
  totalItems,
  loading,
  error,
  title = "Table",
  filterControls,
  sortingControls,
  table,
  pagination,
  stickyControls = true,
}) => {

  const renderTableContent = () => {
    if (error) {
      return <EmptyState type={EmptyStateType.error} label="Error loading data" />;
    }
    else if (loading || totalItems === undefined) {
      return <EmptyState type={EmptyStateType.loading} label="Loading data..." />;
    }
    else if (totalItems === 0) {
      return <EmptyState type={EmptyStateType.noData} label="No data found" />;
    }
    else {
      return (
        <>
          <div className="overflow-x-auto">
            {table}
          </div>
          {pagination}
        </>
      )
    }
  }

  return (
    <div className="flex flex-col w-full">
      {/* Controls Section */}
      {(filterControls || sortingControls) && (
        <div className={`
          flex flex-row justify-between items-center border-b border-gray-200 bg-white gap-2 pb-2
          ${stickyControls ? 'sticky top-0 z-5 shadow-sm' : ''}
        `}>
          <h3 className="text-xl font-semibold leading-6 text-gray-900 md:visible invisible">{title}</h3>
          <div className="flex flex-row justify-end gap-2 w-full md:w-auto">
            {sortingControls}
            {filterControls}
          </div>
        </div>
      )}
      {renderTableContent()}
    </div>
  );
};

export default TableWrapper;
