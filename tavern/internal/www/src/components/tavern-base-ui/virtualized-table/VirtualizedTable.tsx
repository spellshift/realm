import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { useVirtualizer } from "@tanstack/react-virtual";
import { DocumentNode } from "@apollo/client";
import { VirtualizedTableProps } from "./types";
import { VirtualizedTableRowInternal } from "./VirtualizedTableRowInternal";

export function VirtualizedTable<TData, TResponse = unknown>({
    items,
    columns,
    query,
    getVariables,
    extractData,
    pollInterval = 5000,
    onItemClick,
    hasMore = false,
    onLoadMore,
    expandable,
    minWidth = "800px",
    estimateRowSize = 73,
    overscan = 5,
    height = "calc(100vh - 180px)",
    minHeight = "400px",
}: VirtualizedTableProps<TData, TResponse>) {
    const [expandedItems, setExpandedItems] = useState<Set<string>>(new Set());
    const useDynamicSizing = expandable !== undefined;
    const parentRef = useRef<HTMLDivElement>(null);
    const [visibleItemIds, setVisibleItemIds] = useState<Set<string>>(new Set());

    // Compute grid template from columns
    const gridTemplateColumns = useMemo(
        () => columns.map(col => col.width).join(' '),
        [columns]
    );

    // Compute header grid (with expand column if needed)
    const headerGridTemplateColumns = useMemo(
        () => expandable ? `32px ${gridTemplateColumns}` : gridTemplateColumns,
        [expandable, gridTemplateColumns]
    );

    // Get query for a specific item (supports both static and dynamic queries)
    const getQueryForItem = useCallback((itemId: string): DocumentNode => {
        if (typeof query === 'function') {
            return query(itemId);
        }
        return query;
    }, [query]);

    const handleItemClick = useCallback((itemId: string) => {
        if (onItemClick) {
            onItemClick(itemId);
        }
    }, [onItemClick]);

    const handleToggleExpand = useCallback((itemId: string) => {
        setExpandedItems(prev => {
            const next = new Set(prev);
            if (next.has(itemId)) {
                next.delete(itemId);
            } else {
                next.add(itemId);
            }
            return next;
        });
    }, []);

    const rowVirtualizer = useVirtualizer({
        count: items.length,
        getScrollElement: () => parentRef.current,
        estimateSize: () => estimateRowSize,
        overscan,
        measureElement: useDynamicSizing
            ? (element) => element.getBoundingClientRect().height
            : undefined,
    });

    const updateVisibleItemIds = useCallback(() => {
        const virtualItems = rowVirtualizer.getVirtualItems();
        const parent = parentRef.current;

        if (!parent || virtualItems.length === 0) return;

        const parentRect = parent.getBoundingClientRect();
        const newVisibleIds = new Set<string>();

        virtualItems.forEach(virtualItem => {
            const itemTop = virtualItem.start;
            const itemBottom = virtualItem.end;
            const scrollTop = parent.scrollTop;
            const viewportTop = scrollTop;
            const viewportBottom = scrollTop + parentRect.height;

            if (itemBottom > viewportTop && itemTop < viewportBottom) {
                const itemId = items[virtualItem.index];
                if (itemId) {
                    newVisibleIds.add(itemId);
                }
            }
        });

        setVisibleItemIds(newVisibleIds);
    }, [rowVirtualizer, items]);

    const checkAndLoadMore = useCallback(() => {
        const [lastItem] = [...rowVirtualizer.getVirtualItems()].reverse();

        if (!lastItem) return;

        if (lastItem.index >= items.length - 5 && hasMore && onLoadMore) {
            onLoadMore();
        }
    }, [rowVirtualizer, items.length, hasMore, onLoadMore]);

    const range = rowVirtualizer.range;

    useEffect(() => {
        updateVisibleItemIds();
    }, [updateVisibleItemIds, range?.startIndex, range?.endIndex]);

    useEffect(() => {
        checkAndLoadMore();
    }, [checkAndLoadMore, range?.endIndex]);

    return (
        <div
            ref={parentRef}
            className="overflow-auto border border-gray-200 rounded-lg"
            style={{
                height: height,
                minHeight,
                width: '100%'
            }}
        >
            {/* Header */}
            <div
                className='bg-gray-50 sticky top-0 z-10 grid gap-4 px-6 py-3 border-b border-gray-200'
                style={{
                    gridTemplateColumns: headerGridTemplateColumns,
                    minWidth,
                }}
            >
                {expandable && <div />}
                {columns.map((column) => (
                    <div
                        key={column.key}
                        className="text-left text-xs font-medium text-gray-500 uppercase tracking-wider"
                    >
                        {column.label}
                    </div>
                ))}
            </div>

            {/* Virtualized rows */}
            <div style={{ height: `${rowVirtualizer.getTotalSize()}px`, width: '100%', position: 'relative' }}>
                {rowVirtualizer.getVirtualItems().map(virtualRow => {
                    const itemId = items[virtualRow.index];
                    const isVisible = visibleItemIds.has(itemId);
                    return (
                        <div
                            key={itemId}
                            ref={useDynamicSizing ? rowVirtualizer.measureElement : undefined}
                            data-index={virtualRow.index}
                            style={{
                                position: 'absolute',
                                top: 0,
                                left: 0,
                                width: '100%',
                                transform: `translateY(${virtualRow.start}px)`,
                            }}
                        >
                            <VirtualizedTableRowInternal<TData, TResponse>
                                itemId={itemId}
                                query={getQueryForItem(itemId)}
                                getVariables={getVariables}
                                extractData={extractData}
                                columns={columns}
                                isVisible={isVisible}
                                pollInterval={pollInterval}
                                onRowClick={onItemClick ? handleItemClick : undefined}
                                minWidth={minWidth}
                                gridTemplateColumns={gridTemplateColumns}
                                isExpanded={expandedItems.has(itemId)}
                                onToggleExpand={handleToggleExpand}
                                expandable={expandable}
                            />
                        </div>
                    );
                })}
            </div>
        </div>
    );
}
