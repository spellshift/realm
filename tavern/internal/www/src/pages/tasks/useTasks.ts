import { useQuery } from "@apollo/client";
import { useCallback, useEffect, useState } from "react";
import { GET_TASK_QUERY } from "../../utils/queries";
import { getFilterNameByTypes } from "../../utils/utils";

export enum TASK_PAGE_TYPE{   
    questIdQuery= "ID_QUERY",
    questDetailsQuery= "QUEST_DETAILS_QUERY",
}

export const useTasks = (defaultQuery?: TASK_PAGE_TYPE, id?: string) => {
    // store filters
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
      const defaultRowLimit = 2;
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
            case TASK_PAGE_TYPE.questIdQuery:
                const include = [{"hasQuestWith": {"id": id}}] as Array<any>;

                if(searchText){include.push({"outputContains": searchText})};

                query.where.and = include;
                break;
            case TASK_PAGE_TYPE.questDetailsQuery:
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
      const {beacon: beacons, group: groups, service: services, platform: platforms, hosts=[]} = getFilterNameByTypes(filtersSelected);

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
    },[search, filtersSelected, constructDefaultQuery, refetch]);


    useEffect(()=> {
        updateTaskList();
    },[updateTaskList]);



    return {
        data,
        loading,
        error,
        page,
        setPage,
        setSearch: handleSearchChange,
        setFiltersSelected: handleFilterChange,
        updateTaskList
    }
};