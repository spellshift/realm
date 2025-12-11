import { useQuery } from "@apollo/client";
import { useCallback, useEffect, useState } from "react";
import { PageNavItem, TableRowLimit } from "../../utils/enums";
import { GET_TASK_QUERY } from "../../utils/queries";
import { useFilters } from "../../context/FilterContext";
import { constructTaskFilterQuery } from "../../utils/constructQueryUtils";
import { Cursor } from "../../utils/interfacesQuery";
import { useSorts } from "../../context/SortContext";

export const useTasks = (id?: string) => {
    const [page, setPage] = useState<number>(1);
    const {filters} = useFilters();
    const { sorts } = useSorts();
    const taskSort = sorts[PageNavItem.tasks];

    const constructDefaultQuery = useCallback((afterCursor?: Cursor, beforeCursor?: Cursor) => {
      const defaultRowLimit = TableRowLimit.TaskRowLimit;
      const filterQueryFields = (filters && filters.filtersEnabled) && constructTaskFilterQuery(filters);

      const query = {
        "where": {
          ...filterQueryFields && filterQueryFields.hasTasksWith,
          "hasQuestWith": {"id": id},
        },
        "first": beforeCursor ? null : defaultRowLimit,
        "last": beforeCursor ? defaultRowLimit : null,
        "after": afterCursor ? afterCursor : null,
        "before": beforeCursor ? beforeCursor : null,
        ...(taskSort && {orderBy: [taskSort]})
      } as any;

      return query;
    },[id, filters, taskSort]);


    const { loading, error, data, refetch} = useQuery(GET_TASK_QUERY,  {variables: constructDefaultQuery(),  notifyOnNetworkStatusChange: true});

    const updateTaskList = useCallback((afterCursor?: Cursor, beforeCursor?: Cursor) => {
        const query = constructDefaultQuery(afterCursor, beforeCursor);
        return refetch(query);
    },[constructDefaultQuery, refetch]);


    useEffect(()=> {
        const abortController = new AbortController();
        updateTaskList();

        return () => {
            abortController.abort();
        };
    },[updateTaskList]);

    useEffect(()=>{
      setPage(1);
    },[filters, taskSort])

    return {
        data,
        loading,
        error,
        page,
        setPage,
        updateTaskList
    }
};
