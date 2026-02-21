import { useCallback, useEffect, useMemo, useState } from "react";
import { ApolloError, NetworkStatus, useQuery, gql } from "@apollo/client";
import { PageNavItem, TableRowLimit } from "../../utils/enums";
import { Filters, useFilters } from "../../context/FilterContext";
import { constructTaskFilterQuery } from "../../utils/constructQueryUtils";
import { Cursor, OrderByField, QueryPageInfo } from "../../utils/interfacesQuery";
import { useSorts } from "../../context/SortContext";
import { useTags } from "../../context/TagContext";

export const GET_TASK_IDS_QUERY = gql`
    query GetTaskIds($where: TaskWhereInput, $first: Int, $last: Int, $after: Cursor, $before: Cursor, $orderBy: [TaskOrder!]) {
        tasks(
            where: $where
            first: $first
            last: $last
            after: $after
            before: $before
            orderBy: $orderBy
        ) {
            pageInfo {
                hasNextPage
                hasPreviousPage
                startCursor
                endCursor
            }
            totalCount
            edges {
                node {
                    id
                }
            }
        }
    }
`;

interface TaskIdsResponse {
    tasks: {
        pageInfo: QueryPageInfo;
        totalCount: number;
        edges: Array<{ node: { id: string } }>;
    };
}

interface TaskIdsHook {
    taskIds: string[];
    totalCount: number | undefined;
    loading: boolean;
    initialLoading: boolean;
    error: ApolloError | undefined;
    hasMore: boolean;
    loadMore: () => void;
    refetch: () => void;
}

export const useTaskIds = (questId?: string): TaskIdsHook => {
    const [allTaskIds, setAllTaskIds] = useState<string[]>([]);
    const [endCursor, setEndCursor] = useState<Cursor>(null);
    const { filters } = useFilters();
    const { sorts } = useSorts();
    const { lastFetchedTimestamp } = useTags();
    const taskSort = sorts[PageNavItem.tasks];

    const queryVariables = useMemo(
        () => getTaskIdsQuery(filters, undefined, undefined, questId, taskSort, lastFetchedTimestamp),
        [filters, questId, taskSort, lastFetchedTimestamp]
    );

    const { data, error, refetch, networkStatus, loading, fetchMore } = useQuery<TaskIdsResponse>(
        GET_TASK_IDS_QUERY,
        {
            variables: queryVariables,
            notifyOnNetworkStatusChange: true,
            fetchPolicy: 'cache-and-network',
        }
    );

    useEffect(() => {
        if (data?.tasks?.edges) {
            const ids = data.tasks.edges.map(edge => edge.node.id);
            setAllTaskIds(ids);
            setEndCursor(data.tasks.pageInfo.endCursor);
        }
    }, [data]);

    const hasMore = data?.tasks?.pageInfo?.hasNextPage ?? false;

    const loadMore = useCallback(() => {
        if (!hasMore || loading) return;

        fetchMore({
            variables: {
                ...queryVariables,
                after: endCursor,
            },
            updateQuery: (prev, { fetchMoreResult }) => {
                if (!fetchMoreResult) return prev;

                const newEdges = fetchMoreResult.tasks.edges;
                const newIds = newEdges.map(edge => edge.node.id);

                setAllTaskIds(prevIds => [...prevIds, ...newIds]);
                setEndCursor(fetchMoreResult.tasks.pageInfo.endCursor);

                return {
                    tasks: {
                        ...fetchMoreResult.tasks,
                        edges: [...prev.tasks.edges, ...newEdges],
                    },
                };
            },
        });
    }, [hasMore, loading, fetchMore, queryVariables, endCursor]);

    const handleRefetch = useCallback(() => {
        setAllTaskIds([]);
        setEndCursor(null);
        refetch(queryVariables);
    }, [refetch, queryVariables]);

    const currentPageIds = data?.tasks?.edges?.map(edge => edge.node.id) ?? [];
    const taskIds = allTaskIds.length > 0 ? allTaskIds : currentPageIds;

    return {
        taskIds,
        totalCount: data?.tasks?.totalCount,
        loading,
        initialLoading: (networkStatus === NetworkStatus.loading && !data),
        error,
        hasMore,
        loadMore,
        refetch: handleRefetch,
    };
};

const getTaskIdsQuery = (
    filters: Filters,
    afterCursor?: Cursor,
    beforeCursor?: Cursor,
    questId?: string,
    sort?: OrderByField,
    currentTimestamp?: Date
) => {
    const defaultRowLimit = TableRowLimit.TaskRowLimit;
    const filterQueryFields = constructTaskFilterQuery(filters, currentTimestamp, questId);

    const query = {
        "where": {
            ...(filterQueryFields && filterQueryFields.hasTasksWith),
        },
        "first": beforeCursor ? null : defaultRowLimit,
        "last": beforeCursor ? defaultRowLimit : null,
        "after": afterCursor ? afterCursor : null,
        "before": beforeCursor ? beforeCursor : null,
        ...(sort && { orderBy: [sort] })
    } as any;

    return query;
};
