import { useQuery, NetworkStatus } from "@apollo/client";
import { useCallback, useEffect, useMemo, useState } from "react";
import { PageNavItem, TableRowLimit } from "../../utils/enums";
import { GET_TASK_QUERY } from "../../utils/queries";
import { Filters, useFilters } from "../../context/FilterContext";
import { constructHostTaskFilterQuery } from "../../utils/constructQueryUtils";
import { Cursor, OrderByField } from "../../utils/interfacesQuery";
import { useSorts } from "../../context/SortContext";
import { useTags } from "../../context/TagContext";

export const useHostTasks = (id?: string) => {
  const [page, setPage] = useState<number>(1);
  const { filters } = useFilters();
  const { sorts } = useSorts();
  const { lastFetchedTimestamp } = useTags();
  const taskSort = sorts[PageNavItem.tasks];

  const queryVariables = useMemo(
    () => getDefaultHostTaskQuery(filters, undefined, undefined, id, taskSort, lastFetchedTimestamp),
    [filters, id, taskSort, lastFetchedTimestamp]
  );

  const { data, previousData, error, refetch, networkStatus, loading } = useQuery(
    GET_TASK_QUERY,
    {
      variables: queryVariables,
      notifyOnNetworkStatusChange: true,
      fetchPolicy: 'cache-and-network',
    }
  );

  const updateTaskList = useCallback((afterCursor?: Cursor, beforeCursor?: Cursor) => {
    return refetch(
      getDefaultHostTaskQuery(filters, afterCursor, beforeCursor, id, taskSort, lastFetchedTimestamp)
    );
  }, [filters, id, taskSort, lastFetchedTimestamp, refetch]);

  useEffect(() => {
    setPage(prev => prev !== 1 ? 1 : prev);
  }, [filters, taskSort]);

  const currentData = data ?? previousData;

  return {
    data: currentData,
    loading,
    initialLoading: networkStatus === NetworkStatus.loading && !currentData,
    error,
    page,
    setPage,
    updateTaskList
  };
};

const getDefaultHostTaskQuery = (
  filters: Filters,
  afterCursor?: Cursor,
  beforeCursor?: Cursor,
  id?: string,
  sort?: OrderByField,
  currentTimestamp?: Date
) => {
  const defaultRowLimit = TableRowLimit.TaskRowLimit;
  const filterQueryFields = constructHostTaskFilterQuery(filters, currentTimestamp);

  const query = {
    "where": {
      "hasBeaconWith": {
        "hasHostWith": {
          "id": id
        }
      },
      ...filterQueryFields && filterQueryFields.hasTasksWith,
    },
    "first": beforeCursor ? null : defaultRowLimit,
    "last": beforeCursor ? defaultRowLimit : null,
    "after": afterCursor ? afterCursor : null,
    "before": beforeCursor ? beforeCursor : null,
    ...(sort && { orderBy: [sort] })
  } as any;

  return query;
};
