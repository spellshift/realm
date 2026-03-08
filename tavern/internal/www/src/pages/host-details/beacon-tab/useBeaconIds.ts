import { useQuery, NetworkStatus } from "@apollo/client";
import { useCallback, useEffect, useMemo, useState } from "react";
import { TableRowLimit } from "../../../utils/enums";
import { Cursor } from "../../../utils/interfacesQuery";
import { GET_BEACON_IDS_QUERY } from "./queries";
import { BeaconIdsQueryTopLevel, GetBeaconIdsQueryVariables } from "./types";

export const useBeaconIds = (hostId: string) => {
  const [allBeaconIds, setAllBeaconIds] = useState<string[]>([]);
  const [hasMore, setHasMore] = useState(true);
  const [endCursor, setEndCursor] = useState<Cursor | undefined>(undefined);

  const queryVariables = useMemo(
    () => getBeaconIdsQuery(hostId, undefined),
    [hostId]
  );

  const { data, previousData, error, fetchMore, networkStatus, loading, refetch } = useQuery<BeaconIdsQueryTopLevel>(
    GET_BEACON_IDS_QUERY,
    {
      variables: queryVariables,
      notifyOnNetworkStatusChange: true,
      fetchPolicy: 'cache-and-network',
    }
  );

  // When data changes, update accumulated IDs
  useEffect(() => {
    const currentData = data ?? previousData;
    if (currentData?.beacons?.edges) {
      const ids = currentData.beacons.edges.map(edge => edge.node.id);
      setAllBeaconIds(ids);
      setHasMore(currentData.beacons.pageInfo?.hasNextPage ?? false);
      setEndCursor(currentData.beacons.pageInfo?.endCursor);
    }
  }, [data, previousData]);

  // Reset when hostId changes
  useEffect(() => {
    setAllBeaconIds([]);
    setHasMore(true);
    setEndCursor(undefined);
  }, [hostId]);

  const loadMore = useCallback(async () => {
    if (!hasMore || loading) return;

    try {
      await fetchMore({
        variables: getBeaconIdsQuery(hostId, endCursor),
        updateQuery: (previousResult, { fetchMoreResult }) => {
          if (!fetchMoreResult) return previousResult;

          const newEdges = fetchMoreResult.beacons.edges;
          const newIds = newEdges.map(edge => edge.node.id);

          // Append new IDs to existing list
          setAllBeaconIds(prev => [...prev, ...newIds]);
          setHasMore(fetchMoreResult.beacons.pageInfo?.hasNextPage ?? false);
          setEndCursor(fetchMoreResult.beacons.pageInfo?.endCursor);

          return {
            beacons: {
              ...fetchMoreResult.beacons,
              edges: [...(previousResult.beacons?.edges || []), ...newEdges],
            },
          };
        },
      });
    } catch (err) {
      console.error("Error loading more beacons:", err);
    }
  }, [hasMore, loading, fetchMore, hostId, endCursor]);

  const currentData = data ?? previousData;

  return {
    data: currentData,
    beaconIds: allBeaconIds,
    initialLoading: networkStatus === NetworkStatus.loading && !currentData,
    error,
    hasMore,
    loadMore,
    refetch,
  };
};

const getBeaconIdsQuery = (
  hostId: string,
  afterCursor?: Cursor,
): GetBeaconIdsQueryVariables => {
  const defaultRowLimit = TableRowLimit.HostRowLimit;

  const query: GetBeaconIdsQueryVariables = {
    where: {
      hasHostWith: { id: hostId }
    },
    first: defaultRowLimit,
    after: afterCursor || undefined,
  };

  return query;
};
