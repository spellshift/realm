import React, { createContext } from "react";
import { useQuery } from "@apollo/client";
import { HostType } from "../utils/consts";
import { GET_HOST_QUERY } from "../utils/queries";
import { useParams } from "react-router-dom";

export type HostContextQueryType = {
    data: undefined | HostType;
    loading: boolean;
    error: any;
}

const defaultValue = { data: undefined, loading: false, error: undefined } as HostContextQueryType;

export const HostContext = createContext(defaultValue);

export const HostContextProvider = ({ children }: { children: React.ReactNode }) => {
    const { hostId } = useParams();

    const { loading, data, error } = useQuery(GET_HOST_QUERY, {
        variables: {
            "where": {
                "id": hostId
            }
        }
    });

    const host = data?.hosts?.edges && data?.hosts?.edges.length > 0 ? data.hosts?.edges[0]?.node : undefined as HostType | undefined;

    return (
        <HostContext.Provider value={{ data: host, loading, error }}>
            {children}
        </HostContext.Provider>
    );
};
