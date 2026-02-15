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
    dynamicSizing = false,
    expandable = false,
}: VirtualizedTableProps) => {
    const [expandedItems, setExpandedItems] = useState<Set<string>>(new Set());
    const useDynamicSizing = dynamicSizing || expandable;
    const parentRef = useRef<HTMLDivElement>(null);
    const [visibleItemIds, setVisibleItemIds] = useState<Set<string>>(new Set());

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
        // Enable dynamic measurement of actual element sizes
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
            {renderHeader()}

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
                            {renderRow({
                                itemId,
                                isVisible,
                                onItemClick: handleItemClick,
                                isExpanded: expandedItems?.has(itemId) ?? false,
                                onToggleExpand: handleToggleExpand,
                            })}
                        </div>
                    );
                })}
            </div>
        </div>
    );
};
