import { gql } from "@apollo/client";

export const GET_PORTAL_IDS_QUERY = gql`
    query GetPortalIds(
        $where: PortalWhereInput,
        $first: Int,
        $last: Int,
        $after: Cursor,
        $before: Cursor,
        $orderBy: [PortalOrder!]
    ) {
        portals(where: $where, first: $first, last: $last, after: $after, before: $before, orderBy: $orderBy) {
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

export const GET_PORTAL_DETAIL_QUERY = gql`
    query GetPortalDetail($id: ID!) {
        portals(where: { id: $id }, first: 1) {
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
                }
            }
        }
    }
`;
