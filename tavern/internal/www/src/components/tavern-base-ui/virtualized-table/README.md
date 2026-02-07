# Virtualized Table Components

A collection of reusable components for building high-performance, virtualized tables with built-in state management and Apollo GraphQL integration.

## Components

### VirtualizedTableWrapper

A complete table wrapper component that handles all common table states and provides a consistent UI.

#### Features

- **Error state handling**: Displays error messages with icons
- **Empty state handling**: Shows appropriate messages for no data vs. no matches
- **Loading state**: Shows loading spinner during initial data fetch
- **Filter integration**: Detects active filters and provides clear button
- **Sorting controls**: Integrates with SortContext for sorting UI
- **Responsive layout**: Mobile-friendly header and controls

#### Usage

```tsx
import { VirtualizedTableWrapper, VirtualizedTable } from '@/components/virtualized-table';
import { useMyData } from './hooks';

const MyPage = () => {
    const { data, loading, error, itemIds, hasMore, loadMore } = useMyData();

    return (
        <VirtualizedTableWrapper
            title="My Items"
            totalItems={data?.items?.totalCount}
            loading={loading}
            error={error}
            showSorting={true}
            showFiltering={true}
            table={
                <VirtualizedTable
                    items={itemIds}
                    renderRow={renderRow}
                    renderHeader={renderHeader}
                    onItemClick={handleRowClick}
                    hasMore={hasMore}
                    onLoadMore={loadMore}
                />
            }
        />
    );
};
```

#### Props

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `title` | `string` | No | `"Table"` | Table title displayed in header |
| `totalItems` | `number` | No | - | Total number of items (for empty state detection) |
| `loading` | `boolean` | Yes | - | Whether initial data is loading |
| `error` | `ApolloError` | No | - | Apollo error object if query failed |
| `table` | `ReactNode` | Yes | - | The table component to render |
| `className` | `string` | No | `""` | Additional CSS classes |
| `showSorting` | `boolean` | No | `true` | Show sorting controls in header |
| `showFiltering` | `boolean` | No | `true` | Show filtering controls in header |

---

### VirtualizedTableRow

A reusable, generic virtualized table row component that abstracts common patterns for fetching and displaying row data with Apollo GraphQL.

#### Features

- **Lazy data fetching**: Each row fetches its own data using GraphQL queries
- **Visibility-based polling**: Only polls for updates when the row is visible in the viewport
- **Loading skeletons**: Customizable skeleton states that match your content structure
- **Type-safe**: Full TypeScript generics support for data and variables
- **Flexible columns**: Define custom render functions for each column
- **Click handling**: Optional row click callbacks

## Basic Usage

```tsx
import { VirtualizedTableRow, VirtualizedTableColumn } from '@/components/virtualized-table';
import { GET_ITEM_QUERY } from './queries';

const columns: VirtualizedTableColumn<ItemData>[] = [
  {
    key: 'name',
    gridWidth: 'minmax(250px, 3fr)',
    render: (item) => <div>{item.name}</div>,
    renderSkeleton: () => (
      <div className="h-4 bg-gray-200 rounded animate-pulse w-3/4" />
    ),
  },
  {
    key: 'status',
    gridWidth: 'minmax(120px, 1fr)',
    render: (item) => <Badge>{item.status}</Badge>,
    renderSkeleton: () => (
      <div className="h-6 bg-gray-200 rounded animate-pulse w-12" />
    ),
  },
];

<VirtualizedTableRow
  itemId="item-123"
  query={GET_ITEM_QUERY}
  getVariables={(id) => ({ id })}
  columns={columns}
  extractData={(response) => response.items?.edges?.[0]?.node}
  onRowClick={(id) => navigate(`/items/${id}`)}
  isVisible={true}
/>
```

## API Reference

### VirtualizedTableRow Props

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `itemId` | `string` | Yes | - | Unique identifier for this row's item |
| `query` | `DocumentNode` | Yes | - | GraphQL query to fetch data |
| `getVariables` | `(itemId: string) => TVariables` | Yes | - | Function to generate query variables |
| `columns` | `VirtualizedTableColumn<TData>[]` | Yes | - | Column definitions |
| `extractData` | `(response: any) => TData \| null` | Yes | - | Extract data node from query response |
| `onRowClick` | `(itemId: string) => void` | No | - | Callback when row is clicked |
| `isVisible` | `boolean` | Yes | - | Whether row is visible in viewport |
| `pollInterval` | `number` | No | `30000` | Poll interval in ms when visible |
| `className` | `string` | No | `""` | Custom CSS class for the row |
| `minWidth` | `string` | No | `"800px"` | Minimum width for the row |

### VirtualizedTableColumn

