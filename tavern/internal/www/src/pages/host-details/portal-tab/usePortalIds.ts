import { useQuery, NetworkStatus } from "@apollo/client";
import { useCallback, useEffect, useMemo, useState } from "react";
import { TableRowLimit } from "../../../utils/enums";
import { Cursor } from "../../../utils/interfacesQuery";
import { GET_PORTAL_IDS_QUERY } from "./queries";
import { PortalIdsQueryTopLevel, GetPortalIdsQueryVariables } from "./types";

export const usePortalIds = (hostId: string) => {
  const [allPortalIds, setAllPortalIds] = useState<string[]>([]);
  const [hasMore, setHasMore] = useState(true);
  const [endCursor, setEndCursor] = useState<Cursor | undefined>(undefined);

  const queryVariables = useMemo(
    () => getPortalIdsQuery(hostId, undefined),
    [hostId]
  );

  const { data, previousData, error, fetchMore, networkStatus, loading, refetch } = useQuery<PortalIdsQueryTopLevel>(
    GET_PORTAL_IDS_QUERY,
    {
      variables: queryVariables,
      notifyOnNetworkStatusChange: true,
      fetchPolicy: 'cache-and-network',
    }
  );

  // When data changes, update accumulated IDs
  useEffect(() => {
    const currentData = data ?? previousData;
    if (currentData?.portals?.edges) {
      const ids = currentData.portals.edges.map(edge => edge.node.id);
      setAllPortalIds(ids);
      setHasMore(currentData.portals.pageInfo?.hasNextPage ?? false);
      setEndCursor(currentData.portals.pageInfo?.endCursor);
    }
  }, [data, previousData]);

  // Reset when hostId changes
  useEffect(() => {
    setAllPortalIds([]);
    setHasMore(true);
    setEndCursor(undefined);
  }, [hostId]);

  const loadMore = useCallback(async () => {
    if (!hasMore || loading) return;

    try {
      await fetchMore({
        variables: getPortalIdsQuery(hostId, endCursor),
        updateQuery: (previousResult, { fetchMoreResult }) => {
          if (!fetchMoreResult) return previousResult;

          const newEdges = fetchMoreResult.portals.edges;
          const newIds = newEdges.map(edge => edge.node.id);

          // Append new IDs to existing list
          setAllPortalIds(prev => [...prev, ...newIds]);
          setHasMore(fetchMoreResult.portals.pageInfo?.hasNextPage ?? false);
          setEndCursor(fetchMoreResult.portals.pageInfo?.endCursor);

          return {
            portals: {
              ...fetchMoreResult.portals,
              edges: [...(previousResult.portals?.edges || []), ...newEdges],
            },
          };
        },
      });
    } catch (err) {
      console.error("Error loading more portals:", err);
    }
  }, [hasMore, loading, fetchMore, hostId, endCursor]);

  const currentData = data ?? previousData;

  return {
    data: currentData,
    portalIds: allPortalIds,
    initialLoading: networkStatus === NetworkStatus.loading && !currentData,
    error,
    hasMore,
    loadMore,
    refetch,
  };
};

const getPortalIdsQuery = (
  hostId: string,
  afterCursor?: Cursor,
): GetPortalIdsQueryVariables => {
  const defaultRowLimit = TableRowLimit.HostRowLimit;

  const query: GetPortalIdsQueryVariables = {
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
