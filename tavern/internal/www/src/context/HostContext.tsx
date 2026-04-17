import React, { createContext, useContext } from "react";
import { ApolloError, gql, useQuery } from "@apollo/client";
import { useParams } from "react-router-dom";
import { HostNode } from "../utils/interfacesQuery";

const HOST_CONTEXT_POLL_INTERVAL = 5000;

export const GET_HOST_CONTEXT_QUERY = gql`
    query GetHostContext($hostId: ID!) {
        hosts(where: { id: $hostId }) {
            edges {
                node {
                    id
                    name
                    primaryIP
                    externalIP
                    platform
                    tags {
                        edges {
                            node {
                                id
                                name
                                kind
                            }
                        }
                    }
                    beacons {
                        totalCount
                    }
                    processes {
                        totalCount
                    }
                    files {
                        totalCount
                    }
                    credentials {
                        totalCount
                    }
                }
            }
        }
        tasks(where: { hasBeaconWith: { hasHostWith: { id: $hostId } } }) {
            totalCount
        }
        totalShells: shells(where: { hasBeaconWith: { hasHostWith: { id: $hostId } } }) {
            totalCount
        }
        activeShells: shells(where: { hasBeaconWith: { hasHostWith: { id: $hostId } }, closedAtIsNil: true }) {
            totalCount
        }
        totalPortals: portals(where: { hasBeaconWith: { hasHostWith: { id: $hostId } } }) {
            totalCount
        }
        activePortals: portals(where: { hasBeaconWith: { hasHostWith: { id: $hostId } }, closedAtIsNil: true }) {
            totalCount
        }
    }
`;

export type HostContextQueryType = {
    data: HostNode | undefined;
    loading: boolean;
    error: ApolloError | undefined;
    taskCount: number | undefined;
    totalShellCount: number | undefined;
    activeShellCount: number | undefined;
    totalPortalCount: number | undefined;
    activePortalCount: number | undefined;
}

export const HostContext = createContext<HostContextQueryType | undefined>(undefined);

export const HostContextProvider = ({ children }: { children: React.ReactNode }) => {
    const { hostId } = useParams();

    const { loading, data, error } = useQuery(GET_HOST_CONTEXT_QUERY, {
        variables: { hostId },
        skip: !hostId,
        pollInterval: HOST_CONTEXT_POLL_INTERVAL,
    });

    const host = data?.hosts?.edges?.[0]?.node;
    const taskCount = data?.tasks?.totalCount;
    const totalShellCount = data?.totalShells?.totalCount;
    const activeShellCount = data?.activeShells?.totalCount;
    const totalPortalCount = data?.totalPortals?.totalCount;
    const activePortalCount = data?.activePortals?.totalCount;

    return (
        <HostContext.Provider value={{ data: host, loading, error, taskCount, totalShellCount, activeShellCount, totalPortalCount, activePortalCount }}>
            {children}
        </HostContext.Provider>
    );
};

export const useHost = () => {
    const context = useContext(HostContext);
    if (context === undefined) {
        throw new Error('useHost must be used within a HostContextProvider');
    }
    return context;
};
