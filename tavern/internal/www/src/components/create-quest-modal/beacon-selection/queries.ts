import { gql } from "@apollo/client";

export const GET_BEACON_IDS_QUERY = gql`
    query GetBeaconIds(
        $where: BeaconWhereInput
        $first: Int
        $last: Int
        $after: Cursor
        $before: Cursor
        $orderBy: [BeaconOrder!]
    ) {
        beacons(
            where: $where
            first: $first
            last: $last
            after: $after
            before: $before
            orderBy: $orderBy
        ) {
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
                    principal
                    transport
                    host {
                        id
                    }
                }
            }
        }
    }
`;

export const GET_BEACON_DETAIL_QUERY = gql`
    query GetBeaconDetail($id: ID!) {
        beacons(where: { id: $id }, first: 1) {
            edges {
                node {
                    id
                    name
                    principal
                    lastSeenAt
                    interval
                    transport
                    host {
                        id
                        name
                        primaryIP
                        externalIP
                        platform
                        tags {
                            edges {
                                node {
                                    id
                                    kind
                                    name
                                }
                            }
                        }
                    }
                }
            }
        }
    }
`;
