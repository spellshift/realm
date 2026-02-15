# Virtualized Table Components

A high-performance virtualized table component with unified column definitions and built-in Apollo GraphQL integration.

## Overview

The `VirtualizedTable` component provides:
- **Unified column definitions** - Define label, width, and render in one place
- **Built-in data fetching** - Each row fetches its own data with visibility-based polling
- **Expandable rows** - Optional expand/collapse functionality
- **Infinite scrolling** - Automatic load-more detection
- **Type-safe** - Full TypeScript generics support

## Quick Start

```tsx
import { VirtualizedTable, VirtualizedTableColumn } from '@/components/virtualized-table';
import { GET_HOST_QUERY } from './queries';

interface HostData {
    id: string;
    name: string;
    status: string;
}

const columns: VirtualizedTableColumn<HostData>[] = [
    {
        key: 'name',
        label: 'Host Name',
        width: 'minmax(250px, 3fr)',
        render: (host) => <div className="font-medium">{host.name}</div>,
        renderSkeleton: () => (
            <div className="h-4 bg-gray-200 rounded animate-pulse w-3/4" />
        ),
    },
    {
        key: 'status',
        label: 'Status',
        width: 'minmax(120px, 1fr)',
        render: (host) => <Badge>{host.status}</Badge>,
    },
];

const HostsTable = ({ hostIds }) => (
    <VirtualizedTable<HostData, HostQueryResponse>
        items={hostIds}
        columns={columns}
        query={GET_HOST_QUERY}
        getVariables={(id) => ({ id })}
        extractData={(response) => response?.hosts?.edges?.[0]?.node}
        onItemClick={(id) => navigate(`/hosts/${id}`)}
    />
);
```

## API Reference

### VirtualizedTable Props

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `items` | `string[]` | Yes | - | Array of item IDs to display |
| `columns` | `VirtualizedTableColumn<TData>[]` | Yes | - | Column definitions |
| `query` | `DocumentNode \| (id: string) => DocumentNode` | Yes | - | GraphQL query (can be dynamic per row) |
| `getVariables` | `(id: string) => OperationVariables` | Yes | - | Generate query variables from item ID |
| `extractData` | `(response: TResponse, id: string) => TData \| null` | Yes | - | Extract data from query response |
| `pollInterval` | `number` | No | `5000` | Poll interval (ms) when row is visible |
| `onItemClick` | `(id: string) => void` | No | - | Callback when row is clicked |
| `hasMore` | `boolean` | No | `false` | Whether more items can be loaded |
| `onLoadMore` | `() => void` | No | - | Callback to load more items |
| `expandable` | `ExpandableConfig<TData>` | No | - | Configuration for expandable rows |
| `minWidth` | `string` | No | `"800px"` | Minimum table width |
| `estimateRowSize` | `number` | No | `73` | Estimated row height (px) |
| `overscan` | `number` | No | `5` | Extra rows to render outside viewport |
| `height` | `string` | No | `"calc(100vh - 180px)"` | Container height |
| `minHeight` | `string` | No | `"400px"` | Minimum container height |

### VirtualizedTableColumn

| Property | Type | Required | Description |
|----------|------|----------|-------------|
| `key` | `string` | Yes | Unique identifier for the column |
| `label` | `string` | Yes | Column header text |
| `width` | `string` | Yes | CSS grid width (e.g., `"minmax(250px,3fr)"`) |
| `render` | `(data: TData) => ReactNode` | Yes | Render function for cell content |
| `renderSkeleton` | `() => ReactNode` | No | Render function for loading skeleton |

### ExpandableConfig

| Property | Type | Required | Description |
|----------|------|----------|-------------|
| `render` | `(data: TData) => ReactNode` | Yes | Render expanded content |
| `isExpandable` | `(data: TData) => boolean` | No | Determine if row can expand (default: true) |

## Features

### Dynamic Queries

For tables where different rows need different queries:

```tsx
<VirtualizedTable
    items={repositoryIds}
    query={(itemId) =>
        itemId === 'first-party'
            ? GET_FIRST_PARTY_QUERY
            : GET_REPOSITORY_QUERY
    }
    getVariables={(itemId) =>
        itemId === 'first-party' ? {} : { id: itemId }
    }
    extractData={(response, itemId) => {
        if (itemId === 'first-party') {
            return extractFirstPartyData(response);
        }
        return extractRepositoryData(response);
    }}
    // ...
/>
```

