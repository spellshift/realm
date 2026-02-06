import { gql } from "@apollo/client";

export const GET_HOST_IDS_QUERY = gql`
    query GetHostIds(
        $where: HostWhereInput,
        $first: Int,
        $last: Int,
        $after: Cursor,
        $before: Cursor,
        $orderBy: [HostOrder!]
    ) {
        hosts(where: $where, first: $first, last: $last, after: $after, before: $before, orderBy: $orderBy) {
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

export const GET_HOST_DETAIL_QUERY = gql`
    query GetHostDetail($id: ID!) {
        hosts(where: { id: $id }, first: 1) {
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
                    primaryIP
                    externalIP
                    platform
                    lastSeenAt
                    tags {
                        edges {
                            node {
                                name
                                id
                                kind
                            }
                        }
                    }
                    beacons {
                        edges {
                            node {
                                id
                                name
                                principal
                                interval
                                lastSeenAt
                                transport
                            }
                        }
                    }
                    credentials {
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
