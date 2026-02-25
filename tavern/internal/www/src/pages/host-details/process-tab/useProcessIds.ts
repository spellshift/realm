import { useQuery, NetworkStatus } from "@apollo/client";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
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
  const isLoadingMore = useRef(false);

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
    isLoadingMore.current = false;
  }, [hostId, searchTerm, processSort]);

  const loadMore = useCallback(async () => {
    if (!hasMore || loading || isLoadingMore.current) return;

    isLoadingMore.current = true;
    try {
      const result = await fetchMore({
        variables: getProcessIdsQuery(hostId, endCursor, searchTerm, processSort),
        updateQuery: (previousResult, { fetchMoreResult }) => {
          if (!fetchMoreResult) return previousResult;

          const newProcesses = getProcessesFromResponse(fetchMoreResult);
          if (!newProcesses) return previousResult;

          const newEdges = newProcesses.edges;
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

      const newProcesses = getProcessesFromResponse(result.data);
      if (newProcesses?.edges && newProcesses.edges.length === 0) {
        setHasMore(false);
      }
    } catch (err) {
      console.error("Error loading more processes:", err);
      setHasMore(false);
    } finally {
      isLoadingMore.current = false;
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
