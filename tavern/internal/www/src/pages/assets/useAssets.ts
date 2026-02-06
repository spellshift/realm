import { gql, useQuery, useMutation } from "@apollo/client";
import { AssetQueryTopLevel, Cursor } from "../../utils/interfacesQuery";
import { useCallback, useState } from "react";

export const GET_ASSETS = gql`
  query GetAssets($first: Int, $last: Int, $after: Cursor, $before: Cursor, $where: AssetWhereInput) {
    assets(first: $first, last: $last, after: $after, before: $before, where: $where) {
      totalCount
      pageInfo {
        hasNextPage
        hasPreviousPage
        startCursor
        endCursor
      }
      edges {
        node {
          id
          name
          size
          hash
          createdAt
          lastModifiedAt
          links {
            totalCount
          }
          tomes {
            totalCount
          }
        }
      }
    }
  }
`;

export const CREATE_LINK = gql`
  mutation CreateLink($input: CreateLinkInput!) {
    createLink(input: $input) {
      id
      path
      expiresAt
      downloadsRemaining
    }
  }
`;

export const useAssets = (rowLimit = 50, where?: any) => {
  const [page, setPage] = useState(1);
  const { data, loading, error, refetch, fetchMore } = useQuery<AssetQueryTopLevel>(GET_ASSETS, {
    variables: { first: rowLimit, where },
    fetchPolicy: "network-only",
  });

  const updateAssets = useCallback((afterCursor?: Cursor, beforeCursor?: Cursor) => {
      const variables: any = { where };
      if (afterCursor) {
          variables.first = rowLimit;
          variables.after = afterCursor;
      } else if (beforeCursor) {
          variables.last = rowLimit;
          variables.before = beforeCursor;
      } else {
          variables.first = rowLimit;
      }
      return refetch(variables);
  }, [rowLimit, where, refetch]);


  return {
    assets: data?.assets.edges.map((edge) => edge.node) || [],
    pageInfo: data?.assets.pageInfo,
    totalCount: data?.assets.totalCount,
    loading,
    error,
    refetch,
    updateAssets,
    page,
    setPage,
  };
};

export const useCreateLink = () => {
  const [createLink, { data, loading, error }] = useMutation(CREATE_LINK);
  return { createLink, data, loading, error };
};
