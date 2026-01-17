import { useCallback, useEffect, useState } from "react";
import { ApolloError, useQuery } from "@apollo/client";
import { GET_HOST_QUERY } from "../../utils/queries";
import { PageNavItem, TableRowLimit } from "../../utils/enums";
import { Filters, useFilters } from "../../context/FilterContext";
import { constructBeaconFilterQuery, } from "../../utils/constructQueryUtils";
import { Cursor, HostQueryTopLevel, OrderByField } from "../../utils/interfacesQuery";
import { useSorts } from "../../context/SortContext";
import { useTags } from "../../context/TagContext";

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
    const {lastFetchedTimestamp} = useTags();
    const hostSort = sorts[PageNavItem.hosts];

    const constructDefaultQuery = useCallback((afterCursor?: Cursor, beforeCursor?: Cursor) => {
        return getDefaultHostQuery(pagination, afterCursor, beforeCursor, id, filters, hostSort, lastFetchedTimestamp);
    },[pagination, id, filters, hostSort, lastFetchedTimestamp]);

    const { loading, data, error, refetch } = useQuery(
        GET_HOST_QUERY,
        {
            variables: constructDefaultQuery(),
            notifyOnNetworkStatusChange: true,
        }
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

const getDefaultHostQuery = (pagination: boolean, afterCursor?: Cursor, beforeCursor?: Cursor, id?: string | undefined, filters?: Filters, sort?: OrderByField, currentTimestamp?: Date) => {
    const defaultRowLimit = TableRowLimit.HostRowLimit;
    const filterInfo = (filters && filters.filtersEnabled) && constructBeaconFilterQuery(filters.beaconFields, currentTimestamp);

    // Separate host fields from beacon fields
    const hostFields = (filterInfo && filterInfo.hasBeaconWith?.hasHostWith) || {};
    const beaconFields = (filterInfo && filterInfo.hasBeaconWith) ? { ...filterInfo.hasBeaconWith } : {};
    delete beaconFields.hasHostWith; // Remove host fields from beacon fields

    const query = {
      "where": {
        ...id && {"id": id},
        ...hostFields,
        ...(Object.keys(beaconFields).length > 0) && {"hasBeaconsWith": beaconFields}
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
