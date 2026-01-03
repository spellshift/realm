import React, { ReactNode } from 'react';
import { EmptyState, EmptyStateType } from '../EmptyState';
import { ApolloError } from '@apollo/client';

type TableWrapperProps = {
  totalItems: null | number;
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
    else if (loading || !totalItems) {
      return <EmptyState type={EmptyStateType.loading} label="Loading data..." />;
    }
    else if (totalItems === 0) {
      return <EmptyState type={EmptyStateType.noData} label="No data found" />;
    }
    else {
      return (
        <>
          <div className="px-4 sm:px-6 xl:px-8">
            {table}
          </div>
          {pagination}
        </>
      )
    }
  }

  return (
    <div className="flex flex-col justify-center items-center gap-6">
      <div className="flex flex-col w-full -mx-4 sm:-mx-6 xl:-mx-8">
        {/* Controls Section */}
        {(filterControls || sortingControls) && (
          <div className={`
            flex flex-row justify-between items-center
            px-4 sm:px-6 xl:px-8 py-2 border-b border-gray-200 bg-white gap-2
            ${stickyControls ? 'sticky top-0 z-20 shadow-sm' : ''}
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
    </div>
  );
};

export default TableWrapper;
