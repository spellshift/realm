import { gql } from "@apollo/client";

export const GET_BEACON_IDS_QUERY = gql`
    query GetBeaconIds(
        $where: BeaconWhereInput,
        $first: Int,
        $last: Int,
        $after: Cursor,
        $before: Cursor,
        $orderBy: [BeaconOrder!]
    ) {
        beacons(where: $where, first: $first, last: $last, after: $after, before: $before, orderBy: $orderBy) {
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

export const GET_BEACON_DETAIL_QUERY = gql`
    query GetBeaconDetail($id: ID!) {
        beacons(where: { id: $id }, first: 1) {
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
                    principal
                    interval
                    lastSeenAt
                    transport
                    host {
                        id
                    }
                }
            }
        }
    }
`;
