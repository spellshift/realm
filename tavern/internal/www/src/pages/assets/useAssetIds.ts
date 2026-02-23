import { useQuery, NetworkStatus } from "@apollo/client";
import { useCallback, useEffect, useMemo, useState } from "react";
import { PageNavItem, TableRowLimit } from "../../utils/enums";
import { useFilters } from "../../context/FilterContext";
import { Cursor, OrderByField } from "../../utils/interfacesQuery";
import { useSorts } from "../../context/SortContext";
import { GET_ASSET_IDS_QUERY } from "./queries";
import { AssetIdsQueryTopLevel, GetAssetIdsQueryVariables } from "./types";

export const useAssetIds = () => {
  const { filters } = useFilters();
  const { sorts } = useSorts();
  const assetSort = sorts[PageNavItem.assets];

  const [allAssetIds, setAllAssetIds] = useState<string[]>([]);
  const [hasMore, setHasMore] = useState(true);
  const [endCursor, setEndCursor] = useState<Cursor | undefined>(undefined);

  const queryVariables = useMemo(
    () => getAssetIdsQuery(filters, undefined, assetSort),
    [filters, assetSort]
  );

  const { data, previousData, error, fetchMore, networkStatus, loading, refetch } = useQuery<AssetIdsQueryTopLevel>(
    GET_ASSET_IDS_QUERY,
    {
      variables: queryVariables,
      notifyOnNetworkStatusChange: true,
      fetchPolicy: 'cache-and-network',
    }
  );

  useEffect(() => {
    const currentData = data ?? previousData;
    if (currentData?.assets?.edges) {
      const ids = currentData.assets.edges.map(edge => edge.node.id);
      setAllAssetIds(ids);
      setHasMore(currentData.assets.pageInfo?.hasNextPage ?? false);
      setEndCursor(currentData.assets.pageInfo?.endCursor);
    }
  }, [data, previousData]);

  useEffect(() => {
    setAllAssetIds([]);
    setHasMore(true);
    setEndCursor(undefined);
  }, [filters, assetSort]);

  const loadMore = useCallback(async () => {
    if (!hasMore || loading) return;

    try {
      await fetchMore({
        variables: getAssetIdsQuery(filters, endCursor, assetSort),
        updateQuery: (previousResult, { fetchMoreResult }) => {
          if (!fetchMoreResult) return previousResult;

          const newEdges = fetchMoreResult.assets.edges;
          const newIds = newEdges.map(edge => edge.node.id);

          // Append new IDs to existing list
          setAllAssetIds(prev => [...prev, ...newIds]);
          setHasMore(fetchMoreResult.assets.pageInfo?.hasNextPage ?? false);
          setEndCursor(fetchMoreResult.assets.pageInfo?.endCursor);

          return {
            assets: {
              ...fetchMoreResult.assets,
              edges: [...(previousResult.assets?.edges || []), ...newEdges],
            },
          };
        },
      });
    } catch (err) {
      console.error("Error loading more assets:", err);
    }
  }, [hasMore, loading, fetchMore, filters, endCursor, assetSort]);

  const currentData = data ?? previousData;

  return {
    data: currentData,
    assetIds: allAssetIds,
    initialLoading: networkStatus === NetworkStatus.loading && !currentData,
    error,
    hasMore,
    loadMore,
    refetch,
  };
};

const getAssetIdsQuery = (
  filters: any,
  afterCursor?: Cursor,
  sort?: OrderByField,
): GetAssetIdsQueryVariables => {
  const defaultRowLimit = TableRowLimit.AssetRowLimit;

  const where: any = {};

  if (filters.assetName) {
    where.nameContains = filters.assetName;
  }

  if (filters.userId) {
    where.hasCreatorWith = [{ id: filters.userId }];
  }

  const query: GetAssetIdsQueryVariables = {
    where: Object.keys(where).length > 0 ? where : undefined,
    ...(sort && { orderBy: [sort] }),
    first: defaultRowLimit,
    after: afterCursor || undefined,
  };

  return query;
};
