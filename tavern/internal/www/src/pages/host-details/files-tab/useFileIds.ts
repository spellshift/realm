import { useQuery, NetworkStatus } from "@apollo/client";
import { useMemo, useState } from "react";
import { PageNavItem } from "../../../utils/enums";
import { OrderByField } from "../../../utils/interfacesQuery";
import { useSorts } from "../../../context/SortContext";
import { GET_FILE_IDS_QUERY } from "./queries";
import { FileIdsQueryResponse, GetFileIdsQueryVariables } from "./types";

export const useFileIds = (hostId: string) => {
  const { sorts } = useSorts();
  const fileSort = sorts[PageNavItem.files];
  const [searchTerm, setSearchTerm] = useState<string>("");

  const queryVariables = useMemo(
    () => getFileIdsQuery(hostId, searchTerm, fileSort),
    [hostId, searchTerm, fileSort]
  );

  const { data, networkStatus, error } = useQuery<FileIdsQueryResponse>(
        GET_FILE_IDS_QUERY,
        {
            variables: queryVariables,
            fetchPolicy: 'cache-and-network',
        }
  );

  const fileIds = useMemo(
      () => data?.hosts?.edges?.[0]?.node?.files?.edges?.map(edge => edge.node.id) || [],
      [data]
  );

  return {
    fileIds,
    totalCount: fileIds.length,
    initialLoading: networkStatus === NetworkStatus.loading && !data,
    error,
    searchTerm,
    setSearchTerm,
  };
};

const getFileIdsQuery = (
  hostId: string,
  searchTerm?: string,
  sort?: OrderByField,
): GetFileIdsQueryVariables => {

  const query: GetFileIdsQueryVariables = {
    hostId,
    ...(sort && { orderBy: [sort] }),
  };

  if (searchTerm) {
    query.where = {
      or: [
        { pathContainsFold: searchTerm.trim() },
        { hashContainsFold: searchTerm.trim() },
      ],
    };
  }

  return query;
};
