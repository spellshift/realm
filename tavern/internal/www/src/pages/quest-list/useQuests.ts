import { useQuery } from "@apollo/client";
import { useCallback, useEffect, useMemo, useState } from "react";
import { TableRowLimit } from "../../utils/enums";
import { GET_QUEST_QUERY } from "../../utils/queries";
import { Filters, useFilters } from "../../context/FilterContext";
import { constructTaskFilterQuery } from "../../utils/constructQueryUtils";
import { Cursor, GetQuestQueryVariables, QuestQueryTopLevel } from "../../utils/interfacesQuery";

export const useQuests = (pagination: boolean) => {
    const [page, setPage] = useState<number>(1);
    const { filters } = useFilters();

    const constructDefaultQuery = useCallback((afterCursor?: Cursor, beforeCursor?: Cursor, currentFilters?: Filters): GetQuestQueryVariables => {
        const defaultRowLimit = TableRowLimit.QuestRowLimit;
        const filterQueryFields = (currentFilters?.filtersEnabled) ? constructTaskFilterQuery(currentFilters) : null;

        const query: GetQuestQueryVariables = {
          where: {
            ...(currentFilters?.filtersEnabled && currentFilters.questName && {
              nameContains: currentFilters.questName
            }),
            ...(filterQueryFields || {})
          },
          whereTotalTask: {
            ...(filterQueryFields?.hasTasksWith || {})
          },
          whereFinishedTask: {
            execFinishedAtNotNil: true,
            ...(filterQueryFields?.hasTasksWith || {})
          },
          whereOutputTask: {
            outputSizeGT: 0,
            ...(filterQueryFields?.hasTasksWith || {})
          },
          whereErrorTask: {
            errorNotNil: true,
            ...(filterQueryFields?.hasTasksWith || {})
          },
          firstTask: 1,
          orderByTask: [{
            direction: "DESC",
            field: "LAST_MODIFIED_AT"
          }],
          orderBy: [{
            direction: "DESC",
            field: "CREATED_AT"
          }]
        };

        if (pagination) {
          query.first = beforeCursor ? undefined : defaultRowLimit;
          query.last = beforeCursor ? defaultRowLimit : undefined;
          query.after = afterCursor || undefined;
          query.before = beforeCursor || undefined;
        }

        return query;
    }, [pagination]);

    const queryVariables = useMemo(() => constructDefaultQuery(undefined, undefined, filters), [constructDefaultQuery, filters]);

    const { loading, data, error, refetch } = useQuery<QuestQueryTopLevel>(
      GET_QUEST_QUERY,
      {
        variables: queryVariables,
        notifyOnNetworkStatusChange: true
      }
    );

    const updateQuestList = useCallback((afterCursor?: Cursor, beforeCursor?: Cursor) => {
      const query = constructDefaultQuery(afterCursor, beforeCursor, filters);
      refetch(query);
    }, [filters, constructDefaultQuery, refetch]);

    // Reset page when filters change
    useEffect(() => {
      setPage(1);
    }, [filters]);

    return {
        data,
        loading,
        error,
        page,
        setPage,
        updateQuestList
    };
};
