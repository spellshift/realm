import { ApolloError, DocumentNode, OperationVariables } from "@apollo/client";
import { ReactNode } from "react";

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
 * Unified column definition that includes both header and cell configuration
 */
export interface VirtualizedTableColumn<TData> {
    /** Unique identifier for the column */
    key: string;
    /** Column header label */
    label: string;
    /** Grid width specification (e.g., "minmax(250px,3fr)") */
    width: string;
    /** Render function for the row of the column content */
    render: (data: TData) => ReactNode;
    /** Render function for the loading skeleton (optional - uses default if not provided) */
    renderSkeleton?: () => ReactNode;
}

/**
 * Configuration for expandable row behavior
 */
export interface ExpandableConfig<TData> {
    /** Render function for expanded content below the row */
    render: (data: TData) => ReactNode;
    /** Function to determine if this specific row is expandable based on its data (default: true) */
    isExpandable?: (data: TData) => boolean;
}

export interface VirtualizedTableProps<TData, TResponse = unknown> {
    /** Array of item IDs to display */
    items: string[];

    /** Column definitions including header labels, widths, and render functions */
    columns: VirtualizedTableColumn<TData>[];

    /** GraphQL query to fetch data for each row. Can be a function for different queries per row. */
    query: DocumentNode | ((itemId: string) => DocumentNode);

    /** Function to generate query variables from itemId */
    getVariables: (itemId: string) => OperationVariables;

    /** Function to extract the data node from the query response */
    extractData: (response: TResponse, itemId: string) => TData | null;

    /** Poll interval in milliseconds when row is visible (default: 5000) */
    pollInterval?: number;

    /** Callback when a row is clicked */
    onItemClick?: (id: string) => void;

    /** Whether there are more items to load */
    hasMore?: boolean;

    /** Callback to load more items */
    onLoadMore?: () => void;

    /** Configuration for expandable rows */
    expandable?: ExpandableConfig<TData>;

    /** Minimum width for horizontal scrolling (default: '800px') */
    minWidth?: string;

    /** Estimated height of each row in pixels */
    estimateRowSize?: number;

    /** Number of extra rows to render outside viewport */
    overscan?: number;

    /** CSS height calculation (default: calc(100vh - 180px)) */
    height?: string;

    /** Minimum height in pixels */
    minHeight?: string;
}

/**
 * Props for the internal row component that handles data fetching
 */
export interface VirtualizedTableRowInternalProps<TData, TResponse> {
    /** Unique identifier for this row's item */
    itemId: string;
    /** GraphQL query to fetch data for this row */
    query: DocumentNode;
    /** Function to generate query variables from itemId */
    getVariables: (itemId: string) => Record<string, unknown>;
    /** Function to extract the data node from the query response */
    extractData: (response: TResponse, itemId: string) => TData | null;
    /** Column definitions */
    columns: VirtualizedTableColumn<TData>[];
    /** Whether this row is currently visible in the viewport */
    isVisible: boolean;
    /** Poll interval in milliseconds when row is visible */
    pollInterval: number;
    /** Callback when the row is clicked */
    onRowClick?: (id: string) => void;
    /** Minimum width for horizontal scrolling */
    minWidth: string;
    /** Grid template columns for row layout */
    gridTemplateColumns: string;
    /** Whether the row is currently expanded */
    isExpanded: boolean;
    /** Callback to toggle row expansion */
    onToggleExpand: (id: string) => void;
    /** Configuration for expandable row behavior */
    expandable?: ExpandableConfig<TData>;
}
