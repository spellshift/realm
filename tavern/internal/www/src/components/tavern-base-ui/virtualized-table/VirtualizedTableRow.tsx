import { useQuery } from "@apollo/client";
import { useMemo } from "react";
import { renderCellSkeleton } from "./CellDefaultSkeleton";
import { VirtualizedTableRowProps } from "./types";

/**
 * Generic virtualized table row component that fetches and displays data
 *
 * This component abstracts the common pattern of:
 * - Fetching data per row with Apollo useQuery
 * - Polling when visible in viewport
 * - Showing loading skeletons
 * - Rendering custom column content
 *
 * @example
 * ```tsx
 * <VirtualizedTableRow
 *   itemId="host-123"
 *   query={GET_HOST_DETAIL_QUERY}
 *   getVariables={(id) => ({ id })}
 *   columns={[
 *     {
 *       key: 'name',
 *       gridWidth: 'minmax(250px,3fr)',
 *       render: (host) => <div>{host.name}</div>,
 *       renderSkeleton: () => <div className="h-4 bg-gray-200 rounded animate-pulse w-3/4" />
 *     }
 *   ]}
 *   extractData={(response) => response.hosts?.edges?.[0]?.node}
 *   onRowClick={(id) => console.log(id)}
 *   isVisible={true}
 * />
 * ```
 */
export function VirtualizedTableRow<TData, TVariables = Record<string, unknown>>({
    itemId,
    query,
    getVariables,
    columns,
    onRowClick,
    isVisible,
    pollInterval = 30000,
    extractData,
    className = "",
    minWidth = "800px",
}: VirtualizedTableRowProps<TData, TVariables>) {
    const queryVariables = useMemo(() => getVariables(itemId), [itemId, getVariables]);

    const { data } = useQuery(query, {
        variables: queryVariables,
        pollInterval: isVisible ? pollInterval : 0,
        fetchPolicy: 'cache-and-network',
    });

    const gridTemplateColumns = useMemo(
        () => columns.map(col => col.gridWidth).join(' '),
        [columns]
    );

    const baseClassName = `grid gap-4 px-6 py-4 border-b border-gray-200 bg-white hover:bg-gray-50 ${className}`;
    const clickableClassName = onRowClick ? "cursor-pointer" : "";

    if (!data) {
        return (
            <div
                className={baseClassName}
                style={{
                    gridTemplateColumns,
                    minWidth
                }}
            >
                {columns.map((column) => (
                    <div key={column.key}>
                        {renderCellSkeleton(column)}
                    </div>
                ))}
            </div>
        );
    }

    const itemData = extractData(data);

    if (!itemData) {
        return null;
    }

    const handleClick = () => {
        if (onRowClick) {
            onRowClick(itemId);
        }
    };

    return (
        <div
            className={`${baseClassName} ${clickableClassName}`}
            style={{
                gridTemplateColumns,
                minWidth
            }}
            onClick={handleClick}
        >
            {columns.map((column) => (
                <div key={column.key}>
                    {column.render(itemData)}
                </div>
            ))}
        </div>
    );
}
