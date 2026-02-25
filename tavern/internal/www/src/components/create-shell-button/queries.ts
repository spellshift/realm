import { gql } from "@apollo/client";

export const GET_BEACONS_FOR_HOST_QUERY = gql`
    query GetBeaconsForHost($hostId: ID!) {
        beacons(where: { hasHostWith: { id: $hostId } }) {
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
    mutation CreateShell($input: CreateShellInput!) {
        createShell(input: $input) {
            id
        }
    }
`;