| Property | Type | Description |
|----------|------|-------------|
| `key` | `string` | Unique identifier for the column |
| `gridWidth` | `string` | CSS grid width (e.g., `"minmax(250px,3fr)"`) |
| `render` | `(data: TData) => ReactNode` | Render function for column content |
| `renderSkeleton` | `() => ReactNode` | Render function for loading skeleton |

---

### VirtualizedTable

The core virtualized table component using `@tanstack/react-virtual` for efficient rendering of large lists.

#### Features

- **Efficient virtualization**: Only renders visible rows plus overscan
- **Infinite scrolling**: Automatic load-more detection
- **Viewport tracking**: Tracks which items are actually visible
- **Customizable rendering**: Flexible row and header rendering

#### Usage

```tsx
import { VirtualizedTable, VirtualizedTableHeader } from '@/components/virtualized-table';

const MyTable = ({ itemIds, hasMore, onLoadMore }) => {
    const columns = ["Name", "Status", "Created At"];
    const gridCols = "minmax(200px,2fr) minmax(100px,1fr) minmax(150px,1fr)";

    const renderHeader = () => (
        <VirtualizedTableHeader
            columns={columns}
            gridCols={gridCols}
            minWidth="600px"
        />
    );

    const renderRow = ({ itemId, isVisible, onItemClick }) => (
        <MyRowComponent
            itemId={itemId}
            onRowClick={onItemClick}
            isVisible={isVisible}
        />
    );

    return (
        <VirtualizedTable
            items={itemIds}
            renderRow={renderRow}
            renderHeader={renderHeader}
            onItemClick={(id) => navigate(`/items/${id}`)}
            hasMore={hasMore}
            onLoadMore={onLoadMore}
            estimateRowSize={73}
            overscan={5}
        />
    );
};
```

#### Props

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `items` | `string[]` | Yes | - | Array of item IDs to display |
| `renderRow` | `function` | Yes | - | Function to render each row |
| `renderHeader` | `function` | Yes | - | Function to render table header |
| `onItemClick` | `function` | No | - | Callback when row is clicked |
| `hasMore` | `boolean` | No | `false` | Whether more items can be loaded |
| `onLoadMore` | `function` | No | - | Callback to load more items |
| `estimateRowSize` | `number` | No | `73` | Estimated height of each row (px) |
| `overscan` | `number` | No | `5` | Number of extra rows to render |
| `emptyMessage` | `string` | No | `"No items found"` | Message when items array is empty |
| `height` | `string` | No | `"calc(100vh - 180px)"` | Container height CSS value |
| `minHeight` | `string` | No | `"400px"` | Minimum container height |

---

### VirtualizedTableHeader

A simple header component for grid-based table layouts with sticky positioning.

#### Usage

```tsx
<VirtualizedTableHeader
    columns={["Name", "Status", "Created At"]}
    gridCols="minmax(200px,2fr) 1fr minmax(150px,1fr)"
    minWidth="600px"
/>
```

#### Props

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `columns` | `string[]` | Yes | - | Column labels to display |
| `gridCols` | `string` | Yes | - | CSS grid template columns value |
| `minWidth` | `string` | No | `"800px"` | Minimum width for horizontal scrolling |
| `className` | `string` | No | `""` | Additional CSS classes |

---

## Complete Example

Here's a full implementation showing how all components work together:

### 1. Page Component with VirtualizedTableWrapper

```tsx
// pages/quests/Quests.tsx
import Breadcrumbs from "../../components/Breadcrumbs";
import { VirtualizedTableWrapper } from "../../components/virtualized-table";
import { QuestsTable } from "./QuestsTable";
import { useQuestIds } from "./useQuestIds";

const Quests = () => {
    const {
        data,
        questIds,
        loading,
        error,
        hasMore,
        loadMore,
    } = useQuestIds();

    return (
        <>
            <Breadcrumbs
                pages={[{
                    label: "Quests",
                    link: "/quests"
                }]}
            />
            <VirtualizedTableWrapper
                title="Quests"
                totalItems={data?.quests?.totalCount}
                loading={loading}
                error={error}
                showSorting={true}
                showFiltering={true}
                table={
                    <QuestsTable
                        questIds={questIds}
                        hasMore={hasMore}
                        onLoadMore={loadMore}
                    />
                }
            />
        </>
    );
}

export default Quests;
```

### 2. Table Component

