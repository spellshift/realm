import { gql } from "@apollo/client";

export const GET_BEACONS_FOR_HOST_QUERY = gql`
    query GetBeaconsForHost($hostId: ID!) {
        beacons(
            where: { hasHostWith: { id: $hostId } },
            first: 100,
            orderBy: [{ direction: DESC, field: LAST_SEEN_AT }]
        ) {
            edges {
                node {
                    id
                    principal
                    interval
                    lastSeenAt
                    nextSeenAt
                }
            }
        }
    }
`;

export const CREATE_SHELL_MUTATION = gql`
    mutation CreateShell($beaconId: ID!) {
        createShell(input: { beaconID: $beaconId }) {
            id
        }
    }
`;
