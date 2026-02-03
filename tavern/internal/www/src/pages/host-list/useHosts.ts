import { useCallback, useEffect, useMemo, useState } from "react";
import { ApolloError, NetworkStatus, useQuery } from "@apollo/client";
import { GET_HOST_QUERY } from "../../utils/queries";
import { PageNavItem, TableRowLimit } from "../../utils/enums";
import { Filters, useFilters } from "../../context/FilterContext";
import { constructBeaconFilterQuery, } from "../../utils/constructQueryUtils";
import { Cursor, HostQueryTopLevel, OrderByField } from "../../utils/interfacesQuery";
import { useSorts } from "../../context/SortContext";
import { useTags } from "../../context/TagContext";

interface HostsHook {
  data: HostQueryTopLevel | undefined
  loading: boolean,
  error: ApolloError | undefined,
  page: number,
  setPage: React.Dispatch<React.SetStateAction<number>>,
  updateHosts: (afterCursor?: Cursor, beforeCursor?: Cursor) => void,
}

export const useHosts = (pagination: boolean, id?: string): HostsHook => {
  const [page, setPage] = useState<number>(1);
  const { filters } = useFilters();
  const { sorts } = useSorts();
  const { lastFetchedTimestamp } = useTags();
  const hostSort = sorts[PageNavItem.hosts];

  const queryVariables = useMemo(
    () => getDefaultHostQuery(filters, pagination, undefined, undefined, id, hostSort, lastFetchedTimestamp),
    [filters, pagination, id, hostSort, lastFetchedTimestamp]
  );

  const { data, previousData, error, refetch, networkStatus } = useQuery(
    GET_HOST_QUERY,
    {
      variables: queryVariables,
      notifyOnNetworkStatusChange: true,
      fetchPolicy: 'cache-and-network',
    }
  );

  const updateHosts = useCallback((afterCursor?: Cursor, beforeCursor?: Cursor) => {
    return refetch(
      getDefaultHostQuery(filters, pagination, afterCursor, beforeCursor, id, hostSort, lastFetchedTimestamp)
    );
  }, [filters, pagination, id, hostSort, lastFetchedTimestamp, refetch]);

  useEffect(() => {
    setPage(prev => prev !== 1 ? 1 : prev);
  }, [filters, hostSort]);

  const currentData = data ?? previousData;


  return {
    data: currentData,
    loading: networkStatus === NetworkStatus.loading && !currentData,
    error,
    page,
    setPage,
    updateHosts,
  }
};

const getDefaultHostQuery = (filters: Filters, pagination: boolean, afterCursor?: Cursor, beforeCursor?: Cursor, id?: string | undefined, sort?: OrderByField, currentTimestamp?: Date) => {
  const defaultRowLimit = TableRowLimit.HostRowLimit;
  const filterInfo = constructBeaconFilterQuery(filters.beaconFields, currentTimestamp);

  // Separate host fields from beacon fields
  const hostFields = (filterInfo && filterInfo.hasBeaconWith?.hasHostWith) || {};
  const beaconFields = (filterInfo && filterInfo.hasBeaconWith) ? { ...filterInfo.hasBeaconWith } : {};
  delete beaconFields.hasHostWith; // Remove host fields from beacon fields

  const query = {
    "where": {
      ...id && { "id": id },
      ...hostFields,
      ...(Object.keys(beaconFields).length > 0) && { "hasBeaconsWith": beaconFields }
    },
    ...(sort && { orderBy: [sort] })
  } as any;

  if (pagination) {
    query.first = beforeCursor ? null : defaultRowLimit;
    query.last = beforeCursor ? defaultRowLimit : null;
    query.after = afterCursor ? afterCursor : null;
    query.before = beforeCursor ? beforeCursor : null;
  }

  return query
};
