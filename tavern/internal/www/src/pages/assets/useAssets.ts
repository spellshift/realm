import { gql, useQuery, useMutation } from "@apollo/client";
import { AssetQueryTopLevel, Cursor } from "../../utils/interfacesQuery";
import { useCallback, useMemo, useState } from "react";
import { PageNavItem } from "../../utils/enums";
import { useSorts } from "../../context/SortContext";

export const GET_ASSETS = gql`
  query GetAssets($first: Int, $last: Int, $after: Cursor, $before: Cursor, $where: AssetWhereInput, $orderBy: [AssetOrder!]) {
    assets(first: $first, last: $last, after: $after, before: $before, where: $where, orderBy: $orderBy) {
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
          links(first: 100) {
            totalCount
            edges {
              node {
                id
                path
                expiresAt
                downloadLimit
                downloads
                creator {
                  id
                  name
                  photoURL
                }
              }
            }
          }
          tomes(first: 100) {
            totalCount
            edges {
              node {
                id
                name
              }
            }
          }
          creator {
            id
            name
            photoURL
          }
        }
      }
    }
  }
`;

export const DISABLE_LINK = gql`
  mutation DisableLink($linkID: ID!) {
    disableLink(linkID: $linkID) {
      id
      expiresAt
    }
  }
`;

export const CREATE_LINK = gql`
  mutation CreateLink($input: CreateLinkInput!) {
    createLink(input: $input) {
      id
      path
      expiresAt
      downloads
      downloadLimit
    }
  }
`;

export const useAssets = (rowLimit = 50, where?: any) => {
  const [page, setPage] = useState(1);
  const { sorts } = useSorts();
  const assetSort = sorts[PageNavItem.assets];

  const queryVariables = useMemo(() => {
    return {
      first: rowLimit,
      where,
      orderBy: assetSort ? [assetSort] : undefined
    }
  }, [rowLimit, where, assetSort]);

  const { data, loading, error, refetch } = useQuery<AssetQueryTopLevel>(GET_ASSETS, {
    variables: queryVariables,
    fetchPolicy: "network-only",
  });

  const updateAssets = useCallback((afterCursor?: Cursor, beforeCursor?: Cursor) => {
      const variables: any = {
        where,
        orderBy: assetSort ? [assetSort] : undefined,
      "first": beforeCursor ? null : rowLimit,
      "last": beforeCursor ? rowLimit : null,
      "after": afterCursor ? afterCursor : null,
      "before": beforeCursor ? beforeCursor : null,
      };
      return refetch(variables);
  }, [rowLimit, where, refetch, assetSort]);


  return {
    assets: data?.assets.edges || [],
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
