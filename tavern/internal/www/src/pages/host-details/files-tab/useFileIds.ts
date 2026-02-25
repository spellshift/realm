import { useQuery, NetworkStatus } from "@apollo/client";
import { useCallback, useEffect, useMemo, useState } from "react";
import { PageNavItem, TableRowLimit } from "../../../utils/enums";
import { Cursor, OrderByField } from "../../../utils/interfacesQuery";
import { useSorts } from "../../../context/SortContext";
import { GET_FILE_IDS_QUERY } from "./queries";
import { FileIdsQueryResponse, GetFileIdsQueryVariables } from "./types";

export const useFileIds = (hostId: string) => {
  const { sorts } = useSorts();
  const fileSort = sorts[PageNavItem.files];

  const [allFileIds, setAllFileIds] = useState<string[]>([]);
  const [hasMore, setHasMore] = useState(true);
  const [endCursor, setEndCursor] = useState<Cursor | undefined>(undefined);
  const [searchTerm, setSearchTerm] = useState<string>("");

  const queryVariables = useMemo(
    () => getFileIdsQuery(hostId, undefined, searchTerm, fileSort),
    [hostId, searchTerm, fileSort]
  );

  const { data, previousData, error, fetchMore, networkStatus, loading, refetch } = useQuery<FileIdsQueryResponse>(
    GET_FILE_IDS_QUERY,
    {
      variables: queryVariables,
      notifyOnNetworkStatusChange: true,
      fetchPolicy: 'cache-and-network',
      skip: !hostId,
    }
  );

  const getFilesFromResponse = useCallback((response: FileIdsQueryResponse | undefined) => {
    return response?.hosts?.edges?.[0]?.node?.files;
  }, []);

  useEffect(() => {
    const currentData = data ?? previousData;
    const files = getFilesFromResponse(currentData);
    if (files?.edges) {
      const ids = files.edges.map(edge => edge.node.id);
      setAllFileIds(ids);
      setHasMore(files.pageInfo?.hasNextPage ?? false);
      setEndCursor(files.pageInfo?.endCursor);
    }
  }, [data, previousData, getFilesFromResponse]);

  useEffect(() => {
    setAllFileIds([]);
    setHasMore(true);
    setEndCursor(undefined);
  }, [hostId, searchTerm, fileSort]);

  const loadMore = useCallback(async () => {
    if (!hasMore || loading) return;

    try {
      await fetchMore({
        variables: getFileIdsQuery(hostId, endCursor, searchTerm, fileSort),
        updateQuery: (previousResult, { fetchMoreResult }) => {
          if (!fetchMoreResult) return previousResult;

          const newFiles = getFilesFromResponse(fetchMoreResult);
          if (!newFiles) return previousResult;

          const newEdges = newFiles.edges;
          const newIds = newEdges.map(edge => edge.node.id);

          // Append new IDs to existing list
          setAllFileIds(prev => [...prev, ...newIds]);
          setHasMore(newFiles.pageInfo?.hasNextPage ?? false);
          setEndCursor(newFiles.pageInfo?.endCursor);

          const prevFiles = getFilesFromResponse(previousResult);

          return {
            hosts: {
              edges: [{
                node: {
                  files: {
                    ...newFiles,
                    edges: [...(prevFiles?.edges || []), ...newEdges],
                  },
                },
              }],
            },
          };
        },
      });
    } catch (err) {
      console.error("Error loading more files:", err);
    }
  }, [hasMore, loading, fetchMore, hostId, endCursor, searchTerm, fileSort, getFilesFromResponse]);

  const currentData = data ?? previousData;
  const files = getFilesFromResponse(currentData);

  return {
    data: files,
    fileIds: allFileIds,
    initialLoading: networkStatus === NetworkStatus.loading && !currentData,
    error,
    hasMore,
    loadMore,
    refetch,
    searchTerm,
    setSearchTerm,
  };
};

const getFileIdsQuery = (
  hostId: string,
  afterCursor?: Cursor,
  searchTerm?: string,
  sort?: OrderByField,
): GetFileIdsQueryVariables => {
  const defaultRowLimit = TableRowLimit.HostRowLimit;

  const query: GetFileIdsQueryVariables = {
    hostId,
    first: defaultRowLimit,
    after: afterCursor || undefined,
    ...(sort && { orderBy: [sort] }),
  };

  if (searchTerm) {
    query.where = {
      or: [
        { pathContainsFold: searchTerm.trim() },
        { hashContainsFold: searchTerm.trim() },
      ],
    };
  }

  return query;
};
