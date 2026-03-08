import { useMemo } from "react";
import { useQuery } from "@apollo/client";
import { SupportedPlatforms, SupportedTransports } from "../../utils/enums";
import { FilterBarOption } from "../../utils/interfacesUI";
import { FilterOptionGroup, UseBeaconFilterBarProps, UseBeaconFilterBarResult, BeaconFilterOptionsResponse } from "./types";
import {
    GET_BEACON_FILTER_OPTIONS,
    ONLINE_OFFLINE_OPTIONS,
} from "./constants";

export function useBeaconFilterBar(props?: UseBeaconFilterBarProps): UseBeaconFilterBarResult {
    const hideStatusFilter = props?.hideStatusFilter ?? false;

    const { data, loading: isLoading, error } = useQuery<BeaconFilterOptionsResponse>(
        GET_BEACON_FILTER_OPTIONS,
        {
            variables: {
                groupTag: { kind: "group" },
                serviceTag: { kind: "service" },
            },
            fetchPolicy: "network-only",
        }
    );

    const options = useMemo(() => {
        if (!data) {
            return [];
        }

        const platforms = buildEnumOptions(SupportedPlatforms, "platform");
        const transports = buildEnumOptions(SupportedTransports, "transport");
        const serviceTags = buildTagOptions(data.serviceTags.edges);
        const groupTags = buildTagOptions(data.groupTags.edges);
        const principals = buildUniqueOptions(
            data.beacons.edges,
            (node) => node.principal,
            "principal"
        );
        const primaryIPs = buildUniqueOptions(
            data.hosts.edges,
            (node) => node.primaryIP,
            "primaryIP"
        );
        const hosts = buildEntityOptions(data.hosts.edges, "host");
        const beacons = buildEntityOptions(data.beacons.edges, "beacon");

        const allOptions: FilterOptionGroup[] = [
            { label: "Platform", options: platforms },
            { label: "Transport", options: transports },
            ...(hideStatusFilter ? [] : [{ label: "Status", options: ONLINE_OFFLINE_OPTIONS }]),
            { label: "Service", options: serviceTags },
            { label: "Group", options: groupTags },
            { label: "Principal", options: principals },
            { label: "PrimaryIPs", options: primaryIPs },
            { label: "Host", options: hosts },
            { label: "Beacon", options: beacons },
        ];

        return allOptions;
    }, [data, hideStatusFilter]);

    return { options, isLoading, error };
}

function buildEnumOptions(enumObj: Record<string, string>, kind: string): FilterBarOption[] {
    return Object.entries(enumObj).map(([label, value]) => ({
        id: value,
        name: value,
        value,
        label,
        kind,
    }));
}

function buildTagOptions(
    edges: { node: { id: string; name: string; kind: string } }[]
): FilterBarOption[] {
    return edges.map(({ node }) => ({
        id: node.id,
        name: node.name,
        value: node.id,
        label: node.name,
        kind: node.kind,
    }));
}

function buildEntityOptions(
    edges: { node: { id: string; name: string } }[],
    kind: string
): FilterBarOption[] {
    return edges.map(({ node }) => ({
        id: node.id,
        name: node.name,
        value: node.id,
        label: node.name,
        kind,
    }));
}

function buildUniqueOptions<TNode>(
    edges: { node: TNode }[],
    getValue: (node: TNode) => string | null | undefined,
    kind: string
): FilterBarOption[] {
    const seen = new Set<string>();
    const options: FilterBarOption[] = [];

    for (const { node } of edges) {
        const value = getValue(node);
        if (value && !seen.has(value)) {
            seen.add(value);
            options.push({
                id: value,
                name: value,
                value,
                label: value,
                kind,
            });
        }
    }

    return options;
}
