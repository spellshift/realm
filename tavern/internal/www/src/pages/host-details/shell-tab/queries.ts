import { gql } from "@apollo/client";

export const GET_SHELL_IDS_QUERY = gql`
    query GetShellIds(
        $where: ShellWhereInput,
        $first: Int,
        $last: Int,
        $after: Cursor,
        $before: Cursor,
        $orderBy: [ShellOrder!]
    ) {
        shells(where: $where, first: $first, last: $last, after: $after, before: $before, orderBy: $orderBy) {
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

export const GET_SHELL_DETAIL_QUERY = gql`
    query GetShellDetail($id: ID!) {
        shells(where: { id: $id }, first: 1) {
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
                    closedAt
                    beacon {
                        id
                        name
                        lastSeenAt
                        interval
                    }
                    owner {
                        id
                        name
                        photoURL
                    }
                    activeUsers {
                        edges {
                            node {
                                id
                                name
                                photoURL
                            }
                        }
                    }
                    shellTasks {
                        totalCount
                    }
                }
            }
        }
    }
`;
