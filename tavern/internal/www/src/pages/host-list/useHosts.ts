import { useCallback, useEffect, useState } from "react";
import { ApolloError, useQuery } from "@apollo/client";
import { GET_HOST_QUERY } from "../../utils/queries";
import { PageNavItem, TableRowLimit } from "../../utils/enums";
import { Filters, useFilters } from "../../context/FilterContext";
import { constructBeaconFilterQuery, } from "../../utils/constructQueryUtils";
import { Cursor, HostQueryTopLevel, OrderByField } from "../../utils/interfacesQuery";
import { useSorts } from "../../context/SortContext";

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
    const {sorts} = useSorts();
    const hostSort = sorts[PageNavItem.hosts];

    const constructDefaultQuery = useCallback((afterCursor?: Cursor, beforeCursor?: Cursor) => {
        return getDefaultHostQuery(pagination, afterCursor, beforeCursor, id, filters, hostSort);
    },[pagination, id, filters, hostSort]);

    const { loading, data, error, refetch } = useQuery(
        GET_HOST_QUERY, {variables: constructDefaultQuery(),  notifyOnNetworkStatusChange: true}
    );

    const updateHosts = useCallback((afterCursor?: Cursor, beforeCursor?: Cursor) => {
        const query = constructDefaultQuery(afterCursor, beforeCursor);
        return refetch(query);
      },[constructDefaultQuery, refetch]);


    useEffect(()=>{
        const abortController = new AbortController();
        updateHosts();

        return () => {
            abortController.abort();
        };
    },[updateHosts]);

    useEffect(()=>{
      setPage(1);
    },[filters, hostSort])

    return {
        data,
        loading,
        error,
        page,
        setPage,
        updateHosts,
    }
};

const getDefaultHostQuery = (pagination: boolean, afterCursor?: Cursor, beforeCursor?: Cursor, id?: string | undefined, filters?: Filters, sort?: OrderByField) => {
    const defaultRowLimit = TableRowLimit.HostRowLimit;
    const filterInfo = (filters && filters.filtersEnabled) && constructBeaconFilterQuery(filters.beaconFields);

    const query = {
      "where": {
        ...id && {"id": id},
        ...filterInfo && filterInfo.hasBeaconWith.hasHostWith,
        ...(filterInfo && filterInfo.hasBeaconWith.nameIn) && {"hasBeaconsWith": {"nameIn": filterInfo.hasBeaconWith.nameIn}},
        ...(filterInfo && filterInfo.hasBeaconWith.principalIn) && {"hasBeaconsWith": {"principalIn": filterInfo.hasBeaconWith.principalIn}}
      },
      ...(sort && {orderBy: [sort]})
    } as any;

    if(pagination){
      query.first = beforeCursor ? null : defaultRowLimit;
      query.last =  beforeCursor ? defaultRowLimit : null;
      query.after = afterCursor ? afterCursor : null;
      query.before = beforeCursor ? beforeCursor : null;
    }

    return query
};
