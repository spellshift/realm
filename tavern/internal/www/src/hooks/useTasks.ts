import { useQuery } from "@apollo/client";
import { useCallback, useEffect, useState } from "react";
import { DEFAULT_QUERY_TYPE, TableRowLimit } from "../utils/enums";
import { GET_TASK_QUERY } from "../utils/queries";
import { useFilters } from "../context/FilterContext";
import { constructTaskFilterQuery } from "../utils/constructQueryUtils";
import { Cursor } from "../utils/interfacesQuery";


export const useTasks = (defaultQuery?: DEFAULT_QUERY_TYPE, id?: string) => {
    const [page, setPage] = useState<number>(1);
    const {filters} = useFilters();

    const constructDefaultQuery = useCallback((afterCursor?: Cursor, beforeCursor?: Cursor) => {
      const defaultRowLimit = TableRowLimit.TaskRowLimit;
      const filterQueryFields = (filters && filters.filtersEnabled) && constructTaskFilterQuery(filters);

      const query = {
        "where": {
          ...filterQueryFields && filterQueryFields.hasTasksWith,
          ...(defaultQuery === DEFAULT_QUERY_TYPE.questIdQuery && id) && {"hasQuestWith": {"id": id}},
        },
        "first": beforeCursor ? null : defaultRowLimit,
        "last": beforeCursor ? defaultRowLimit : null,
        "after": afterCursor ? afterCursor : null,
        "before": beforeCursor ? beforeCursor : null,
        "orderBy": [{
          "direction": "DESC",
          "field": "LAST_MODIFIED_AT"
        }]
      } as any;

      if(defaultQuery === DEFAULT_QUERY_TYPE.hostIDQuery && id){
        query.where.hasBeaconWith ??= {};
        query.where.hasBeaconWith.hasHostWith ??= {"id": id};
      }

      return query;
    },[defaultQuery, id, filters]);


    const { loading, error, data, refetch} = useQuery(GET_TASK_QUERY,  {variables: constructDefaultQuery(),  notifyOnNetworkStatusChange: true});

    const updateTaskList = useCallback((afterCursor?: Cursor, beforeCursor?: Cursor) => {
        const query = constructDefaultQuery(afterCursor, beforeCursor);
        refetch(query);
    },[constructDefaultQuery, refetch]);


    useEffect(()=> {
        updateTaskList();
    },[updateTaskList]);

    useEffect(()=>{
      setPage(1);
    },[filters])

    return {
        data,
        loading,
        error,
        page,
        setPage,
        updateTaskList
    }
};
