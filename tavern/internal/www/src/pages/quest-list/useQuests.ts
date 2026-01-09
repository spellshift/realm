import { useQuery } from "@apollo/client";
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
    const {lastFetchedTimestamp} = useTags();

    const constructDefaultQuery = useCallback((afterCursor?: Cursor, beforeCursor?: Cursor, currentFilters?: Filters, sort?: OrderByField, currentTimestamp?: Date): GetQuestQueryVariables => {
        const defaultRowLimit = TableRowLimit.QuestRowLimit;
        const filterQueryTaskFields = (currentFilters?.filtersEnabled) ? constructTaskFilterQuery(currentFilters, currentTimestamp) : null;
        const questFilterFields = (currentFilters?.filtersEnabled) ? constructQuestFilterQuery(currentFilters) : null;

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
          ...(sort && {orderBy: [sort]})
        };

        if (pagination) {
          query.first = beforeCursor ? undefined : defaultRowLimit;
          query.last = beforeCursor ? defaultRowLimit : undefined;
          query.after = afterCursor || undefined;
          query.before = beforeCursor || undefined;
        }

        return query;
    }, [pagination]);

    const questSort = sorts[PageNavItem.quests];
    const queryVariables = useMemo(() => constructDefaultQuery(undefined, undefined, filters, questSort, lastFetchedTimestamp), [constructDefaultQuery, filters, questSort, lastFetchedTimestamp]);

    const { loading, data, error, refetch } = useQuery<QuestQueryTopLevel>(
      GET_QUEST_QUERY,
      {
        variables: queryVariables,
        notifyOnNetworkStatusChange: true,
      }
    );

    const updateQuestList = useCallback((afterCursor?: Cursor, beforeCursor?: Cursor) => {
      const query = constructDefaultQuery(afterCursor, beforeCursor, filters, questSort, lastFetchedTimestamp);
      return refetch(query);
    }, [filters, questSort, constructDefaultQuery, refetch, lastFetchedTimestamp]);

    useEffect(() => {
      const abortController = new AbortController();
      updateQuestList();

      return () => {
        abortController.abort();
      };
    }, [updateQuestList]);

    useEffect(() => {
      setPage(1);
    }, [filters, questSort]);

    return {
        data,
        loading,
        error,
        page,
        setPage,
        updateQuestList
    };
};
