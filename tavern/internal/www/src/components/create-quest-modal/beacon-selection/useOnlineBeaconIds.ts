import { useQuery, NetworkStatus } from "@apollo/client";
import { useMemo} from "react";
import { sub } from "date-fns";
import { FilterBarOption } from "../../../utils/interfacesUI";
import { getBeaconFilterNameByTypes } from "../../../utils/utils";
import { OrderDirection, PrincipalAdminTypes, SupportedTransports } from "../../../utils/enums";
import { GET_BEACON_IDS_QUERY } from "../../../utils/queries";
import {
    BeaconIdNode,
    BeaconIdsQueryTopLevel,
    GetBeaconIdsQueryVariables,
} from "../../../utils/interfacesQuery";
import { UseOnlineBeaconIdsResult } from "./types";

interface UseOnlineBeaconIdsProps {
    typeFilters?: FilterBarOption[];
    selectedBeaconIds?: string[];
    viewOnlySelected?: boolean;
    viewOnePerHost?: boolean;
}

export function useOnlineBeaconIds({
    typeFilters = [],
    selectedBeaconIds = [],
    viewOnlySelected = false,
    viewOnePerHost = false,
}: UseOnlineBeaconIdsProps = {}): UseOnlineBeaconIdsResult {
    const queryVariables = useMemo(
        () =>
            buildBeaconIdsQuery({
                typeFilters,
                selectedBeaconIds: viewOnlySelected ? selectedBeaconIds : undefined,
            }),
        [typeFilters, viewOnlySelected, selectedBeaconIds]
    );

    const { data, previousData, error, networkStatus, refetch } =
        useQuery<BeaconIdsQueryTopLevel>(GET_BEACON_IDS_QUERY, {
            variables: queryVariables,
            notifyOnNetworkStatusChange: true,
            fetchPolicy: "cache-and-network",
        });
    
    const currentData = data ?? previousData;

    const filteredBeaconIds = useMemo(() => {
        if (!viewOnePerHost) {
            return currentData?.beacons?.edges.map((edge) => edge?.node?.id) || [];
        }
        const allBeaconNodes = currentData?.beacons?.edges.map((edge) => edge?.node) || [];
        return filterOnePerHost(allBeaconNodes);
    }, [currentData, viewOnePerHost]);

    return {
        beaconIds: filteredBeaconIds,
        totalCount: currentData?.beacons?.totalCount ?? 0,
        initialLoading: networkStatus === NetworkStatus.loading && !currentData,
        error,
        refetch,
    };
}

/**
 * Filters beacons to show only one per host, prioritizing:
 * 1. Admin privileges (root, SYSTEM, Administrator)
 * 2. More reliable transports (GRPC > HTTP1 > ICMP > DNS)
 */
function filterOnePerHost(beacons: BeaconIdNode[]): string[] {
    const adminPrincipals = Object.values(PrincipalAdminTypes) as string[];
    const hosts: { [hostId: string]: BeaconIdNode } = {};

    const getTransportPriority = (transport: string | undefined): number => {
        if (!transport) return 0;
        switch (transport) {
            case SupportedTransports.GRPC:
                return 4;
            case SupportedTransports.HTTP1:
                return 3;
            case SupportedTransports.ICMP:
                return 2;
            case SupportedTransports.DNS:
                return 1;
            default:
                return 0;
        }
    };

    const isAdmin = (principal: string | undefined): boolean => {
        if (!principal) return false;
        return adminPrincipals.includes(principal);
    };

    const shouldReplace = (current: BeaconIdNode, candidate: BeaconIdNode): boolean => {
        const currentIsAdmin = isAdmin(current.principal);
        const candidateIsAdmin = isAdmin(candidate.principal);

        // Prefer admin over non-admin
        if (!currentIsAdmin && candidateIsAdmin) {
            return true;
        }

        // If both are admin or both are non-admin, prefer better transport
        if (currentIsAdmin === candidateIsAdmin) {
            const currentPriority = getTransportPriority(current.transport);
            const candidatePriority = getTransportPriority(candidate.transport);
            return candidatePriority > currentPriority;
        }

        return false;
    };

    for (const beacon of beacons) {
        const hostId = beacon.host?.id;
        if (!hostId) continue;

        if (!(hostId in hosts)) {
            hosts[hostId] = beacon;
        } else if (shouldReplace(hosts[hostId], beacon)) {
            hosts[hostId] = beacon;
        }
    }

    return Object.values(hosts).map((beacon) => beacon.id);
}

interface BuildBeaconIdsQueryParams {
    typeFilters: FilterBarOption[];
    selectedBeaconIds?: string[];
}

function buildBeaconIdsQuery({
    typeFilters,
    selectedBeaconIds,
}: BuildBeaconIdsQueryParams): GetBeaconIdsQueryVariables {
    const currentTimestamp = new Date();

    const onlineFilter = {
        nextSeenAtGTE: sub(currentTimestamp, { seconds: 30 }).toISOString(),
    };

    const additionalFilters = buildFiltersFromTypeFilters(typeFilters);

    const selectedFilter = selectedBeaconIds?.length
        ? { idIn: selectedBeaconIds }
        : {};

    const where = {
        ...onlineFilter,
        ...additionalFilters,
        ...selectedFilter,
    };

    return {
        where,
        orderBy: [{ field: "CREATED_AT", direction: OrderDirection.Desc }],
    };
}

function buildFiltersFromTypeFilters(
    typeFilters: FilterBarOption[]
): Record<string, unknown> {
    if (typeFilters.length === 0) {
        return {};
    }

    const searchTypes = getBeaconFilterNameByTypes(typeFilters);
    const filters: Record<string, unknown> = {};

    // Beacon-level filters
    if (searchTypes.beacon.length > 0) {
        filters.nameIn = searchTypes.beacon;
    }
    if (searchTypes.principal.length > 0) {
        filters.principalIn = searchTypes.principal;
    }
    if (searchTypes.transport.length > 0) {
        filters.transportIn = searchTypes.transport;
    }

    // Host-level filters need to use hasHostWith
    const hostFilters: Record<string, unknown> = {};
    if (searchTypes.host.length > 0) {
        hostFilters.nameIn = searchTypes.host;
    }
    if (searchTypes.platform.length > 0) {
        hostFilters.platformIn = searchTypes.platform;
    }
    if (searchTypes.primaryIP.length > 0) {
        hostFilters.primaryIPIn = searchTypes.primaryIP;
    }

    // Tag filters on host
    const tagFilters: Array<Record<string, unknown>> = [];
    if (searchTypes.group.length > 0) {
        tagFilters.push({
            hasTagsWith: {
                kind: "group",
                nameIn: searchTypes.group,
            },
        });
    }
    if (searchTypes.service.length > 0) {
        tagFilters.push({
            hasTagsWith: {
                kind: "service",
                nameIn: searchTypes.service,
            },
        });
    }

    if (tagFilters.length > 0) {
        hostFilters.and = tagFilters;
    }

    if (Object.keys(hostFilters).length > 0) {
        filters.hasHostWith = hostFilters;
    }

    return filters;
}
