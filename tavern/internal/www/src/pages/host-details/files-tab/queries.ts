import { gql } from "@apollo/client";

export const GET_FILE_IDS_QUERY = gql`
    query GetFileIds(
        $hostId: ID!,
        $first: Int,
        $after: Cursor,
        $orderBy: [HostFileOrder!],
        $where: HostFileWhereInput
    ) {
        hosts(where: { id: $hostId }) {
            edges {
                node {
                    files(first: $first, after: $after, orderBy: $orderBy, where: $where) {
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
            }
        }
    }
`;

export const GET_FILE_DETAIL_QUERY = gql`
    query GetFileDetail($hostId: ID!, $fileId: ID!) {
        hosts(where: { id: $hostId }) {
            edges {
                node {
                    files(where: { id: $fileId }, first: 1) {
                        edges {
                            node {
                                id
                                createdAt
                                lastModifiedAt
                                path
                                owner
                                group
                                permissions
                                size
                                hash
                            }
                        }
                    }
                }
            }
        }
    }
`;
