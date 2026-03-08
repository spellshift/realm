import { useMemo } from "react";
import { useQuery } from "@apollo/client";
import { ChevronDown, ChevronRight } from "lucide-react";
import { VirtualizedTableRowInternalProps } from "./types";
import { renderCellSkeleton } from "./CellDefaultSkeleton";

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

    const supportsExpand = expandable !== undefined;
    const fullGridTemplateColumns = supportsExpand ? `32px ${gridTemplateColumns}` : gridTemplateColumns;
    const rowClassName = `grid gap-4 px-6 py-4 border-b border-gray-200 bg-white hover:bg-gray-50`;

    if (!data) {
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

    const itemData = extractData(data, itemId);
    if (!itemData) {
        return null;
    }

    const canExpand = expandable?.isExpandable?.(itemData) ?? true;

    const handleExpandClick = (e: React.MouseEvent) => {
        e.stopPropagation();
        onToggleExpand(itemId);
    };

    return (
        <div>
            <div
                className={`${rowClassName} ${onRowClick ? "cursor-pointer" : ""}`}
                style={{ gridTemplateColumns: fullGridTemplateColumns, minWidth }}
                onClick={() => onRowClick?.(itemId)}
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
