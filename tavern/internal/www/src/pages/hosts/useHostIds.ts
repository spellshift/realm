import { useQuery, NetworkStatus } from "@apollo/client";
import { useCallback, useEffect, useMemo, useState } from "react";
import { PageNavItem, TableRowLimit } from "../../utils/enums";
import { Filters, useFilters } from "../../context/FilterContext";
import { constructBeaconFilterQuery } from "../../utils/constructQueryUtils";
import { Cursor, OrderByField } from "../../utils/interfacesQuery";
import { useSorts } from "../../context/SortContext";
import { useTags } from "../../context/TagContext";
import { GET_HOST_IDS_QUERY } from "./queries";
import { HostIdsQueryTopLevel, GetHostIdsQueryVariables } from "./types";

export const useHostIds = () => {
  const { filters } = useFilters();
  const { sorts } = useSorts();
  const { lastFetchedTimestamp } = useTags();
  const hostSort = sorts[PageNavItem.hosts];

  const [allHostIds, setAllHostIds] = useState<string[]>([]);
  const [hasMore, setHasMore] = useState(true);
  const [endCursor, setEndCursor] = useState<Cursor | undefined>(undefined);

  const queryVariables = useMemo(
    () => getHostIdsQuery(filters, undefined, hostSort, lastFetchedTimestamp),
    [filters, hostSort, lastFetchedTimestamp]
  );

  const { data, previousData, error, fetchMore, networkStatus, loading, refetch } = useQuery<HostIdsQueryTopLevel>(
    GET_HOST_IDS_QUERY,
    {
      variables: queryVariables,
      notifyOnNetworkStatusChange: true,
      fetchPolicy: 'cache-and-network',
    }
  );

  // When data changes, update accumulated IDs
  useEffect(() => {
    const currentData = data ?? previousData;
    if (currentData?.hosts?.edges) {
      const ids = currentData.hosts.edges.map(edge => edge.node.id);
      setAllHostIds(ids);
      setHasMore(currentData.hosts.pageInfo?.hasNextPage ?? false);
      setEndCursor(currentData.hosts.pageInfo?.endCursor);
    }
  }, [data, previousData]);

  // Reset when filters or sort change
  useEffect(() => {
    setAllHostIds([]);
    setHasMore(true);
    setEndCursor(undefined);
  }, [filters, hostSort]);

  const loadMore = useCallback(async () => {
    if (!hasMore || loading) return;

    try {
      await fetchMore({
        variables: getHostIdsQuery(filters, endCursor, hostSort, lastFetchedTimestamp),
        updateQuery: (previousResult, { fetchMoreResult }) => {
          if (!fetchMoreResult) return previousResult;

          const newEdges = fetchMoreResult.hosts.edges;
          const newIds = newEdges.map(edge => edge.node.id);

          // Append new IDs to existing list
          setAllHostIds(prev => [...prev, ...newIds]);
          setHasMore(fetchMoreResult.hosts.pageInfo?.hasNextPage ?? false);
          setEndCursor(fetchMoreResult.hosts.pageInfo?.endCursor);

          return {
            hosts: {
              ...fetchMoreResult.hosts,
              edges: [...(previousResult.hosts?.edges || []), ...newEdges],
            },
          };
        },
      });
    } catch (err) {
      console.error("Error loading more hosts:", err);
    }
  }, [hasMore, loading, fetchMore, filters, endCursor, hostSort, lastFetchedTimestamp]);

  const currentData = data ?? previousData;

  return {
    data: currentData,
    hostIds: allHostIds,
    initialLoading: networkStatus === NetworkStatus.loading && !currentData,
    error,
    hasMore,
    loadMore,
    refetch,
  };
};

const getHostIdsQuery = (
  filters: Filters,
  afterCursor?: Cursor,
  sort?: OrderByField,
  currentTimestamp?: Date
): GetHostIdsQueryVariables => {
  const defaultRowLimit = TableRowLimit.HostRowLimit;
  const filterInfo = constructBeaconFilterQuery(filters.beaconFields, currentTimestamp);
  const hostFields = (filterInfo && filterInfo.hasBeaconWith?.hasHostWith) || {};
  const beaconFields = (filterInfo && filterInfo.hasBeaconWith) ? { ...filterInfo.hasBeaconWith } : {};
  delete beaconFields.hasHostWith; 

  const query: GetHostIdsQueryVariables = {
    where: {
      ...hostFields,
      ...(Object.keys(beaconFields).length > 0) && { "hasBeaconsWith": beaconFields }
    },
    ...(sort && { orderBy: [sort] }),
    first: defaultRowLimit,
    after: afterCursor || undefined,
  };

  return query;
};
