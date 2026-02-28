import { useQuery, NetworkStatus } from "@apollo/client";
import { useCallback, useEffect, useMemo, useState } from "react";
import { TableRowLimit } from "../../../utils/enums";
import { Cursor } from "../../../utils/interfacesQuery";
import { GET_SHELL_IDS_QUERY } from "./queries";
import { ShellIdsQueryTopLevel, GetShellIdsQueryVariables } from "./types";

export const useShellIds = (hostId: string) => {
  const [allShellIds, setAllShellIds] = useState<string[]>([]);
  const [hasMore, setHasMore] = useState(true);
  const [endCursor, setEndCursor] = useState<Cursor | undefined>(undefined);

  const queryVariables = useMemo(
    () => getShellIdsQuery(hostId, undefined),
    [hostId]
  );

  const { data, previousData, error, fetchMore, networkStatus, loading, refetch } = useQuery<ShellIdsQueryTopLevel>(
    GET_SHELL_IDS_QUERY,
    {
      variables: queryVariables,
      notifyOnNetworkStatusChange: true,
      fetchPolicy: 'cache-and-network',
    }
  );

  // When data changes, update accumulated IDs
  useEffect(() => {
    const currentData = data ?? previousData;
    if (currentData?.shells?.edges) {
      const ids = currentData.shells.edges.map(edge => edge.node.id);
      setAllShellIds(ids);
      setHasMore(currentData.shells.pageInfo?.hasNextPage ?? false);
      setEndCursor(currentData.shells.pageInfo?.endCursor);
    }
  }, [data, previousData]);

  // Reset when hostId changes
  useEffect(() => {
    setAllShellIds([]);
    setHasMore(true);
    setEndCursor(undefined);
  }, [hostId]);

  const loadMore = useCallback(async () => {
    if (!hasMore || loading) return;

    try {
      await fetchMore({
        variables: getShellIdsQuery(hostId, endCursor),
        updateQuery: (previousResult, { fetchMoreResult }) => {
          if (!fetchMoreResult) return previousResult;

          const newEdges = fetchMoreResult.shells.edges;
          const newIds = newEdges.map(edge => edge.node.id);

          // Append new IDs to existing list
          setAllShellIds(prev => [...prev, ...newIds]);
          setHasMore(fetchMoreResult.shells.pageInfo?.hasNextPage ?? false);
          setEndCursor(fetchMoreResult.shells.pageInfo?.endCursor);

          return {
            shells: {
              ...fetchMoreResult.shells,
              edges: [...(previousResult.shells?.edges || []), ...newEdges],
            },
          };
        },
      });
    } catch (err) {
      console.error("Error loading more shells:", err);
    }
  }, [hasMore, loading, fetchMore, hostId, endCursor]);

  const currentData = data ?? previousData;

  return {
    data: currentData,
    shellIds: allShellIds,
    initialLoading: networkStatus === NetworkStatus.loading && !currentData,
    error,
    hasMore,
    loadMore,
    refetch,
  };
};

const getShellIdsQuery = (
  hostId: string,
  afterCursor?: Cursor,
): GetShellIdsQueryVariables => {
  const defaultRowLimit = TableRowLimit.HostRowLimit;

  const query: GetShellIdsQueryVariables = {
    where: {
      hasBeaconWith: {
        hasHostWith: {
          id: hostId
        }
      }
    },
    first: defaultRowLimit,
    after: afterCursor || undefined,
  };

  return query;
};
