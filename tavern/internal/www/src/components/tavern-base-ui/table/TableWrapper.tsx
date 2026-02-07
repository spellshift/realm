import React, { ReactNode } from 'react';
import { EmptyState, EmptyStateType } from '../EmptyState';
import { ApolloError } from '@apollo/client';
import { FilterControls, useFilters } from '../../../context/FilterContext';
import { SortingControls } from '../../../context/SortContext';
import Button from '../button/Button';

type TableWrapperProps = {
  totalItems?: number;
  loading: boolean;
  error: ApolloError | undefined;
  title?: string;
  table: ReactNode;
  pagination?: ReactNode;
}

export const TableWrapper: React.FC<TableWrapperProps> = ({
  totalItems,
  loading,
  error,
  title = "Table",
  table,
  pagination,
}) => {
  const { filterCount, resetFilters } = useFilters();

  const renderTableContent = () => {
    if (error) {
      return <EmptyState type={EmptyStateType.error} label="Error loading data" />;
    }
    else if (loading || totalItems === undefined) {
      return <EmptyState type={EmptyStateType.loading} label="Loading data..." />;
    }
    else if (totalItems === 0 && filterCount > 0) {
      return (
        <EmptyState type={EmptyStateType.noMatches} label="No data matching your filters">
          <Button onClick={resetFilters} buttonVariant="solid" buttonStyle={{ color: "purple", size: "md" }}>
            Clear filters
          </Button>
        </EmptyState>
      );
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
          {pagination && pagination}
        </>
      )
    }
  }

  return (
    <div className="flex flex-col w-full">
      <div className={`
        flex flex-row justify-between items-center border-b border-gray-200 bg-white gap-2 py-2 sticky top-0 z-5 shadow-sm' : ''}
      `}>
        <h3 className="text-xl font-semibold leading-6 text-gray-900 md:visible invisible">{title}</h3>
        <div className="flex flex-row justify-end gap-2 w-full md:w-auto">
          <SortingControls />
          <FilterControls />
        </div>
      </div>
      {renderTableContent()}
    </div>
  );
};

export default TableWrapper;
