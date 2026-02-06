import { useCallback, useEffect, useRef, useState } from "react";
import { useVirtualizer } from "@tanstack/react-virtual";
import { VirtualizedCardListProps } from "./types";

export const VirtualizedCardList = ({
    items,
    renderCard,
    hasMore = false,
    onLoadMore,
    estimateCardSize = 300,
    overscan = 3,
    height = "calc(100vh - 180px)",
    minHeight = "400px",
    gap = 16,
    padding = "1rem",
    dynamicSizing = true,
}: VirtualizedCardListProps) => {
    const parentRef = useRef<HTMLDivElement>(null);
    const [visibleItemIds, setVisibleItemIds] = useState<Set<string>>(new Set());

    const rowVirtualizer = useVirtualizer({
        count: items.length,
        getScrollElement: () => parentRef.current,
        estimateSize: () => estimateCardSize + gap,
        overscan,
        // Enable dynamic measurement of actual element sizes
        measureElement: dynamicSizing
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

        if (lastItem.index >= items.length - 3 && hasMore && onLoadMore) {
            onLoadMore();
        }
    }, [rowVirtualizer, items.length, hasMore, onLoadMore]);

    const range = rowVirtualizer.range;

    // Track which cards are actually visible (not just in overscan)
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
            className="overflow-auto"
            style={{
                height,
                minHeight,
                width: '100%',
                padding,
            }}
        >
            {/* Virtualized cards container */}
            <div style={{ height: `${rowVirtualizer.getTotalSize()}px`, width: '100%', position: 'relative' }}>
                {rowVirtualizer.getVirtualItems().map(virtualItem => {
                    const itemId = items[virtualItem.index];
                    const isVisible = visibleItemIds.has(itemId);
                    return (
                        <div
                            key={itemId}
                            ref={dynamicSizing ? rowVirtualizer.measureElement : undefined}
                            data-index={virtualItem.index}
                            style={{
                                position: 'absolute',
                                top: 0,
                                left: 0,
                                width: '100%',
                                transform: `translateY(${virtualItem.start}px)`,
                                paddingBottom: `${gap}px`,
                            }}
                        >
                            {renderCard({ itemId, isVisible })}
                        </div>
                    );
                })}
            </div>
        </div>
    );
};
