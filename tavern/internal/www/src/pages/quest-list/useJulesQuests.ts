import { useQuery } from "@apollo/client";
import { useCallback, useEffect, useMemo, useState } from "react";
import { PageNavItem, TableRowLimit } from "../../utils/enums";
import { GET_QUEST_QUERY } from "../../utils/queries";
import { Filters, useFilters } from "../../context/FilterContext";
import { constructQuestFilterQuery, constructTaskFilterQuery } from "../../utils/constructQueryUtils";
import { Cursor, GetQuestQueryVariables, OrderByField, QuestQueryTopLevel, QuestEdge } from "../../utils/interfacesQuery";
import { useSorts } from "../../context/SortContext";
import { useTags } from "../../context/TagContext";

export const useJulesQuests = () => {
    const { filters } = useFilters();
    const { sorts } = useSorts();
    const { lastFetchedTimestamp } = useTags();

    const constructQuery = useCallback((afterCursor?: Cursor, currentFilters?: Filters, sort?: OrderByField, currentTimestamp?: Date): GetQuestQueryVariables => {
        const defaultRowLimit = TableRowLimit.QuestRowLimit; // Usually 20 or 50
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
          first: defaultRowLimit,
          after: afterCursor,
          ...(sort && {orderBy: [sort]})
        };

        return query;
    }, []);

    const questSort = sorts[PageNavItem.quests];
    // Initial query variables (Page 1)
    const queryVariables = useMemo(() => constructQuery(undefined, filters, questSort, lastFetchedTimestamp), [constructQuery, filters, questSort, lastFetchedTimestamp]);

    const { loading, data, error, fetchMore } = useQuery<QuestQueryTopLevel>(
      GET_QUEST_QUERY,
      {
        variables: queryVariables,
        pollInterval: 1000, // Aggressive polling <1s
        notifyOnNetworkStatusChange: true,
      }
    );

    const loadMore = useCallback(() => {
        if (!data?.quests?.pageInfo?.hasNextPage) return;

        const endCursor = data.quests.pageInfo.endCursor;

        fetchMore({
            variables: {
                after: endCursor,
            },
            updateQuery: (prev, { fetchMoreResult }) => {
                if (!fetchMoreResult) return prev;

                return {
                    ...prev,
                    quests: {
                        ...prev.quests,
                        totalCount: fetchMoreResult.quests.totalCount,
                        pageInfo: fetchMoreResult.quests.pageInfo,
                        edges: [
                            ...prev.quests.edges,
                            ...fetchMoreResult.quests.edges
                        ]
                    }
                };
            }
        });
    }, [data, fetchMore]);


    return {
        data,
        loading,
        error,
        loadMore,
        hasNextPage: data?.quests?.pageInfo?.hasNextPage ?? false,
    };
};
