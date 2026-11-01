import { useQuery, NetworkStatus } from "@apollo/client";
import { useCallback, useEffect, useMemo, useState } from "react";
import { PageNavItem, TableRowLimit } from "../../utils/enums";
import { useFilters } from "../../context/FilterContext";
import { constructHostFilterQuery } from "../../utils/constructQueryUtils";
import { Cursor, OrderByField } from "../../utils/interfacesQuery";
import { HostFilterRow } from "../../utils/interfacesUI";
import { useSorts } from "../../context/SortContext";
import { GET_HOST_IDS_QUERY } from "./queries";
import { HostIdsQueryTopLevel, GetHostIdsQueryVariables } from "./types";

// Incomplete rows (no value picked yet) must not change the query: the table
// waits until the row is complete instead of flashing empty.
const isCompleteHostFilterRow = (row: HostFilterRow): boolean => row.values.length > 0;

// TODO: Clean up load more / hasMore, this seems like overengineering to avoid getting a full list of ids, we might as well.
export const useHostIds = () => {
  const { filters } = useFilters();
  const { sorts } = useSorts();
  const hostSort = sorts[PageNavItem.hosts];

  const [allHostIds, setAllHostIds] = useState<string[]>([]);
  const [hasMore, setHasMore] = useState(true);
  const [endCursor, setEndCursor] = useState<Cursor | undefined>(undefined);

  const completeHostRows = useMemo(
    () => filters.hostFilters.filter(isCompleteHostFilterRow),
    [filters.hostFilters]
  );
  const completeHostRowsKey = JSON.stringify(completeHostRows);

  const queryVariables = useMemo(
    () => getHostIdsQuery(completeHostRows, undefined, hostSort),
    // eslint-disable-next-line react-hooks/exhaustive-deps
    [completeHostRowsKey, hostSort]
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

  // Reset when the completed filters or sort change. Incomplete rows are
  // ignored so half-built filters never clear the table.
  useEffect(() => {
    setAllHostIds([]);
    setHasMore(true);
    setEndCursor(undefined);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [completeHostRowsKey, hostSort]);

  const loadMore = useCallback(async () => {
    if (!hasMore || loading) return;

    try {
      await fetchMore({
        variables: getHostIdsQuery(completeHostRows, endCursor, hostSort),
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
  }, [hasMore, loading, fetchMore, completeHostRows, endCursor, hostSort]);

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
  rows: HostFilterRow[],
  afterCursor?: Cursor,
  sort?: OrderByField,
): GetHostIdsQueryVariables => {
  const currentTimestamp = new Date();
  const defaultRowLimit = TableRowLimit.HostRowLimit;
  const where = constructHostFilterQuery(rows, currentTimestamp) ?? {};

  const query: GetHostIdsQueryVariables = {
    where,
    ...(sort && { orderBy: [sort] }),
    first: defaultRowLimit,
    after: afterCursor || undefined,
  };

  return query;
};
