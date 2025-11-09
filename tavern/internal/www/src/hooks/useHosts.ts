import { useCallback, useEffect, useMemo, useState } from "react";
import { FilterBarOption, HostType } from "../utils/consts";
import { useQuery } from "@apollo/client";
import { GET_HOST_QUERY } from "../utils/queries";
import { PrincipalAdminTypes, TableRowLimit } from "../utils/enums";
import { convertArrayOfObjectsToObject, getFilterNameByTypes, getOfflineOnlineStatus } from "../utils/utils";
import { formatDistance } from "date-fns";
import { useLocation } from "react-router-dom";

export const useHosts = (pagination: boolean, id?: string) => {
    const currentDate = new Date();
    const {state} = useLocation();
    const [page, setPage] = useState<number>(1);

    const defaultFilter = useMemo(() : Array<FilterBarOption> => {
        return getDefaultFilter(state);
    },[state]);
    const [filtersSelected, setFiltersSelected] = useState<Array<any>>(defaultFilter);

    const constructDefaultQuery = useCallback((afterCursor?: string | undefined, beforeCursor?: string | undefined) => {
        return getDefaultHostQuery(pagination, afterCursor, beforeCursor, id);
    },[pagination, id]);

    const constructFilterBasedQuery = useCallback((defaultQuery: any, filtersSelected: Array<any>) => {
        return getFilterHostQuery(defaultQuery, filtersSelected);
    },[]);

    const { loading, data, error, refetch } = useQuery(
        GET_HOST_QUERY, {variables: constructDefaultQuery(),  notifyOnNetworkStatusChange: true}
    );

    const updateHosts = useCallback((afterCursor?: string | undefined, beforeCursor?: string | undefined) => {
        const defaultQuery = constructDefaultQuery(afterCursor, beforeCursor);
        const queryWithFilter =  constructFilterBasedQuery(defaultQuery, filtersSelected);
        refetch(queryWithFilter);
      },[constructDefaultQuery, constructFilterBasedQuery, refetch, filtersSelected]);

    const handleFilterChange = (filters: Array<any>)=> {
        setPage(1);
        setFiltersSelected(filters);
    }

    useEffect(()=>{
        updateHosts();
    },[updateHosts]);


    const hostData =  {
        hasData: data ? true : false,
        hosts: data?.hosts?.edges.map((edge: { node: HostType }) => {
            return {
                ...edge?.node,
                beaconPrincipals: getFormatForPrincipal(edge?.node),
                beaconStatus: getOfflineOnlineStatus(edge?.node.beacons),
                formattedLastSeenAt: edge?.node?.lastSeenAt ? formatDistance(new Date(edge.node.lastSeenAt), currentDate) : "N/A"
            }
        }),
        pageInfo: data?.hosts.pageInfo,
        totalCount: data?.hosts?.totalCount
    };

    return {
        data: hostData,
        loading,
        error,
        page,
        setPage,
        updateHosts,
        filtersSelected,
        setFiltersSelected: handleFilterChange
    }
};


/**
 * Utility functions for useHosts
 *
*/

const getFormatForPrincipal = (host: HostType) => {
    const uniqueObject = convertArrayOfObjectsToObject(host.beacons || [], 'principal');
    const princialUserList = Object.values(PrincipalAdminTypes) as Array<string>;
    const finalList = [] as Array<string>;
    for (const property in uniqueObject) {
        if(princialUserList.indexOf(property) !== -1){
            finalList.unshift(property);
        }
        else{
            finalList.push(property);
        }
    }
    return finalList;
};

const getDefaultFilter = (state: Array<FilterBarOption>) => {
    const allTrue  = state && Array.isArray(state) && state.every((stateItem: FilterBarOption) => 'kind' in stateItem && 'value' in stateItem && 'name' in stateItem);
    if(allTrue){
        return state;
    }
    else{
        return [];
    }
};

const getDefaultHostQuery = (pagination: boolean, afterCursor?: string | undefined, beforeCursor?: string | undefined, id?: string | undefined) => {
    const defaultRowLimit = TableRowLimit.HostRowLimit;
    const query = {
      "where": {
        and: []
      },
    } as any;

    if(pagination){
      query.first = beforeCursor ? null : defaultRowLimit;
      query.last =  beforeCursor ? defaultRowLimit : null;
      query.after = afterCursor ? afterCursor : null;
      query.before = beforeCursor ? beforeCursor : null;
    }

    if(id){
      query.where.id = id;
    }

    return query
};

const getFilterHostQuery = (query: any, filtersSelected: Array<any>) => {
    const {beacon: beacons, group: groups, service: services, platform: platforms, host:hosts} = getFilterNameByTypes(filtersSelected);

    const fq = query;

    if(beacons.length > 0){
        fq.where.hasBeaconsWith = {"nameIn": beacons};
    };

    if(hosts.length > 0){
        fq.where.nameIn = hosts;
    };

    if(platforms.length > 0){
        fq.where.platformIn = platforms;
    };

    if(services.length > 0){
        fq.where.and.push(
            {
                "hasTagsWith": {
                  "kind":"service",
                  "nameIn": services
                }
            }
        );
    };

    if(groups.length > 0){
        fq.where.and.push(
            {
                "hasTagsWith": {
                  "kind":"group",
                  "nameIn": groups
                }
            }
        );
    }

    return fq;
};
