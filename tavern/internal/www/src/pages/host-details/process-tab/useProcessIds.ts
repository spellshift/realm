import { useQuery, NetworkStatus } from "@apollo/client";
import { useCallback, useEffect, useMemo, useState } from "react";
import { PageNavItem, TableRowLimit } from "../../../utils/enums";
import { Cursor, OrderByField } from "../../../utils/interfacesQuery";
import { useSorts } from "../../../context/SortContext";
import { GET_PROCESS_IDS_QUERY } from "./queries";
import { ProcessIdsQueryResponse, GetProcessIdsQueryVariables } from "./types";

export const useProcessIds = (hostId: string) => {
  const { sorts } = useSorts();
  const processSort = sorts[PageNavItem.processes];

  const [allProcessIds, setAllProcessIds] = useState<string[]>([]);
  const [hasMore, setHasMore] = useState(true);
  const [endCursor, setEndCursor] = useState<Cursor | undefined>(undefined);
  const [searchTerm, setSearchTerm] = useState<string>("");

  const queryVariables = useMemo(
    () => getProcessIdsQuery(hostId, undefined, searchTerm, processSort),
    [hostId, searchTerm, processSort]
  );

  const { data, previousData, error, fetchMore, networkStatus, loading, refetch } = useQuery<ProcessIdsQueryResponse>(
    GET_PROCESS_IDS_QUERY,
    {
      variables: queryVariables,
      notifyOnNetworkStatusChange: true,
      fetchPolicy: 'cache-and-network',
      skip: !hostId,
    }
  );

  // Extract processes from nested host response
  const getProcessesFromResponse = useCallback((response: ProcessIdsQueryResponse | undefined) => {
    return response?.hosts?.edges?.[0]?.node?.processes;
  }, []);

  // When data changes, update accumulated IDs
  useEffect(() => {
    const currentData = data ?? previousData;
    const processes = getProcessesFromResponse(currentData);
    if (processes?.edges) {
      const ids = processes.edges.map(edge => edge.node.id);
      setAllProcessIds(ids);
      setHasMore(processes.pageInfo?.hasNextPage ?? false);
      setEndCursor(processes.pageInfo?.endCursor);
    }
  }, [data, previousData, getProcessesFromResponse]);

  // Reset when hostId, searchTerm, or sort changes
  useEffect(() => {
    setAllProcessIds([]);
    setHasMore(true);
    setEndCursor(undefined);
  }, [hostId, searchTerm, processSort]);

  const loadMore = useCallback(async () => {
    if (!hasMore || loading) return;

    try {
      await fetchMore({
        variables: getProcessIdsQuery(hostId, endCursor, searchTerm, processSort),
        updateQuery: (previousResult, { fetchMoreResult }) => {
          if (!fetchMoreResult) return previousResult;

          const newProcesses = getProcessesFromResponse(fetchMoreResult);
          if (!newProcesses) return previousResult;

          const newEdges = newProcesses.edges;
          const newIds = newEdges.map(edge => edge.node.id);

          // Append new IDs to existing list
          setAllProcessIds(prev => [...prev, ...newIds]);
          setHasMore(newProcesses.pageInfo?.hasNextPage ?? false);
          setEndCursor(newProcesses.pageInfo?.endCursor);

          const prevProcesses = getProcessesFromResponse(previousResult);

          return {
            hosts: {
              edges: [{
                node: {
                  processes: {
                    ...newProcesses,
                    edges: [...(prevProcesses?.edges || []), ...newEdges],
                  },
                },
              }],
            },
          };
        },
      });
    } catch (err) {
      console.error("Error loading more processes:", err);
    }
  }, [hasMore, loading, fetchMore, hostId, endCursor, searchTerm, processSort, getProcessesFromResponse]);

  const currentData = data ?? previousData;
  const processes = getProcessesFromResponse(currentData);

  return {
    data: processes,
    processIds: allProcessIds,
    initialLoading: networkStatus === NetworkStatus.loading && !currentData,
    error,
    hasMore,
    loadMore,
    refetch,
    searchTerm,
    setSearchTerm,
  };
};

const getProcessIdsQuery = (
  hostId: string,
  afterCursor?: Cursor,
  searchTerm?: string,
  sort?: OrderByField,
): GetProcessIdsQueryVariables => {
  const defaultRowLimit = TableRowLimit.HostRowLimit;

  const query: GetProcessIdsQueryVariables = {
    hostId,
    first: defaultRowLimit,
    after: afterCursor || undefined,
    ...(sort && { orderBy: [sort] }),
  };

  if (searchTerm) {
    query.where = {
      or: [
        { nameContainsFold: searchTerm.trim() },
        { pathContainsFold: searchTerm.trim() },
      ],
    };
  }

  return query;
};
