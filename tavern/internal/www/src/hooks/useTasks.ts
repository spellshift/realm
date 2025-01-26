import { useQuery } from "@apollo/client";
import { useCallback, useEffect, useMemo, useState } from "react";
import { useLocation } from "react-router-dom";
import { FilterBarOption } from "../utils/consts";
import { DEFAULT_QUERY_TYPE, TableRowLimit } from "../utils/enums";
import { GET_TASK_QUERY } from "../utils/queries";
import { getFilterNameByTypes } from "../utils/utils";


export const useTasks = (defaultQuery?: DEFAULT_QUERY_TYPE, id?: string) => {
    // store filters
    const {state} = useLocation();
    const [page, setPage] = useState<number>(1);
    const [search, setSearch] = useState("");

    const defaultFilter = useMemo(() : Array<FilterBarOption> => {
      const allTrue  = state && Array.isArray(state) && state.every((stateItem: FilterBarOption) => 'kind' in stateItem && 'value' in stateItem && 'name' in stateItem);
      if(allTrue){
          return state;
      }
      else{
          return [];
      }
    },[state]);

    const [filtersSelected, setFiltersSelected] = useState<Array<any>>(defaultFilter);

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
          "field": "LAST_MODIFIED_AT"
        }]
      } as any;

      switch(defaultQuery){
            case DEFAULT_QUERY_TYPE.hostIDQuery:
              const hostParams = [{
                  "hasBeaconWith": {
                      "hasHostWith": {
                          "id": id
                      }
                  }
              }] as Array<any>;

              if(searchText){
                hostParams.push({
                "or": [
                  {"outputContains": searchText},
                  {"hasQuestWith": {
                      "nameContains": searchText
                    }
                  },
                  {"hasQuestWith":
                    {"hasTomeWith": {"nameContains": searchText}}}
                ]
              })};

              query.where.and = hostParams;
              break;
            case DEFAULT_QUERY_TYPE.questIdQuery:
                const include = [{"hasQuestWith": {"id": id}}] as Array<any>;

                if(searchText){include.push({"outputContains": searchText})};

                query.where.and = include;
                break;
            case DEFAULT_QUERY_TYPE.questDetailsQuery:
            default:
                const text = searchText || "";
                query.where.and = [{
                                "or": [
                                  {"outputContains": text},
                                  {"hasQuestWith": {
                                      "nameContains": text
                                    }
                                  },
                                  {"hasQuestWith":
                                    {"hasTomeWith": {"nameContains": text}}}
                                ]
                }];
                break;
        }
        return query;
    },[defaultQuery, id]);

    const constructFilterBasedQuery = useCallback((filtersSelected: Array<any>, currentQuery: any) => {
      const fq = currentQuery;
      const {beacon: beacons, group: groups, service: services, platform: platforms, host:hosts} = getFilterNameByTypes(filtersSelected);

      if(beacons.length > 0){
            fq.where.and = fq.where.and.concat(
                {
                "hasBeaconWith": {"nameIn": beacons}
                }
            );
      }

      if(groups.length > 0){
          fq.where.and = fq.where.and.concat(
              {
                  "hasBeaconWith": { "hasHostWith": {
                    "hasTagsWith": {
                      "and": [
                        {"kind": "group"},
                        {"nameIn": groups}
                      ]
                    }
                  }}
              }
          );
      }

      if(services.length > 0){
          fq.where.and = fq.where.and.concat(
              {
                  "hasBeaconWith": { "hasHostWith": {
                    "hasTagsWith": {
                      "and": [
                        {"kind": "service"},
                        {"nameIn": services}
                      ]
                    }
                  }}
              }
          );
      }

      if(hosts.length > 0){
          fq.where.and = fq.where.and.concat(
              {
                  "hasBeaconWith": {
                    "hasHostWith": {"nameIn": hosts}
                  }
              }
          );
      }

      if(platforms.length > 0){
          fq.where.and = fq.where.and.concat(
              {
                  "hasBeaconWith": {
                    "hasHostWith": {
                      "platformIn": platforms
                    }
                  }
                }
          );
      }
      return fq;
    },[]);

    // get tasks
    const { loading, error, data, refetch} = useQuery(GET_TASK_QUERY,  {variables: constructDefaultQuery(),  notifyOnNetworkStatusChange: true});

    const updateTaskList = useCallback((afterCursor?: string | undefined, beforeCursor?: string | undefined) => {
        const defaultQuery = constructDefaultQuery(search, afterCursor, beforeCursor);
        const queryWithFilter =  constructFilterBasedQuery(filtersSelected , defaultQuery) as any;
        refetch(queryWithFilter);
    },[search, filtersSelected, constructDefaultQuery, constructFilterBasedQuery, refetch]);


    useEffect(()=> {
        updateTaskList();
    },[updateTaskList]);



    return {
        data,
        loading,
        error,
        page,
        filtersSelected,
        setPage,
        setSearch: handleSearchChange,
        setFiltersSelected: handleFilterChange,
        updateTaskList
    }
};
