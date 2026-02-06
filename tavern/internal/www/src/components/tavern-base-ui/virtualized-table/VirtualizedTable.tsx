import { useCallback, useEffect, useRef, useState } from "react";
import { useVirtualizer } from "@tanstack/react-virtual";
import { VirtualizedTableProps } from "./types";


export const VirtualizedTable = ({
    items,
    renderRow,
    renderHeader,
    onItemClick,
    hasMore = false,
    onLoadMore,
    estimateRowSize = 73,
    overscan = 5,
    height = "calc(100vh - 180px)",
    minHeight = "400px",
}: VirtualizedTableProps) => {
    const parentRef = useRef<HTMLDivElement>(null);
    const [visibleItemIds, setVisibleItemIds] = useState<Set<string>>(new Set());

    const handleItemClick = useCallback((itemId: string) => {
        if (onItemClick) {
            onItemClick(itemId);
        }
    }, [onItemClick]);

    const rowVirtualizer = useVirtualizer({
        count: items.length,
        getScrollElement: () => parentRef.current,
        estimateSize: () => estimateRowSize,
        overscan,
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

    // Track which rows are actually visible (not just in overscan)
    useEffect(() => {
        updateVisibleItemIds();
    }, [updateVisibleItemIds, range?.startIndex, range?.endIndex]);

    // Detect when user scrolls near the bottom and load more
    useEffect(() => {
        checkAndLoadMore();
    }, [checkAndLoadMore, range?.endIndex]);

    return (
        <div
            ref={parentRef}
            className="overflow-auto border border-gray-200 rounded-lg"
            style={{
                height,
                minHeight,
                width: '100%'
            }}
        >
            {/* Header */}
            {renderHeader()}

            {/* Virtualized rows container */}
            <div style={{ height: `${rowVirtualizer.getTotalSize()}px`, width: '100%', position: 'relative' }}>
                {rowVirtualizer.getVirtualItems().map(virtualRow => {
                    const itemId = items[virtualRow.index];
                    const isVisible = visibleItemIds.has(itemId);
                    return (
                        <div
                            key={itemId}
                            style={{
                                position: 'absolute',
                                top: 0,
                                left: 0,
                                width: '100%',
                                transform: `translateY(${virtualRow.start}px)`,
                            }}
                        >
                            {renderRow({ itemId, isVisible, onItemClick: handleItemClick })}
                        </div>
                    );
                })}
            </div>
        </div>
    );
};
