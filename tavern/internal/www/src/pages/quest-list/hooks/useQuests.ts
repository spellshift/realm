import { useQuery } from "@apollo/client";
import { useCallback, useEffect, useState } from "react";
import { TableRowLimit } from "../../../utils/enums";
import { GET_QUEST_QUERY } from "../../../utils/queries";

export const useQuests = () => {
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
        const defaultRowLimit = TableRowLimit.QuestRowLimit;
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
        const whereParams = [];

        if(searchText){
          whereParams.push({
            "or": [
            {"hasTasksWith": {"outputContains": searchText}},
            {"nameContains": searchText},
            {"hasTomeWith": {"nameContains": searchText}}
          ]
          });
        };

        query.where.and = whereParams;

        return query
    },[]);

    const { loading, data, error, refetch } = useQuery(GET_QUEST_QUERY,{variables: constructDefaultQuery(),  notifyOnNetworkStatusChange: true});

    const updateQuestList = useCallback((afterCursor?: string | undefined, beforeCursor?: string | undefined) => {
      const defaultQuery = constructDefaultQuery(search, afterCursor, beforeCursor);
      // Add filter handling
      //const queryWithFilter =  constructFilterBasedQuery(filtersSelected , defaultQuery) as any;
      refetch(defaultQuery);
    },[search, filtersSelected, constructDefaultQuery, refetch]);

    useEffect(()=> {
      updateQuestList();
  },[updateQuestList]);

    return {
        data,
        loading,
        error,
        page,
        filtersSelected,
        setPage,
        setSearch: handleSearchChange,
        setFiltersSelected: handleFilterChange,
        updateQuestList
    }
}
