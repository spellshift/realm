import React, { createContext, useCallback, useEffect, useState } from "react";
import { ApolloError, useQuery } from "@apollo/client";
import { GET_TAG_FILTERS } from "../utils/queries";
import { BeaconEdge, TagContextProps, TagContextQueryResponse, TagEdge } from "../utils/interfacesQuery";

const defaultValue = { data: undefined, isLoading: false, error: undefined } as { data: undefined | TagContextProps, isLoading: boolean, error: ApolloError | undefined };

export const TagContext = createContext(defaultValue);

export const TagContextProvider = ({ children }: { children: React.ReactNode }) => {
    const [tags, setTags] = useState({} as TagContextProps);

    const PARAMS = {
        variables: {
            groupTag: { kind: "group" },
            serviceTag: { kind: "service" },
        }
    }
    const { loading: isLoading, error, data, startPolling, stopPolling } = useQuery(GET_TAG_FILTERS, PARAMS);

    const getTags = useCallback((data: TagContextQueryResponse) => {
        if (!data) {
            return;
        }
        const tags = {
            beacons: data?.beacons?.edges.map((beacon: BeaconEdge) => beacon.node),
            groupTags: data?.groupTags.edges.map((tag: TagEdge) => tag.node),
            serviceTags: data?.serviceTags.edges.map((tag: TagEdge) => tag.node),
            hosts: data?.hosts?.edges.map((edge: { node: { id: string, name: string } }) => edge.node)
        };
        setTags(tags);
    }, []);

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
