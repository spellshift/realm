import { useQuery } from "@apollo/client";
import { useMemo } from "react";
import { renderCellSkeleton } from "./CellDefaultSkeleton";
import { VirtualizedTableRowProps } from "./types";
import { ChevronDown, ChevronRight } from "lucide-react";

export function VirtualizedTableRow<TData, TResponse = unknown>({
    itemId,
    query,
    getVariables,
    columns,
    onRowClick,
    isVisible,
    pollInterval = 5000,
    extractData,
    className = "",
    minWidth = "800px",
    isExpanded = false,
    onToggleExpand,
    renderExpandedContent,
    isExpandable,
}: VirtualizedTableRowProps<TData, TResponse>) {
    const variables = useMemo(() => getVariables(itemId), [itemId, getVariables]);

    const { data } = useQuery<TResponse>(query, {
        variables,
        pollInterval: isVisible ? pollInterval : 0,
        fetchPolicy: 'cache-and-network',
    });

    const gridTemplateColumns = useMemo(
        () => columns.map(col => col.gridWidth).join(' '),
        [columns]
    );

    const supportsExpand = renderExpandedContent !== undefined;
    const fullGridTemplateColumns = supportsExpand ? `32px ${gridTemplateColumns}` : gridTemplateColumns;
    const rowClassName = `grid gap-4 px-6 py-4 border-b border-gray-200 bg-white hover:bg-gray-50 ${className}`;

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

    const itemData = extractData(data);
    if (!itemData) {
        return null;
    }

    const canExpand = isExpandable?.(itemData) ?? true;

    const handleExpandClick = (e: React.MouseEvent) => {
        e.stopPropagation();
        onToggleExpand?.(itemId);
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
                    <div key={column.key}>
                        {column.render(itemData)}
                    </div>
                ))}
            </div>
            {isExpanded && canExpand && renderExpandedContent && (
                <div className="bg-gray-50 border-b border-gray-200" style={{ minWidth }}>
                    {renderExpandedContent(itemData)}
                </div>
            )}
        </div>
    );
}
