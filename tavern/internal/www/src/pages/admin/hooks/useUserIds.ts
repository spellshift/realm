import { useQuery, NetworkStatus } from "@apollo/client";
import { useCallback, useEffect, useState } from "react";
import { Cursor } from "../../../utils/interfacesQuery";
import { GET_USER_IDS_QUERY } from "../queries";
import { UserIdsQueryTopLevel, GetUserIdsQueryVariables } from "../types";

const DEFAULT_ROW_LIMIT = 14;

export const useUserIds = () => {
    const [allUserIds, setAllUserIds] = useState<string[]>([]);
    const [hasMore, setHasMore] = useState(true);
    const [endCursor, setEndCursor] = useState<Cursor | undefined>(undefined);

    const queryVariables: GetUserIdsQueryVariables = {
        first: DEFAULT_ROW_LIMIT,
    };

    const { data, previousData, error, fetchMore, networkStatus, loading, refetch } = useQuery<UserIdsQueryTopLevel>(
        GET_USER_IDS_QUERY,
        {
            variables: queryVariables,
            notifyOnNetworkStatusChange: true,
            fetchPolicy: 'cache-and-network',
        }
    );

    useEffect(() => {
        const currentData = data ?? previousData;
        if (currentData?.users?.edges) {
            const ids = currentData.users.edges.map(edge => edge.node.id);
            setAllUserIds(ids);
            setHasMore(currentData.users.pageInfo?.hasNextPage ?? false);
            setEndCursor(currentData.users.pageInfo?.endCursor);
        }
    }, [data, previousData]);

    const loadMore = useCallback(async () => {
        if (!hasMore || loading) return;

        try {
            await fetchMore({
                variables: {
                    first: DEFAULT_ROW_LIMIT,
                    after: endCursor,
                } as GetUserIdsQueryVariables,
                updateQuery: (previousResult, { fetchMoreResult }) => {
                    if (!fetchMoreResult) return previousResult;

                    const newEdges = fetchMoreResult.users.edges;
                    const newIds = newEdges.map(edge => edge.node.id);

                    setAllUserIds(prev => [...prev, ...newIds]);
                    setHasMore(fetchMoreResult.users.pageInfo?.hasNextPage ?? false);
                    setEndCursor(fetchMoreResult.users.pageInfo?.endCursor);

                    return {
                        users: {
                            ...fetchMoreResult.users,
                            edges: [...(previousResult.users?.edges || []), ...newEdges],
                        },
                    };
                },
            });
        } catch (err) {
            console.error("Error loading more users:", err);
        }
    }, [hasMore, loading, fetchMore, endCursor]);

    const currentData = data ?? previousData;

    return {
        data: currentData,
        userIds: allUserIds,
        initialLoading: networkStatus === NetworkStatus.loading && !currentData,
        error,
        hasMore,
        loadMore,
        refetch,
    };
};
