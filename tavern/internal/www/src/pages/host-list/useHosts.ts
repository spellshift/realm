import { useCallback, useEffect, useState } from "react";
import { HostType } from "../../utils/consts";
import { useQuery } from "@apollo/client";
import { GET_HOST_QUERY } from "../../utils/queries";
import { PrincipalAdminTypes, TableRowLimit } from "../../utils/enums";
import { convertArrayOfObjectsToObject, getFilterNameByTypes, getOfflineOnlineStatus } from "../../utils/utils";
import { formatDistance } from "date-fns";
import { Filters, useFilters } from "../../context/FilterContext";
import { constructBeaconFilterQuery, constructHostFieldQuery } from "../../utils/constructQueryUtils";

export const useHosts = (pagination: boolean, id?: string) => {
    const currentDate = new Date();
    const [page, setPage] = useState<number>(1);
    const {filters} = useFilters();

    const constructDefaultQuery = useCallback((afterCursor?: string | undefined, beforeCursor?: string | undefined) => {
        return getDefaultHostQuery(pagination, afterCursor, beforeCursor, id, filters);
    },[pagination, id, filters]);

    const { loading, data, error, refetch } = useQuery(
        GET_HOST_QUERY, {variables: constructDefaultQuery(),  notifyOnNetworkStatusChange: true}
    );

    const updateHosts = useCallback((afterCursor?: string | undefined, beforeCursor?: string | undefined) => {
        const query = constructDefaultQuery(afterCursor, beforeCursor);
        refetch(query)
      },[constructDefaultQuery, refetch]);


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

const getDefaultHostQuery = (pagination: boolean, afterCursor?: string | undefined, beforeCursor?: string | undefined, id?: string | undefined, filters?: Filters) => {
    const defaultRowLimit = TableRowLimit.HostRowLimit;
    const filterInfo = (filters && filters.filtersEnabled) && constructBeaconFilterQuery(filters.beaconFields);

    const query = {
      "where": {
        ...id && {"id": id},
        ...filterInfo && filterInfo.hasBeaconWith.hasHostWith,
        ...(filterInfo && filterInfo.hasBeaconWith.nameIn) && {"hasBeaconsWith": {"nameIn": filterInfo.hasBeaconWith.nameIn}}
      },
    } as any;

    if(pagination){
      query.first = beforeCursor ? null : defaultRowLimit;
      query.last =  beforeCursor ? defaultRowLimit : null;
      query.after = afterCursor ? afterCursor : null;
      query.before = beforeCursor ? beforeCursor : null;
    }

    return query
};
