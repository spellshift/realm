import { useQuery, NetworkStatus } from "@apollo/client";
import { useCallback, useEffect, useMemo, useState } from "react";
import { PageNavItem, TableRowLimit } from "../../utils/enums";
import { GET_QUEST_QUERY } from "../../utils/queries";
import { Filters, useFilters } from "../../context/FilterContext";
import { constructQuestFilterQuery, constructTaskFilterQuery } from "../../utils/constructQueryUtils";
import { Cursor, GetQuestQueryVariables, OrderByField, QuestQueryTopLevel } from "../../utils/interfacesQuery";
import { useSorts } from "../../context/SortContext";
import { useTags } from "../../context/TagContext";

export const useQuests = (pagination: boolean) => {
  const [page, setPage] = useState<number>(1);
  const { filters } = useFilters();
  const { sorts } = useSorts();
  const { lastFetchedTimestamp } = useTags();
  const questSort = sorts[PageNavItem.quests];

  const queryVariables = useMemo(
    () => getDefaultQuestQuery(filters, pagination, undefined, undefined, questSort, lastFetchedTimestamp),
    [filters, pagination, questSort, lastFetchedTimestamp]
  );

  const { data, previousData, error, refetch, networkStatus } = useQuery<QuestQueryTopLevel>(
    GET_QUEST_QUERY,
    {
      variables: queryVariables,
      notifyOnNetworkStatusChange: true,
      fetchPolicy: 'cache-and-network',
    }
  );

  const updateQuestList = useCallback((afterCursor?: Cursor, beforeCursor?: Cursor) => {
    return refetch(
      getDefaultQuestQuery(filters, pagination, afterCursor, beforeCursor, questSort, lastFetchedTimestamp)
    );
  }, [filters, pagination, questSort, lastFetchedTimestamp, refetch]);

  useEffect(() => {
    setPage(prev => prev !== 1 ? 1 : prev);
  }, [filters, questSort]);

  const currentData = data ?? previousData;

  return {
    data: currentData,
    loading: networkStatus === NetworkStatus.loading && !currentData,
    error,
    page,
    setPage,
    updateQuestList
  };
};

const getDefaultQuestQuery = (
  filters: Filters,
  pagination: boolean,
  afterCursor?: Cursor,
  beforeCursor?: Cursor,
  sort?: OrderByField,
  currentTimestamp?: Date
): GetQuestQueryVariables => {
  const defaultRowLimit = TableRowLimit.QuestRowLimit;
  const filterQueryTaskFields = constructTaskFilterQuery(filters, currentTimestamp);
  const questFilterFields = constructQuestFilterQuery(filters);

  const query: GetQuestQueryVariables = {
    where: {
      ...(questFilterFields || {}),
      ...(filterQueryTaskFields || {})
    },
    whereTotalTask: {
      ...(filterQueryTaskFields?.hasTasksWith || {})
    },
    whereFinishedTask: {
      execFinishedAtNotNil: true,
      ...(filterQueryTaskFields?.hasTasksWith || {})
    },
    whereOutputTask: {
      outputSizeGT: 0,
      ...(filterQueryTaskFields?.hasTasksWith || {})
    },
    whereErrorTask: {
      errorNotNil: true,
      ...(filterQueryTaskFields?.hasTasksWith || {})
    },
    firstTask: 1,
    ...(sort && { orderBy: [sort] })
  };

  if (pagination) {
    query.first = beforeCursor ? undefined : defaultRowLimit;
    query.last = beforeCursor ? defaultRowLimit : undefined;
    query.after = afterCursor || undefined;
    query.before = beforeCursor || undefined;
  }

  return query;
};
