import { useQuery, NetworkStatus } from "@apollo/client";
import { useMemo, useState } from "react";
import { PageNavItem } from "../../../utils/enums";
import { OrderByField } from "../../../utils/interfacesQuery";
import { useSorts } from "../../../context/SortContext";
import { GET_PROCESS_IDS_QUERY } from "./queries";
import { ProcessIdsQueryResponse, GetProcessIdsQueryVariables } from "./types";

export const useProcessIds = (hostId: string) => {
  const { sorts } = useSorts();
  const processSort = sorts[PageNavItem.processes];
  const [searchTerm, setSearchTerm] = useState<string>("");

  const queryVariables = useMemo(
    () => getProcessIdsQuery(hostId, searchTerm, processSort),
    [hostId, searchTerm, processSort]
  );

  const { data, networkStatus, error } = useQuery<ProcessIdsQueryResponse>(
        GET_PROCESS_IDS_QUERY,
        {
            variables: queryVariables,
            fetchPolicy: 'cache-and-network',
        }
  );
  
  const processIds = useMemo(
      () => data?.hosts?.edges?.[0]?.node?.processes?.edges?.map(edge => edge.node.id) || [],
      [data]
  );

  return {
    processIds,
    totalCount: processIds.length,
    initialLoading: networkStatus === NetworkStatus.loading && !data,
    error,
    searchTerm,
    setSearchTerm,
  };
};

const getProcessIdsQuery = (
  hostId: string,
  searchTerm?: string,
  sort?: OrderByField,
): GetProcessIdsQueryVariables => {

  const query: GetProcessIdsQueryVariables = {
    hostId,
    ...(sort && { orderBy: [sort] }),
  };

  if (searchTerm) {
    query.where = {
      or: [
        { nameContainsFold: searchTerm.trim() },
        { pathContainsFold: searchTerm.trim() },
      ],
    };
  }

  return query;
};
