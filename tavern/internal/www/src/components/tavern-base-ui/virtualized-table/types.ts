import { ApolloError, DocumentNode, OperationVariables } from "@apollo/client";
import { ReactNode } from "react";

export interface VirtualizedTableHeaderProps {
    /** Column labels to display */
    columns: string[];

    /** Grid template columns (e.g., "minmax(200px,2fr) 1fr 100px") */
    gridCols: string;

    /** Minimum width for horizontal scrolling */
    minWidth?: string;

    /** Additional CSS class names */
    className?: string;
}

export interface VirtualizedTableWrapperProps {
    /** Total number of items (used to determine empty states) */
    totalItems?: number;

    /** Whether the initial data is loading */
    loading: boolean;

    /** Apollo error if query failed */
    error: ApolloError | undefined;

    /** Title displayed in the header */
    title?: string;

    /** The virtualized table component to render */
    table: ReactNode;

    /** Optional className for custom styling */
    className?: string;

    /** Whether to show sorting controls */
    showSorting?: boolean;

    /** Whether to show filtering controls */
    showFiltering?: boolean;
}

/**
 * Defines a column in the virtualized table
 */
export interface VirtualizedTableColumn<TData> {
    /** Unique identifier for the column */
    key: string;
    /** Grid width specification (e.g., "minmax(250px,3fr)") */
    gridWidth: string;
    /** Render function for the column content */
    render: (data: TData) => ReactNode;
    /** Render function for the loading skeleton (optional - uses default if not provided) */
    renderSkeleton?: () => ReactNode;
}

export interface VirtualizedTableProps {
    /** Array of item IDs to display */
    items: string[];

    /** Function to render each row */
    renderRow: (props: {
        itemId: string;
        isVisible: boolean;
        onItemClick: (id: string) => void;
        isExpanded: boolean;
        onToggleExpand: (id: string) => void;
    }) => ReactNode;

    /** Function to render the table header */
    renderHeader: () => ReactNode;

    /** Callback when a row is clicked */
    onItemClick?: (id: string) => void;

    /** Whether there are more items to load */
    hasMore?: boolean;

    /** Callback to load more items */
    onLoadMore?: () => void;

    /** Estimated height of each row in pixels */
    estimateRowSize?: number;

    /** Number of extra rows to render outside viewport */
    overscan?: number;

    /** CSS height calculation (default: calc(100vh - 180px)) */
    height?: string;

    /** Minimum height in pixels */
    minHeight?: string;

    /** Enable dynamic sizing - measures actual element heights after render (default: false) */
    dynamicSizing?: boolean;

    /** Set of expanded item IDs (for expandable rows) */
    expandedItems?: Set<string>;

    /** Callback when an item's expand state is toggled */
    onToggleExpand?: (id: string) => void;
}

/**
 * Props for the VirtualizedTableRow component
 */
export interface VirtualizedTableRowProps<TData, TResponse = unknown> {
    /** Unique identifier for this row's item */
    itemId: string;
    /** GraphQL query to fetch data for this row */
    query: DocumentNode;
    /** Function to generate query variables from itemId */
    getVariables: (itemId: string) => OperationVariables | undefined;
    /** Function to extract the data node from the query response */
    extractData: (response: TResponse) => TData | null;
    /** Whether the row is currently visible in viewport */
    isVisible: boolean;
    /** Poll interval in milliseconds when visible (default: 10000) */
    pollInterval?: number;
    /** Column definitions */
    columns: VirtualizedTableColumn<TData>[];
    /** Callback when row is clicked */
    onRowClick?: (itemId: string) => void;
    /** Custom CSS class for the row */
    className?: string;
    /** Minimum width for the row (default: '800px') */
    minWidth?: string;
    /** Whether the row is currently expanded */
    isExpanded?: boolean;
    /** Callback when the expand toggle is clicked */
    onToggleExpand?: (itemId: string) => void;
    /** Function to render expanded content below the row */
    renderExpandedContent?: (data: TData) => ReactNode;
    /** Function to determine if this specific row is expandable based on its data (default: true if renderExpandedContent is provided) */
    isExpandable?: (data: TData) => boolean;
}
