import React from 'react';
import { EmptyState, EmptyStateType } from '../EmptyState';
import { FilterControls, useFilters } from '../../../context/FilterContext';
import { SortingControls } from '../../../context/SortContext';
import Button from '../button/Button';
import { VirtualizedCardListWrapperProps } from './types';

/**
 * VirtualizedCardListWrapper - A reusable wrapper component for virtualized card lists
 *
 * Handles:
 * - Error state display
 * - Empty state display (with and without filters)
 * - Loading state display
 * - Header with title, sorting, and filtering controls
 * - Responsive layout
 *
 * @example
 * ```tsx
 * <VirtualizedCardListWrapper
 *   title="Tasks"
 *   totalItems={data?.tasks?.totalCount}
 *   loading={initialLoading}
 *   error={error}
 *   sortType={PageNavItem.tasks}
 *   showFiltering={true}
 *   cardList={
 *     <VirtualizedCardList
 *       items={taskIds}
 *       renderCard={renderCard}
 *       hasMore={hasMore}
 *       onLoadMore={loadMore}
 *     />
 *   }
 * />
 * ```
 */
export const VirtualizedCardListWrapper: React.FC<VirtualizedCardListWrapperProps> = ({
    totalItems,
    loading,
    error,
    title = "Cards",
    cardList,
    className = '',
    sortType,
    showFiltering = true,
}) => {
    const { filterCount, resetFilters } = useFilters();

    const renderContent = () => {
        if (error) {
            return (
                <EmptyState
                    type={EmptyStateType.error}
                    label="Error loading data"
                    details={error.message}
                />
            );
        }

        if (loading || totalItems === undefined) {
            return (
                <EmptyState
                    type={EmptyStateType.loading}
                    label="Loading data..."
                />
            );
        }

        if (totalItems === 0 && filterCount > 0) {
            return (
                <EmptyState
                    type={EmptyStateType.noMatches}
                    label="No data matching your filters"
                >
                    <Button
                        onClick={resetFilters}
                        buttonVariant="solid"
                        buttonStyle={{ color: "purple", size: "md" }}
                    >
                        Clear filters
                    </Button>
                </EmptyState>
            );
        }

        if (totalItems === 0) {
            return (
                <EmptyState
                    type={EmptyStateType.noData}
                    label="No data found"
                />
            );
        }

        return cardList;
    };

    return (
        <div className={`flex flex-col w-full ${className} gap-2`}>
            <div className="flex flex-row justify-between items-center border-b border-gray-200 bg-white gap-2 py-2 sticky top-0 z-5 shadow-sm">
                <div className='flex flex-row gap-2 items-center'>
                    <h3 className="text-xl font-semibold leading-6 text-gray-900 md:visible invisible">
                        {title}
                    </h3>
                    <p className='text-md text-gray-600'>{totalItems !== undefined && `(${totalItems})`}</p>
                </div>
                <div className="flex flex-row justify-end gap-2 w-full md:w-auto">
                    {sortType && <SortingControls sortType={sortType} />}
                    {showFiltering && <FilterControls />}
                </div>
            </div>

            {renderContent()}
        </div>
    );
};

export default VirtualizedCardListWrapper;
