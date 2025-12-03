import React, { createContext, useCallback, useContext, useEffect, useState } from "react";
import { ApolloError, useQuery } from "@apollo/client";
import { GET_TAG_FILTERS } from "../utils/queries";
import { BeaconEdge, TagContextProps, TagContextQueryResponse, TagEdge } from "../utils/interfacesQuery";

type TagContextType = {
    data: TagContextProps;
    isLoading: boolean;
    error: ApolloError | undefined;
};

export const TagContext = createContext<TagContextType | undefined>(undefined);

export const TagContextProvider = ({ children }: { children: React.ReactNode }) => {
    const [tags, setTags] = useState<TagContextProps>({ beacons: [], groupTags: [], serviceTags: [], hosts: [] });

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
            beacons: data?.beacons?.edges?.map((beacon: BeaconEdge) => beacon.node) || [],
            groupTags: data?.groupTags?.edges?.map((tag: TagEdge) => tag.node) || [],
            serviceTags: data?.serviceTags?.edges?.map((tag: TagEdge) => tag.node) || [],
            hosts: data?.hosts?.edges?.map((edge: { node: { id: string, name: string } }) => edge.node) || []
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

export const useTags = () => {
    const context = useContext(TagContext);
    if (context === undefined) {
        throw new Error('useTags must be used within a TagContextProvider');
    }
    return context;
};
