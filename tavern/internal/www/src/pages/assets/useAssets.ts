import { gql, useQuery, useMutation } from "@apollo/client";
import { AssetQueryTopLevel, Cursor } from "../../utils/interfacesQuery";
import { useCallback, useMemo, useState } from "react";
import { PageNavItem } from "../../utils/enums";
import { useSorts } from "../../context/SortContext";
import { useFilters } from "../../context/FilterContext";

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
          creator {
            name
          }
          links(first: 100) {
            totalCount
            edges {
              node {
                id
                path
                expiresAt
                downloadsRemaining
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

export const DISABLE_LINK = gql`
  mutation DisableLink($linkID: ID!) {
    disableLink(linkID: $linkID) {
      id
      path
      expiresAt
      downloadsRemaining
    }
  }
`;

export const useAssets = (rowLimit = 50, where?: any) => {
  const [page, setPage] = useState(1);
  const { sorts } = useSorts();
  const { filters } = useFilters();
  const assetSort = sorts[PageNavItem.assets];

  const queryVariables = useMemo(() => {
    const vars: any = {
      first: rowLimit,
      where: where || {},
      orderBy: assetSort ? [assetSort] : undefined
    }

    if (filters.assetCreator) {
      vars.where.hasCreatorWith = { nameContains: filters.assetCreator };
    }

    return vars;
  }, [rowLimit, where, assetSort, filters.assetCreator]);

  const { data, loading, error, refetch } = useQuery<AssetQueryTopLevel>(GET_ASSETS, {
    variables: queryVariables,
    fetchPolicy: "network-only",
  });

  const updateAssets = useCallback((afterCursor?: Cursor, beforeCursor?: Cursor) => {
      const variables: any = { where: queryVariables.where, orderBy: assetSort ? [assetSort] : undefined };
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
  }, [rowLimit, queryVariables.where, refetch, assetSort]);


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

export const useDisableLink = () => {
  const [disableLink, { data, loading, error }] = useMutation(DISABLE_LINK);
  return { disableLink, data, loading, error };
};
