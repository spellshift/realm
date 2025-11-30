import React, { createContext, useContext } from "react";
import { ApolloError, useQuery } from "@apollo/client";
import { GET_HOST_QUERY } from "../utils/queries";
import { useParams } from "react-router-dom";
import { HostNode } from "../utils/interfacesQuery";

export type HostContextQueryType = {
    data: HostNode | undefined;
    loading: boolean;
    error: ApolloError | undefined;
}

export const HostContext = createContext<HostContextQueryType | undefined>(undefined);

export const HostContextProvider = ({ children }: { children: React.ReactNode }) => {
    const { hostId } = useParams();

    const { loading, data, error } = useQuery(GET_HOST_QUERY, {
        variables: {
            "where": {
                "id": hostId
            }
        }
    });

    const host = data?.hosts?.edges?.[0]?.node;

    return (
        <HostContext.Provider value={{ data: host, loading, error }}>
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
