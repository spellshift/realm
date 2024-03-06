import { useQuery } from "@apollo/client";
import { useCallback, useState } from "react";
import { TableRowLimit } from "../../../utils/enums";
import { GET_QUEST_QUERY } from "../../../utils/queries";

export const useAltQuest = () => {
    const [page, setPage] = useState<number>(1);
    const [search, setSearch] = useState("");
    const [filtersSelected, setFiltersSelected] = useState<Array<any>>([]);

    const handleFilterChange = (filters: Array<any>)=> {
        setPage(1);
        setFiltersSelected(filters);
      }

      const handleSearchChange = (search: string)=> {
        setPage(1);
        setSearch(search);
      }

    const constructDefaultQuery = useCallback((searchText?: string, afterCursor?: string | undefined, beforeCursor?: string | undefined) => {

        const defaultRowLimit = TableRowLimit.TaskRowLimit;
        const query = {
          "where": {
            "and": [] as Array<any>
          },
          "first": beforeCursor ? null : defaultRowLimit,
          "last": beforeCursor ? defaultRowLimit : null,
          "after": afterCursor ? afterCursor : null,
          "before": beforeCursor ? beforeCursor : null,
          "orderBy": [{
            "direction": "DESC",
            "field": "CREATED_AT"
          }]
        } as any;

        return query
    },[]);
    const { loading, data, error } = useQuery(GET_QUEST_QUERY,{variables: constructDefaultQuery(),  notifyOnNetworkStatusChange: true});

    return {
        data,
        loading,
        error,
        page,
        filtersSelected,
        setPage,
        setSearch: handleSearchChange,
        setFiltersSelected: handleFilterChange,
    }
}
