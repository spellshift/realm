import { useQuery } from "@apollo/client";
import { useCallback, useEffect, useMemo, useState } from "react";
import { useLocation } from "react-router-dom";
import { FilterBarOption } from "../utils/consts";
import { TableRowLimit } from "../utils/enums";
import { GET_QUEST_BY_ID_QUERY, GET_QUEST_QUERY } from "../utils/queries";
import { getFilterNameByTypes } from "../utils/utils";

export const useQuests = (pagination: boolean, id?: string) => {
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
        const defaultRowLimit = TableRowLimit.QuestRowLimit;
        const query = {
          "where": {
            "and": [] as Array<any>
          },
          "whereTotalTask": {
            "and": [] as Array<any>
          },
          "whereFinishedTask": {
            "and": [
              {"execFinishedAtNotNil": true}
            ]
          },
          "whereOutputTask":{
            "and": [
              {"outputSizeGT": 0}
            ]
          },
          "whereErrorTask": {
            "and": [
              {"errorNotNil": true}
            ]
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

        const whereParams = [];

        if(id){
          whereParams.push({"id": id});
        }

        if(searchText){
          whereParams.push({
            "or": [
            {"nameContains": searchText},
            {"hasTomeWith": {"nameContains": searchText}}
          ]
          });
        };

        query.where.and = query.where.and.concat(whereParams);

        return query
    },[pagination, id]);

    const constructBeaconFilterQuery = useCallback((query: any, beacons: any)=>{
      const fq = query;

      if(beacons.length > 0){
            const beaconList = {
              "hasBeaconWith": {"nameIn": beacons}
            };

            fq.where.and = fq.where.and.concat(
                {
                  "hasTasksWith": beaconList
                }
            );

            fq.whereFinishedTask.and = fq.whereFinishedTask.and.concat(
              beaconList
            );
            fq.whereOutputTask.and = fq.whereOutputTask.and.concat(
              beaconList
            );
            fq.whereErrorTask.and =fq.whereErrorTask.and.concat(
              beaconList
            );
            fq.whereTotalTask.and = fq.whereTotalTask.and.concat(
              beaconList
            );

      };
      return fq;
    },[])

    const constructTagFilterQuery = useCallback((query: any, tags: any, tagKind: string)=>{
      const fq = query;
      if(tags.length > 0){
        const tagList = {
          "hasBeaconWith": { "hasHostWith": {
            "hasTagsWith": {
              "and": [
                {"kind": tagKind},
                {"nameIn": tags}
              ]
            }
          }}
        };

        fq.where.and = fq.where.and.concat(
          {
            "hasTasksWith": tagList
          }
        );

        fq.whereFinishedTask.and = fq.whereFinishedTask.and.concat(
          tagList
        );
        fq.whereOutputTask.and = fq.whereOutputTask.and.concat(
          tagList
        );
        fq.whereErrorTask.and =fq.whereErrorTask.and.concat(
          tagList
        );
        fq.whereTotalTask.and = fq.whereTotalTask.and.concat(
          tagList
        );
      };
      return fq;
    },[]);

    const constructHostFilterQuery = useCallback((query: any, hosts: any)=>{
      const fq = query;

      if(hosts.length > 0){
        const hostsList = {
          "hasBeaconWith": {
            "hasHostWith": {"nameIn": hosts}
          }
        };
        fq.where.and = fq.where.and.concat(
          {
            "hasTasksWith": hostsList
          }
        );
        fq.whereFinishedTask.and = fq.whereFinishedTask.and.concat(
          hostsList
        );
        fq.whereOutputTask.and = fq.whereOutputTask.and.concat(
          hostsList
        );
        fq.whereErrorTask.and =fq.whereErrorTask.and.concat(
          hostsList
        );
        fq.whereTotalTask.and = fq.whereTotalTask.and.concat(
          hostsList
        );
    };
    return fq;

    },[]);

    const constructPlatformFilterQuery = useCallback((query: any, platforms: any)=>{
      const fq = query;
      const platformList = {
        "hasBeaconWith": {
          "hasHostWith": {
            "platformIn": platforms
          }
        }
      };

      if(platforms.length > 0){
        fq.where.and = fq.where.and.concat(
          {
            "hasTasksWith": platformList
          }
        );
        fq.whereFinishedTask.and = fq.whereFinishedTask.and.concat(
          platformList
        );
        fq.whereOutputTask.and = fq.whereOutputTask.and.concat(
          platformList
        );
        fq.whereErrorTask.and =fq.whereErrorTask.and.concat(
          platformList
        );
        fq.whereTotalTask.and = fq.whereTotalTask.and.concat(
          platformList
        );

      }
      return fq;
    },[]);

    const constructFilterBasedQuery = useCallback((filtersSelected: Array<any>, currentQuery: any) => {
      let fq = currentQuery;
      const {beacon: beacons, group: groups, service: services, platform: platforms, host:hosts} = getFilterNameByTypes(filtersSelected);

      fq = constructBeaconFilterQuery(fq, beacons);
      fq = constructTagFilterQuery(fq, groups, "group");
      fq = constructTagFilterQuery(fq, services, "service");
      fq = constructHostFilterQuery(fq, hosts);
      fq = constructPlatformFilterQuery(fq, platforms);

      return fq;
    },[constructBeaconFilterQuery, constructTagFilterQuery, constructHostFilterQuery, constructPlatformFilterQuery]);


    const { loading, data, error, refetch } = useQuery(
      id ? GET_QUEST_BY_ID_QUERY : GET_QUEST_QUERY, {variables: constructDefaultQuery(),  notifyOnNetworkStatusChange: true}
      );

    const updateQuestList = useCallback((afterCursor?: string | undefined, beforeCursor?: string | undefined) => {
      const defaultQuery = constructDefaultQuery(search, afterCursor, beforeCursor);
      // Add filter handling
      const queryWithFilter =  constructFilterBasedQuery(filtersSelected , defaultQuery) as any;
      refetch(queryWithFilter);
    },[search, filtersSelected, constructDefaultQuery, constructFilterBasedQuery, refetch]);

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
