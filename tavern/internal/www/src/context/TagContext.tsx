import React, { createContext, useCallback, useContext, useEffect, useState } from "react";
import { ApolloError, useQuery } from "@apollo/client";
import { GET_TAG_FILTERS } from "../utils/queries";
import { BeaconEdge, BeaconNode, HostEdge, HostNode, TagContextQueryResponse, TagEdge, TagNode } from "../utils/interfacesQuery";
import { FilterBarOption, TagContextProps } from "../utils/interfacesUI";
import { SupportedPlatforms } from "../utils/enums";
import { OnlineOfflineOptions } from "../utils/utils";

type TagContextType = {
    data: TagContextProps;
    isLoading: boolean;
    error: ApolloError | undefined;
    lastFetchedTimestamp: Date;
};

export const TagContext = createContext<TagContextType | undefined>(undefined);

export const TagContextProvider = ({ children }: { children: React.ReactNode }) => {
    const [tags, setTags] = useState<TagContextProps>({
        beacons: [],
        groupTags: [],
        serviceTags: [],
        hosts: [],
        principals: [],
        primaryIPs: [],
        platforms: [],
        onlineOfflineStatus: [],
    });
    const [lastFetchedTimestamp, setLastFetchedTimestamp] = useState<Date>(new Date());

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
        const supportedPlatformsList = Object.values(SupportedPlatforms);
        const beacons: Array<FilterBarOption & BeaconNode> = [];
        const principalsSet = new Set<string>();
        const principals: FilterBarOption[] = [];
        data?.beacons?.edges?.forEach((beacon: BeaconEdge) => {
            const node = beacon.node;
            beacons.push({
                ...node,
                value: node.id,
                label: node.name,
                kind: "beacon"
            });
            if (node.principal && !principalsSet.has(node.principal)) {
                principalsSet.add(node.principal);
                principals.push({
                    id: node.principal,
                    name: node.principal,
                    value: node.principal,
                    label: node.principal,
                    kind: "principal"
                });
            }
        });

        const hosts: Array<FilterBarOption & HostNode> = [];
        const primaryIPsSet = new Set<string>();
        const primaryIPs: FilterBarOption[] = [];
        data?.hosts?.edges?.forEach((edge: HostEdge) => {
            const node = edge.node;
            hosts.push({
                ...node,
                value: node.id,
                label: node.name,
                kind: "host"
            });
            if (node.primaryIP && !primaryIPsSet.has(node.primaryIP)) {
                primaryIPsSet.add(node.primaryIP);
                primaryIPs.push({
                    id: node.primaryIP,
                    name: node.primaryIP,
                    value: node.primaryIP,
                    label: node.primaryIP,
                    kind: "primaryIP"
                });
            }
        });

        const groupTags: Array<FilterBarOption & TagNode> = [];
        data?.groupTags?.edges?.forEach((tag: TagEdge) => {
            const node = tag.node;
            groupTags.push({
                ...node,
                value: node.id,
                label: node.name,
                kind: node.kind
            });
        });

        const serviceTags: Array<FilterBarOption & TagNode> = [];
        data?.serviceTags?.edges?.forEach((tag: TagEdge) => {
            const node = tag.node;
            serviceTags.push({
                ...node,
                value: node.id,
                label: node.name,
                kind: node.kind
            });
        });

        // Build platform options
        const platforms: FilterBarOption[] = supportedPlatformsList.map((platform: string) => ({
            id: platform,
            name: platform,
            value: platform,
            label: platform,
            kind: "platform"
        }));

        // Set tags state with formatted options
        const tags: TagContextProps = {
            beacons,
            groupTags,
            serviceTags,
            hosts,
            principals,
            primaryIPs,
            platforms,
            onlineOfflineStatus: OnlineOfflineOptions
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
            getTags(data);
            setLastFetchedTimestamp(new Date());
        }
    }, [data, getTags])


    return (
        <TagContext.Provider value={{ data: tags, isLoading, error, lastFetchedTimestamp }}>
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
