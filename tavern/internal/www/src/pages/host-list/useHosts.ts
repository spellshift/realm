import { useCallback, useEffect, useState } from "react";
import { ApolloError, useQuery } from "@apollo/client";
import { GET_HOST_QUERY } from "../../utils/queries";
import { TableRowLimit } from "../../utils/enums";
import { Filters, useFilters } from "../../context/FilterContext";
import { constructBeaconFilterQuery, } from "../../utils/constructQueryUtils";
import { Cursor, HostQueryTopLevel } from "../../utils/interfacesQuery";

interface HostsHook {
    data: HostQueryTopLevel
    loading: boolean,
    error: ApolloError | undefined,
    page: number,
    setPage: React.Dispatch<React.SetStateAction<number>>,
    updateHosts: (afterCursor?: Cursor, beforeCursor?: Cursor) => void,
}

export const useHosts = (pagination: boolean, id?: string): HostsHook =>  {
    const [page, setPage] = useState<number>(1);
    const {filters} = useFilters();

    const constructDefaultQuery = useCallback((afterCursor?: Cursor, beforeCursor?: Cursor) => {
        return getDefaultHostQuery(pagination, afterCursor, beforeCursor, id, filters);
    },[pagination, id, filters]);

    const { loading, data, error, refetch } = useQuery(
        GET_HOST_QUERY, {variables: constructDefaultQuery(),  notifyOnNetworkStatusChange: true}
    );

    const updateHosts = useCallback((afterCursor?: Cursor, beforeCursor?: Cursor) => {
        const query = constructDefaultQuery(afterCursor, beforeCursor);
        refetch(query)
      },[constructDefaultQuery, refetch]);


    useEffect(()=>{
        updateHosts();
    },[updateHosts]);

    useEffect(()=>{
      setPage(1);
    },[filters])

    return {
        data,
        loading,
        error,
        page,
        setPage,
        updateHosts,
    }
};

const getDefaultHostQuery = (pagination: boolean, afterCursor?: Cursor, beforeCursor?: Cursor, id?: string | undefined, filters?: Filters) => {
    const defaultRowLimit = TableRowLimit.HostRowLimit;
    const filterInfo = (filters && filters.filtersEnabled) && constructBeaconFilterQuery(filters.beaconFields);

    const query = {
      "where": {
        ...id && {"id": id},
        ...filterInfo && filterInfo.hasBeaconWith.hasHostWith,
        ...(filterInfo && filterInfo.hasBeaconWith.nameIn) && {"hasBeaconsWith": {"nameIn": filterInfo.hasBeaconWith.nameIn}}
      },
      "orderBy": [{
        "field": "LAST_SEEN_AT",
        "direction": "DESC"
      }]
    } as any;

    if(pagination){
      query.first = beforeCursor ? null : defaultRowLimit;
      query.last =  beforeCursor ? defaultRowLimit : null;
      query.after = afterCursor ? afterCursor : null;
      query.before = beforeCursor ? beforeCursor : null;
    }

    return query
};
