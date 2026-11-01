import { useQuery, NetworkStatus } from "@apollo/client";
import { useMemo} from "react";
import { sub } from "date-fns";
import { HostFilterRow } from "../../../utils/interfacesUI";
import { constructBeaconIdsFilterQuery } from "../../../utils/constructQueryUtils";
import { OrderDirection, PrincipalAdminTypes, SupportedTransports } from "../../../utils/enums";
import { GET_BEACON_IDS_QUERY } from "../../../utils/queries";
import {
    BeaconIdNode,
    BeaconIdsQueryTopLevel,
    GetBeaconIdsQueryVariables,
} from "../../../utils/interfacesQuery";
import { UseOnlineBeaconIdsResult } from "./types";

const NO_SELECTION: string[] = [];
const NO_ROWS: HostFilterRow[] = [];

interface UseOnlineBeaconIdsProps {
    rows?: HostFilterRow[];
    selectedBeaconIds?: string[];
    viewOnlySelected?: boolean;
    viewOnePerHost?: boolean;
}

export function useOnlineBeaconIds({
    rows = NO_ROWS,
    selectedBeaconIds = NO_SELECTION,
    viewOnlySelected = false,
    viewOnePerHost = false,
}: UseOnlineBeaconIdsProps = {}): UseOnlineBeaconIdsResult {
    const queryVariables = useMemo(
        () =>
            buildBeaconIdsQuery({
                rows,
                selectedBeaconIds: viewOnlySelected ? selectedBeaconIds : undefined,
            }),
        [rows, viewOnlySelected, selectedBeaconIds]
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
                return 5;
            case SupportedTransports.QUIC:
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
    rows: HostFilterRow[];
    selectedBeaconIds?: string[];
}

function buildBeaconIdsQuery({
    rows,
    selectedBeaconIds,
}: BuildBeaconIdsQueryParams): GetBeaconIdsQueryVariables {
    const currentTimestamp = new Date();

    const onlineFilter = {
        nextSeenAtGTE: sub(currentTimestamp, { seconds: 30 }).toISOString(),
    };

    const additionalFilters = constructBeaconIdsFilterQuery(rows) ?? {};

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
