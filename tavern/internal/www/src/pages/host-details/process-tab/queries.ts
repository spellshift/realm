import { gql } from "@apollo/client";

export const GET_PROCESS_IDS_QUERY = gql`
    query GetProcessIds(
        $hostId: ID!,
        $first: Int,
        $after: Cursor,
        $orderBy: [HostProcessOrder!],
        $where: HostProcessWhereInput
    ) {
        hosts(where: { id: $hostId }) {
            edges {
                node {
                    processes(first: $first, after: $after, orderBy: $orderBy, where: $where) {
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

export const GET_PROCESS_DETAIL_QUERY = gql`
    query GetProcessDetail($hostId: ID!, $processId: ID!) {
        hosts(where: { id: $hostId }) {
            edges {
                node {
                    processes(where: { id: $processId }, first: 1) {
                        edges {
                            node {
                                id
                                lastModifiedAt
                                principal
                                pid
                                ppid
                                name
                                path
                                cmd
                                status
                                startTime
                                env
                                cwd
                            }
                        }
                    }
                }
            }
        }
    }
`;

export const GET_ONLINE_HOST_BEACONS_QUERY = gql`
    query GetOnlineHostBeacons($where: BeaconWhereInput) {
        beacons(where: $where) {
            edges {
                node {
                    id
                    principal
                    transport
                }
            }
        }
    }
`;
