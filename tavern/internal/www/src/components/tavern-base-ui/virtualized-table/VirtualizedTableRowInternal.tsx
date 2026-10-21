import { useMemo } from "react";
import { useQuery } from "@apollo/client";
import { ChevronDown, ChevronRight } from "lucide-react";
import { VirtualizedTableRowInternalProps, VirtualizedTableRowStaticProps } from "./types";
import { renderCellSkeleton } from "./CellDefaultSkeleton";

interface VirtualizedTableRowContentProps<TData> {
    itemId: string;
    itemData: TData;
    columns: VirtualizedTableRowStaticProps<TData>["columns"];
    onRowClick?: (id: string, data: TData | null) => void;
    minWidth: string;
    gridTemplateColumns: string;
    isExpanded: boolean;
    onToggleExpand: (id: string) => void;
    expandable?: VirtualizedTableRowStaticProps<TData>["expandable"];
}

function VirtualizedTableRowSkeleton<TData>({
    columns,
    minWidth,
    gridTemplateColumns,
    supportsExpand,
}: {
    columns: VirtualizedTableRowStaticProps<TData>["columns"];
    minWidth: string;
    gridTemplateColumns: string;
    supportsExpand: boolean;
}) {
    const fullGridTemplateColumns = supportsExpand ? `32px ${gridTemplateColumns}` : gridTemplateColumns;
    const rowClassName = "grid gap-2 px-4 py-4 border-b border-gray-200 bg-white hover:bg-gray-50";

    return (
        <div
            className={rowClassName}
            style={{ gridTemplateColumns: fullGridTemplateColumns, minWidth }}
        >
            {supportsExpand && <div />}
            {columns.map((column) => (
                <div key={column.key}>
                    {renderCellSkeleton(column)}
                </div>
            ))}
        </div>
    );
}

function VirtualizedTableRowContent<TData>({
    itemId,
    itemData,
    columns,
    onRowClick,
    minWidth,
    gridTemplateColumns,
    isExpanded,
    onToggleExpand,
    expandable,
}: VirtualizedTableRowContentProps<TData>) {
    const supportsExpand = expandable !== undefined;
    const fullGridTemplateColumns = supportsExpand ? `32px ${gridTemplateColumns}` : gridTemplateColumns;
    const rowClassName = "grid gap-2 px-4 py-4 border-b border-gray-200 bg-white hover:bg-gray-50";

    const canExpand = expandable?.isExpandable?.(itemData) ?? true;

    const handleExpandClick = (e: React.MouseEvent) => {
        e.stopPropagation();
        onToggleExpand(itemId);
    };

    const handleKeyDown = (e: React.KeyboardEvent) => {
        if (onRowClick && (e.key === "Enter" || e.key === " ")) {
            e.preventDefault();
            onRowClick(itemId, itemData);
        }
    };

    return (
        <div>
            <div
                className={`${rowClassName} ${onRowClick ? "cursor-pointer focus:outline-none focus:ring-2 focus:ring-inset focus:ring-purple-500" : ""}`}
                style={{ gridTemplateColumns: fullGridTemplateColumns, minWidth }}
                onClick={() => onRowClick?.(itemId, itemData)}
                onKeyDown={handleKeyDown}
                tabIndex={onRowClick ? 0 : undefined}
                role={onRowClick ? "button" : undefined}
                aria-label={onRowClick ? `Select row` : undefined}
            >
                {supportsExpand && (
                    canExpand ? (
                        <button
                            type="button"
                            className="flex items-center justify-center cursor-pointer"
                            onClick={handleExpandClick}
                            aria-expanded={isExpanded}
                            aria-label={isExpanded ? "Collapse row" : "Expand row"}
                        >
                            {isExpanded ? (
                                <ChevronDown className="w-4 text-gray-500" />
                            ) : (
                                <ChevronRight className="w-4 text-gray-500" />
                            )}
                        </button>
                    ) : <div />
                )}
                {columns.map((column) => (
                    <div key={column.key} className="text-sm text-gray-700">
                        {column.render(itemData) ?? "-"}
                    </div>
                ))}
            </div>
            {isExpanded && canExpand && expandable && (
                <div className="bg-gray-50 border-b border-gray-200" style={{ minWidth }}>
                    {expandable.render(itemData)}
                </div>
            )}
        </div>
    );
}

export function VirtualizedTableRowStatic<TData>({
    itemId,
    data,
    columns,
    onRowClick,
    minWidth,
    gridTemplateColumns,
    isExpanded,
    onToggleExpand,
    expandable,
}: VirtualizedTableRowStaticProps<TData>) {
    if (!data) {
        return (
            <VirtualizedTableRowSkeleton
                columns={columns}
                minWidth={minWidth}
                gridTemplateColumns={gridTemplateColumns}
                supportsExpand={expandable !== undefined}
            />
        );
    }

    return (
        <VirtualizedTableRowContent
            itemId={itemId}
            itemData={data}
            columns={columns}
            onRowClick={onRowClick}
            minWidth={minWidth}
            gridTemplateColumns={gridTemplateColumns}
            isExpanded={isExpanded}
            onToggleExpand={onToggleExpand}
            expandable={expandable}
        />
    );
}

export function VirtualizedTableRowInternal<TData, TResponse>({
    itemId,
    query,
    getVariables,
    extractData,
    columns,
    isVisible,
    pollInterval,
    onRowClick,
    minWidth,
    gridTemplateColumns,
    isExpanded,
    onToggleExpand,
    expandable,
}: VirtualizedTableRowInternalProps<TData, TResponse>) {
    const variables = useMemo(() => getVariables(itemId), [itemId, getVariables]);

    const { data } = useQuery<TResponse>(query, {
        variables,
        pollInterval: isVisible ? pollInterval : 0,
        fetchPolicy: 'cache-and-network',
    });

    if (!data) {
        return (
            <VirtualizedTableRowSkeleton
                columns={columns}
                minWidth={minWidth}
                gridTemplateColumns={gridTemplateColumns}
                supportsExpand={expandable !== undefined}
            />
        );
    }

    const itemData = extractData(data, itemId);
    if (!itemData) {
        return null;
    }

    return (
        <VirtualizedTableRowContent
            itemId={itemId}
            itemData={itemData}
            columns={columns}
            onRowClick={onRowClick}
            minWidth={minWidth}
            gridTemplateColumns={gridTemplateColumns}
            isExpanded={isExpanded}
            onToggleExpand={onToggleExpand}
            expandable={expandable}
        />
    );
}
