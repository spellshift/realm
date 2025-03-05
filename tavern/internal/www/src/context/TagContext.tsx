import React, { createContext, useEffect } from "react";
import { ApolloError, useQuery } from "@apollo/client";
import { TagContextType } from "../utils/consts";
import { GET_TAG_FILTERS } from "../utils/queries";

const defaultValue = { data: undefined, isLoading: false, error: undefined } as { data: undefined | TagContextType, isLoading: boolean, error: ApolloError | undefined };

export const TagContext = createContext(defaultValue);

export const TagContextProvider = ({ children }: { children: React.ReactNode }) => {

    const PARAMS = {
        variables: {
            groupTag: { kind: "group" },
            serviceTag: { kind: "service" },
        }
    }
    const { loading: isLoading, error, data, startPolling, stopPolling } = useQuery(GET_TAG_FILTERS, PARAMS);

    useEffect(() => {
        startPolling(60000);
        return () => {
            stopPolling();
        }
    }, [startPolling, stopPolling])


    return (
        <TagContext.Provider value={{ data, isLoading, error }}>
            {children}
        </TagContext.Provider>
    );
};
