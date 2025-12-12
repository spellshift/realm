import React, { createContext, useCallback, useEffect, useState } from "react";
import { ApolloError, useQuery } from "@apollo/client";
import { TagContextType } from "../utils/consts";
import { GET_TAG_FILTERS } from "../utils/queries";

const defaultValue = { data: undefined, isLoading: false, error: undefined } as { data: undefined | TagContextType, isLoading: boolean, error: ApolloError | undefined };

export const TagContext = createContext(defaultValue);

export const TagContextProvider = ({ children }: { children: React.ReactNode }) => {
    const [tags, setTags] = useState({} as TagContextType);

    const PARAMS = {
        variables: {
            groupTag: { kind: "group" },
            serviceTag: { kind: "service" },
        }
    }
    const { loading: isLoading, error, data, startPolling, stopPolling } = useQuery(GET_TAG_FILTERS, PARAMS);

    const getTags = useCallback((data: any) => {
        if (!data) {
            return;
        }
        const tags: TagContextType = {
            beacons: data?.beacons,
            groupTags: data?.groupTags,
            serviceTags: data?.serviceTags,
            hosts: data?.hosts?.edges.map((edge: { node: { id: string, name: string } }) => edge.node)
        };
        setTags(tags);
    }, []) as any;

    useEffect(() => {
        startPolling(60000);
        return () => {
            stopPolling();
        }
    }, [startPolling, stopPolling])

    useEffect(() => {
        if (data) {
            getTags(data)
        }
    }, [data, getTags])


    return (
        <TagContext.Provider value={{ data: tags, isLoading, error }}>
            {children}
        </TagContext.Provider>
    );
};
