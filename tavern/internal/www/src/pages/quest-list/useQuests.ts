import { useQuery } from "@apollo/client";
import { useCallback, useEffect,  useState } from "react";
import { TableRowLimit } from "../../utils/enums";
import { GET_QUEST_BY_ID_QUERY, GET_QUEST_QUERY } from "../../utils/queries";
import { Filters, useFilters } from "../../context/FilterContext";
import { constructTaskFilterQuery } from "../../utils/constructQueryUtils";

export const useQuests = (pagination: boolean, id?: string) => {
    const [page, setPage] = useState<number>(1);
    const {filters} = useFilters();

    const constructDefaultQuery = useCallback((afterCursor?: string | undefined, beforeCursor?: string | undefined, filters?: Filters) => {
        const defaultRowLimit = TableRowLimit.QuestRowLimit;
        const filterQueryFields = (filters && filters.filtersEnabled) && constructTaskFilterQuery(filters);

        const query = {
          "where": {
            ...(id && {"id": id}),
            ...((filters?.filtersEnabled && filters.questName) && {
              "nameContains": filters.questName
            }),
            ...(filterQueryFields && filterQueryFields)
          },
          "whereTotalTask": {
            ...(filterQueryFields && filterQueryFields.hasTasksWith)
          },
          "whereFinishedTask": {
            "execFinishedAtNotNil": true,
            ...(filterQueryFields && filterQueryFields.hasTasksWith)
          },
          "whereOutputTask":{
            "outputSizeGT": 0,
            ...(filterQueryFields && filterQueryFields.hasTasksWith)
          },
          "whereErrorTask": {
            "errorNotNil": true,
            ...(filterQueryFields && filterQueryFields.hasTasksWith)
          },
          firstTask: 1,
          orderByTask: [{
            "direction": "DESC",
            "field": "LAST_MODIFIED_AT"
          }],
          "orderBy": [{
            "direction": "DESC",
            "field": "CREATED_AT"
          }]
        } as any;

        if(pagination){
          query.first = beforeCursor ? null : defaultRowLimit;
          query.last =  beforeCursor ? defaultRowLimit : null;
          query.after = afterCursor ? afterCursor : null;
          query.before = beforeCursor ? beforeCursor : null;
        }

        return query
    },[pagination, id]);

    const { loading, data, error, refetch } = useQuery(
      id ? GET_QUEST_BY_ID_QUERY : GET_QUEST_QUERY, {variables: constructDefaultQuery(),  notifyOnNetworkStatusChange: true}
      );

    const updateQuestList = useCallback((afterCursor?: string | undefined, beforeCursor?: string | undefined) => {
      const query = constructDefaultQuery(afterCursor, beforeCursor, filters);
      refetch(query);
    },[filters, constructDefaultQuery, refetch]);

    useEffect(()=> {
      updateQuestList();
    },[updateQuestList]);

    useEffect(()=>{
      setPage(1);
    },[filters])

    return {
        data,
        loading,
        error,
        page,
        setPage,
        updateQuestList
    }
}
