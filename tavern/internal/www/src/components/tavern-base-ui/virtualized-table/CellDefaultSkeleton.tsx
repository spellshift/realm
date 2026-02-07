import { VirtualizedTableColumn } from "./types";

/**
 * Default skeleton component for loading states
 * Renders an animated pulse bar that fills the available width
 */
export const CellDefaultSkeleton = () => (
    <div className="flex items-center min-w-0">
        <div className="h-4 bg-gray-200 rounded animate-pulse w-3/4"></div>
    </div>
);

/**
 * Renders the skeleton for a column, falling back to default if not provided
 */
export const renderCellSkeleton = <TData,>(column: VirtualizedTableColumn<TData>) => {
    return column.renderSkeleton ? column.renderSkeleton() : <CellDefaultSkeleton />;
};