```tsx
// pages/quests/QuestsTable.tsx
import { useCallback, useMemo } from "react";
import { useNavigate } from "react-router-dom";
import { VirtualizedTableHeader, VirtualizedTable } from "../../components/virtualized-table";
import { QuestRowVirtualized } from "./QuestRowVirtualized";

interface QuestsTableProps {
    questIds: string[];
    hasMore?: boolean;
    onLoadMore?: () => void;
}

export const QuestsTable = ({ questIds, hasMore = false, onLoadMore }: QuestsTableProps) => {
    const navigate = useNavigate();

    const handleRowClick = useCallback((questId: string) => {
        navigate(`/quests/${questId}`);
    }, [navigate]);

    const columns = useMemo(() => [
        "Quest Name",
        "Status",
        "Tasks",
        "Created At"
    ], []);

    const gridCols = "minmax(250px,3fr) minmax(120px,1fr) minmax(100px,1fr) minmax(150px,1fr)";

    const renderHeader = useCallback(() => (
        <VirtualizedTableHeader
            columns={columns}
            gridCols={gridCols}
            minWidth="800px"
        />
    ), [columns]);

    const renderRow = useCallback(({ itemId, isVisible, onItemClick }: {
        itemId: string;
        isVisible: boolean;
        onItemClick: (id: string) => void;
    }) => (
        <QuestRowVirtualized
            questId={itemId}
            onRowClick={onItemClick}
            isVisible={isVisible}
        />
    ), []);

    return (
        <VirtualizedTable
            items={questIds}
            renderRow={renderRow}
            renderHeader={renderHeader}
            onItemClick={handleRowClick}
            hasMore={hasMore}
            onLoadMore={onLoadMore}
            estimateRowSize={73}
            overscan={5}
            emptyMessage="No quests found"
            height="calc(100vh - 180px)"
            minHeight="400px"
        />
    );
};
```

### 3. Custom Row Component

```tsx
// pages/quests/QuestRowVirtualized.tsx
import { useQuery, gql } from "@apollo/client";
import { formatDistanceToNow } from "date-fns";

const GET_QUEST = gql`
  query GetQuest($questId: ID!) {
    quest(id: $questId) {
      id
      name
      status
      taskCount
      createdAt
    }
  }
`;

interface QuestRowVirtualizedProps {
    questId: string;
    onRowClick: (questId: string) => void;
    isVisible: boolean;
}

export const QuestRowVirtualized = ({
    questId,
    onRowClick,
    isVisible
}: QuestRowVirtualizedProps) => {
    const { data, loading } = useQuery(GET_QUEST, {
        variables: { questId },
        pollInterval: isVisible ? 30000 : 0,
    });

    const quest = data?.quest;
    const gridCols = "minmax(250px,3fr) minmax(120px,1fr) minmax(100px,1fr) minmax(150px,1fr)";

    if (loading || !quest) {
        return (
            <div
                className="grid gap-4 px-6 py-4 border-b border-gray-200"
                style={{ gridTemplateColumns: gridCols, minWidth: '800px' }}
            >
                <div className="h-4 bg-gray-200 rounded animate-pulse w-3/4" />
                <div className="h-4 bg-gray-200 rounded animate-pulse w-1/2" />
                <div className="h-4 bg-gray-200 rounded animate-pulse w-1/3" />
                <div className="h-4 bg-gray-200 rounded animate-pulse w-2/3" />
            </div>
        );
    }

    return (
        <div
            className="grid gap-4 px-6 py-4 border-b border-gray-200 hover:bg-gray-50 cursor-pointer"
            style={{ gridTemplateColumns: gridCols, minWidth: '800px' }}
            onClick={() => onRowClick(questId)}
        >
            <div className="font-medium text-gray-900">{quest.name}</div>
            <div className="text-sm text-gray-500">{quest.status}</div>
            <div className="text-sm text-gray-500">{quest.taskCount}</div>
            <div className="text-sm text-gray-500">
                {formatDistanceToNow(new Date(quest.createdAt), { addSuffix: true })}
            </div>
        </div>
    );
};
```

### Key Features Demonstrated

1. **VirtualizedTableWrapper** handles all states (loading, error, empty, no matches)
2. **Sorting & Filtering** are integrated automatically via context
3. **Infinite scrolling** with `hasMore` and `loadMore` props
4. **Conditional polling** - rows only poll when visible in viewport
5. **Loading skeletons** provide smooth UX during data fetch
6. **Navigation** on row click for detailed views
7. **Responsive grid layout** with consistent column widths

For the real implementation reference, see [Hosts.tsx](/workspaces/realm/tavern/internal/www/src/pages/hosts/Hosts.tsx) and [HostsTable.tsx](/workspaces/realm/tavern/internal/www/src/pages/hosts/HostsTable.tsx).

## Design Principles

1. **Single Responsibility**: Each component handles one concern
2. **Performance**: Virtualization and conditional polling reduce overhead
3. **Flexibility**: Fully customizable rendering for all components
4. **Type Safety**: Generic types ensure compile-time correctness
5. **Separation of Concerns**: Domain logic stays in parent components
6. **Reusability**: Consistent patterns across all table implementations
