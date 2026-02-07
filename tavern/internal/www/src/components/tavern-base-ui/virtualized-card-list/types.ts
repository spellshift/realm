import { ApolloError, DocumentNode, OperationVariables } from "@apollo/client";
import { ReactNode } from "react";

export interface VirtualizedCardListWrapperProps {
    /** Total number of items (used to determine empty states) */
    totalItems?: number;

    /** Whether the initial data is loading */
    loading: boolean;

    /** Apollo error if query failed */
    error: ApolloError | undefined;

    /** Title displayed in the header */
    title?: string;

    /** The virtualized card list component to render */
    cardList: ReactNode;

    /** Optional className for custom styling */
    className?: string;

    /** Whether to show sorting controls */
    showSorting?: boolean;

    /** Whether to show filtering controls */
    showFiltering?: boolean;
}

export interface VirtualizedCardListProps {
    /** Array of item IDs to display */
    items: string[];

    /** Function to render each card */
    renderCard: (props: {
        itemId: string;
        isVisible: boolean;
    }) => ReactNode;

    /** Whether there are more items to load */
    hasMore?: boolean;

    /** Callback to load more items */
    onLoadMore?: () => void;

    /** Estimated height of each card in pixels */
    estimateCardSize?: number;

    /** Number of extra cards to render outside viewport */
    overscan?: number;

    /** CSS height calculation (default: calc(100vh - 180px)) */
    height?: string;

    /** Minimum height in pixels */
    minHeight?: string;

    /** Gap between cards in pixels */
    gap?: number;

    /** Padding around the card list */
    padding?: string;

    /** Enable dynamic sizing - measures actual element heights after render (default: true) */
    dynamicSizing?: boolean;
}

/**
 * Props for the VirtualizedCardItem component
 */
export interface VirtualizedCardItemProps<TData, TVariables = Record<string, unknown>> {
    /** Unique identifier for this card's item */
    itemId: string;
    /** GraphQL query to fetch data for this card */
    query: DocumentNode;
    /** Function to generate query variables from itemId */
    getVariables: (itemId: string) => OperationVariables | undefined;
    /** Render function for the card content */
    renderCard: (data: TData) => ReactNode;
    /** Render function for loading skeleton */
    renderSkeleton: () => ReactNode;
    /** Whether the card is currently visible in viewport */
    isVisible: boolean;
    /** Poll interval in milliseconds when visible (default: 30000) */
    pollInterval?: number;
    /** Function to extract the data node from the query response */
    extractData: (response: any) => TData | null;
    /** Custom CSS class for the card wrapper */
    className?: string;
}
