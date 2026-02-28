import { useCallback, useEffect, useMemo, useState } from "react";
import { ApolloError, NetworkStatus, useQuery, gql } from "@apollo/client";
import { TableRowLimit } from "../../../utils/enums";
import { Cursor, OrderByField, QueryPageInfo } from "../../../utils/interfacesQuery";

export const GET_HOST_SCREENSHOT_IDS_QUERY = gql`
    query GetHostScreenshotIds($where: ScreenshotWhereInput, $first: Int, $last: Int, $after: Cursor, $before: Cursor, $orderBy: [ScreenshotOrder!]) {
        screenshots(
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

interface HostScreenshotIdsResponse {
    screenshots: {
        pageInfo: QueryPageInfo;
        totalCount: number;
        edges: Array<{ node: { id: string } }>;
    };
}

interface HostScreenshotIdsHook {
    screenshotIds: string[];
    totalCount: number | undefined;
    loading: boolean;
    initialLoading: boolean;
    error: ApolloError | undefined;
    hasMore: boolean;
    loadMore: () => void;
    refetch: () => void;
}

export const useHostScreenshotIds = (hostId?: string): HostScreenshotIdsHook => {
    const [allScreenshotIds, setAllScreenshotIds] = useState<string[]>([]);
    const [endCursor, setEndCursor] = useState<Cursor>(null);

    const queryVariables = useMemo(
        () => ({
            "where": {
                "hasHostWith": {
                    "id": hostId
                }
            },
            "first": 50,
            "orderBy": [
                {
                    "direction": "DESC",
                    "field": "CREATED_AT"
                }
            ]
        }),
        [hostId]
    );

    const { data, error, refetch, networkStatus, loading, fetchMore } = useQuery<HostScreenshotIdsResponse>(
        GET_HOST_SCREENSHOT_IDS_QUERY,
        {
            variables: queryVariables,
            notifyOnNetworkStatusChange: true,
            fetchPolicy: 'cache-and-network',
            skip: !hostId
        }
    );

    useEffect(() => {
        if (data?.screenshots?.edges) {
            const ids = data.screenshots.edges.map(edge => edge.node.id);
            setAllScreenshotIds(ids);
            setEndCursor(data.screenshots.pageInfo.endCursor);
        }
    }, [data]);

    const hasMore = data?.screenshots?.pageInfo?.hasNextPage ?? false;

    const loadMore = useCallback(() => {
        if (!hasMore || loading) return;

        fetchMore({
            variables: {
                ...queryVariables,
                after: endCursor,
            },
            updateQuery: (prev, { fetchMoreResult }) => {
                if (!fetchMoreResult) return prev;

                const newEdges = fetchMoreResult.screenshots.edges;
                const newIds = newEdges.map(edge => edge.node.id);

                setAllScreenshotIds(prevIds => [...prevIds, ...newIds]);
                setEndCursor(fetchMoreResult.screenshots.pageInfo.endCursor);

                return {
                    screenshots: {
                        ...fetchMoreResult.screenshots,
                        edges: [...prev.screenshots.edges, ...newEdges],
                    },
                };
            },
        });
    }, [hasMore, loading, fetchMore, queryVariables, endCursor]);

    const handleRefetch = useCallback(() => {
        setAllScreenshotIds([]);
        setEndCursor(null);
        refetch(queryVariables);
    }, [refetch, queryVariables]);

    const currentPageIds = data?.screenshots?.edges?.map(edge => edge.node.id) ?? [];
    const screenshotIds = allScreenshotIds.length > 0 ? allScreenshotIds : currentPageIds;

    return {
        screenshotIds,
        totalCount: data?.screenshots?.totalCount,
        loading,
        initialLoading: (networkStatus === NetworkStatus.loading && !data),
        error,
        hasMore,
        loadMore,
        refetch: handleRefetch,
    };
};
