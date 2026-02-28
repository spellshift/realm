import { useQuery } from "@apollo/client";
import { useMemo } from "react";
import { VirtualizedCardItemProps } from "./types";

/**
 * Generic virtualized card item component that fetches and displays data
 *
 * This component abstracts the common pattern of:
 * - Fetching data per card with Apollo useQuery
 * - Polling when visible in viewport
 * - Showing loading skeletons
 * - Rendering custom card content
 *
 * @example
 * ```tsx
 * <VirtualizedCardItem
 *   itemId="task-123"
 *   query={GET_TASK_DETAIL_QUERY}
 *   getVariables={(id) => ({ where: { id } })}
 *   renderCard={(task) => <TaskCard task={task} />}
 *   renderSkeleton={() => <TaskCardSkeleton />}
 *   extractData={(response) => response.tasks?.edges?.[0]?.node}
 *   isVisible={true}
 * />
 * ```
 */
export function VirtualizedCardItem<TData, TVariables = Record<string, unknown>>({
    itemId,
    query,
    getVariables,
    renderCard,
    renderSkeleton,
    isVisible,
    pollInterval = 10000,
    extractData,
    className = "",
}: VirtualizedCardItemProps<TData, TVariables>) {
    const queryVariables = useMemo(() => getVariables(itemId), [itemId, getVariables]);

    const { data } = useQuery(query, {
        variables: queryVariables,
        pollInterval: isVisible ? pollInterval : 0,
        fetchPolicy: 'cache-and-network',
    });

    if (!data) {
        return (
            <div className={className}>
                {renderSkeleton()}
            </div>
        );
    }

    const itemData = extractData(data);

    if (!itemData) {
        return null;
    }

    return (
        <div className={className}>
            {renderCard(itemData)}
        </div>
    );
}
