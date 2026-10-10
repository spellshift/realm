import { gql } from "@apollo/client";

export const GET_BEACONS_BY_IDS_QUERY = gql`
    query GetBeaconsByIds($where: BeaconWhereInput) {
        beacons(where: $where) {
            edges {
                node {
                    id
                    lastSeenAt
                    interval
                }
            }
        }
    }
`;