### Expandable Rows

```tsx
<VirtualizedTable
    items={itemIds}
    columns={columns}
    expandable={{
        render: (data) => (
            <div className="p-4">
                <DetailView data={data} />
            </div>
        ),
        isExpandable: (data) => data.hasDetails,
    }}
    // ...
/>
```

### Infinite Scrolling

```tsx
const { data, hasMore, loadMore } = useMyQuery();

<VirtualizedTable
    items={itemIds}
    columns={columns}
    hasMore={hasMore}
    onLoadMore={loadMore}
    // ...
/>
```

## Complete Example

```tsx
import { useCallback, useMemo } from "react";
import { useNavigate } from "react-router-dom";
import { formatDistance } from "date-fns";
import { VirtualizedTable, VirtualizedTableColumn } from "@/components/virtualized-table";
import { GET_HOST_DETAIL_QUERY } from "./queries";
import { HostDetailQueryResponse } from "./types";
import { HostNode } from "@/utils/interfacesQuery";

interface HostsTableProps {
    hostIds: string[];
    hasMore?: boolean;
    onLoadMore?: () => void;
}

export const HostsTable = ({ hostIds, hasMore = false, onLoadMore }: HostsTableProps) => {
    const navigate = useNavigate();
    const currentDate = useMemo(() => new Date(), []);

    const columns: VirtualizedTableColumn<HostNode>[] = useMemo(() => [
        {
            key: 'host-details',
            label: 'Host details',
            width: 'minmax(250px,3fr)',
            render: (host) => (
                <div className="flex flex-col">
                    <span className="font-medium">{host.name}</span>
                    <span className="text-sm text-gray-500">{host.primaryIP}</span>
                </div>
            ),
            renderSkeleton: () => (
                <div className="space-y-2">
                    <div className="h-4 bg-gray-200 rounded animate-pulse w-3/4" />
                    <div className="h-3 bg-gray-200 rounded animate-pulse w-1/2" />
                </div>
            ),
        },
        {
            key: 'last-seen',
            label: 'Last seen',
            width: 'minmax(120px,1fr)',
            render: (host) => (
                <span className="text-gray-600">
                    {host.lastSeenAt
                        ? formatDistance(new Date(host.lastSeenAt), currentDate)
                        : 'N/A'
                    }
                </span>
            ),
        },
    ], [currentDate]);

    return (
        <VirtualizedTable<HostNode, HostDetailQueryResponse>
            items={hostIds}
            columns={columns}
            query={GET_HOST_DETAIL_QUERY}
            getVariables={useCallback((id) => ({ id }), [])}
            extractData={useCallback((response) =>
                response?.hosts?.edges?.[0]?.node || null
            , [])}
            onItemClick={useCallback((id) => navigate(`/hosts/${id}`), [navigate])}
            hasMore={hasMore}
            onLoadMore={onLoadMore}
        />
    );
};
```

## VirtualizedTableWrapper

For complete table views with loading, error, and empty states:

```tsx
import { VirtualizedTableWrapper, VirtualizedTable } from '@/components/virtualized-table';

const HostsPage = () => {
    const { data, hostIds, loading, error, hasMore, loadMore } = useHostsQuery();

    return (
        <VirtualizedTableWrapper
            title="Hosts"
            totalItems={data?.hosts?.totalCount}
            loading={loading}
            error={error}
            showSorting={true}
            showFiltering={true}
            table={
                <HostsTable
                    hostIds={hostIds}
                    hasMore={hasMore}
                    onLoadMore={loadMore}
                />
            }
        />
    );
};
```

## Design Principles

1. **Single Source of Truth** - Column definitions include label, width, and render in one place
2. **Performance** - Virtualization and visibility-based polling minimize overhead
3. **Type Safety** - Full TypeScript generics ensure compile-time correctness
4. **Flexibility** - Support for dynamic queries, expandable rows, and custom rendering
5. **Simplicity** - Clean API that handles common patterns automatically
