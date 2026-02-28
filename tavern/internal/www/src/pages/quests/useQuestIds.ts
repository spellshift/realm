import { useQuery, NetworkStatus } from "@apollo/client";
import { useCallback, useEffect, useMemo, useState } from "react";
import { PageNavItem, TableRowLimit } from "../../utils/enums";
import { Filters, useFilters } from "../../context/FilterContext";
import { constructQuestFilterQuery, constructTaskFilterQuery } from "../../utils/constructQueryUtils";
import { Cursor, OrderByField } from "../../utils/interfacesQuery";
import { useSorts } from "../../context/SortContext";
import { GET_QUEST_IDS_QUERY } from "./queries";
import { QuestIdsQueryTopLevel, GetQuestIdsQueryVariables } from "./types";

export const useQuestIds = () => {
  const { filters } = useFilters();
  const { sorts } = useSorts();
  const questSort = sorts[PageNavItem.quests];

  const [allQuestIds, setAllQuestIds] = useState<string[]>([]);
  const [hasMore, setHasMore] = useState(true);
  const [endCursor, setEndCursor] = useState<Cursor | undefined>(undefined);

  const queryVariables = useMemo(
    () => getQuestIdsQuery(filters, undefined, questSort),
    [filters, questSort]
  );

  const { data, previousData, error, fetchMore, networkStatus, loading } = useQuery<QuestIdsQueryTopLevel>(
    GET_QUEST_IDS_QUERY,
    {
      variables: queryVariables,
      notifyOnNetworkStatusChange: true,
      fetchPolicy: 'cache-and-network',
    }
  );

  // When data changes, update accumulated IDs
  useEffect(() => {
    const currentData = data ?? previousData;
    if (currentData?.quests?.edges) {
      const ids = currentData.quests.edges.map(edge => edge.node.id);
      setAllQuestIds(ids);
      setHasMore(currentData.quests.pageInfo?.hasNextPage ?? false);
      setEndCursor(currentData.quests.pageInfo?.endCursor);
    }
  }, [data, previousData]);

  // Reset when filters or sort change
  useEffect(() => {
    setAllQuestIds([]);
    setHasMore(true);
    setEndCursor(undefined);
  }, [filters, questSort]);

  const loadMore = useCallback(async () => {
    if (!hasMore || loading) return;

    try {
      await fetchMore({
        variables: getQuestIdsQuery(filters, endCursor, questSort),
        updateQuery: (previousResult, { fetchMoreResult }) => {
          if (!fetchMoreResult) return previousResult;

          const newEdges = fetchMoreResult.quests.edges;
          const newIds = newEdges.map(edge => edge.node.id);

          // Append new IDs to existing list
          setAllQuestIds(prev => [...prev, ...newIds]);
          setHasMore(fetchMoreResult.quests.pageInfo?.hasNextPage ?? false);
          setEndCursor(fetchMoreResult.quests.pageInfo?.endCursor);

          return {
            quests: {
              ...fetchMoreResult.quests,
              edges: [...(previousResult.quests?.edges || []), ...newEdges],
            },
          };
        },
      });
    } catch (err) {
      console.error("Error loading more quests:", err);
    }
  }, [hasMore, loading, fetchMore, filters, endCursor, questSort]);

  const currentData = data ?? previousData;

  return {
    data: currentData,
    questIds: allQuestIds,
    initialLoading: networkStatus === NetworkStatus.loading && !currentData,
    error,
    hasMore,
    loadMore,
  };
};

const getQuestIdsQuery = (
  filters: Filters,
  afterCursor?: Cursor,
  sort?: OrderByField,
): GetQuestIdsQueryVariables => {
  const currentTimestamp = new Date();
  const defaultRowLimit = TableRowLimit.QuestRowLimit;
  const filterQueryTaskFields = constructTaskFilterQuery(filters, currentTimestamp);
  const questFilterFields = constructQuestFilterQuery(filters);

  const query: GetQuestIdsQueryVariables = {
    where: {
      ...(questFilterFields || {}),
      ...(filterQueryTaskFields || {})
    },
    ...(sort && { orderBy: [sort] }),
    first: defaultRowLimit,
    after: afterCursor || undefined,
  };

  return query;
};
