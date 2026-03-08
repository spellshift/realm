import { gql } from "@apollo/client";

export const GET_ASSET_IDS_QUERY = gql`
    query GetAssetIds(
        $where: AssetWhereInput,
        $first: Int,
        $last: Int,
        $after: Cursor,
        $before: Cursor,
        $orderBy: [AssetOrder!]
    ) {
        assets(where: $where, first: $first, last: $last, after: $after, before: $before, orderBy: $orderBy) {
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
                }
            }
        }
    }
`;

export const GET_ASSET_DETAIL_QUERY = gql`
    query GetAssetDetail($id: ID!) {
        assets(where: { id: $id }, first: 1) {
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
