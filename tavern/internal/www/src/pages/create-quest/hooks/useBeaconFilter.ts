import { useCallback, useEffect, useState } from "react"
import { PrincipalAdminTypes, SupportedTransports } from "../../../utils/enums";
import { useFilters } from "../../../context/FilterContext";
import { BeaconNode, TagEdge } from "../../../utils/interfacesQuery";
import { SelectedBeacons } from "../../../utils/interfacesUI";
import { getBeaconFilterNameByTypes } from "../../../utils/utils";

export const useBeaconFilter = (beacons: Array<BeaconNode>, selectedBeacons: SelectedBeacons) => {
    const { filters } = useFilters();

    const [filteredBeacons, setFilteredBeacons] = useState(beacons);

    const [typeFilters, setTypeFilters] = useState(filters.beaconFields);

    // Sync typeFilters when filters.beaconFields changes (e.g., on page navigation reset)
    useEffect(() => {
        setTypeFilters(filters.beaconFields);
    }, [filters.beaconFields]);

    const [viewOnlySelected, setViewOnlySelected] = useState(false);

    const [viewOnlyOnePerHost, setViewOnlyOnePerHost] = useState(false);

    const filterByTypes = useCallback((filteredBeacons: Array<BeaconNode>) => {
        if (typeFilters.length < 1) {
            return filteredBeacons;
        }

        const searchTypes = getBeaconFilterNameByTypes(typeFilters);

        return filteredBeacons.filter((beacon: BeaconNode) => {
            let group = beacon?.host?.tags ? (beacon?.host?.tags?.edges).find((obj: TagEdge) => {
                return obj?.node.kind === "group"
            }) : null;

            let service = beacon?.host?.tags ? (beacon?.host?.tags?.edges).find((obj: TagEdge) => {
                return obj?.node.kind === "service"
            }) : null;

            let match = true;

            if (searchTypes.beacon.length > 0) {
                // If a beacon filter is applied ignore other filters to just match the beacon
                if (searchTypes.beacon.indexOf(beacon.name) > -1) {
                    match = true;
                }
                else {
                    return false;
                }
            }

            if (searchTypes.principal.length > 0) {
                if (searchTypes.principal.indexOf(beacon.principal) > -1) {
                    match = true;
                }
                else {
                    return false;
                }
            }

            if (searchTypes.host.length > 0) {
                if (beacon?.host?.id && searchTypes.host.indexOf(beacon?.host?.name) > -1) {
                    match = true;
                }
                else {
                    return false;
                }
            }

            if (searchTypes.service.length > 0) {
                if (service && searchTypes.service.indexOf(service?.node.name) > -1) {
                    match = true;
                }
                else {
                    return false;
                }
            }

            if (searchTypes.group.length > 0) {
                if (group && searchTypes.group.indexOf(group?.node.name) > -1) {
                    match = true;
                }
                else {
                    return false;
                }
            }

            if (searchTypes.platform.length > 0) {
                if (beacon?.host?.platform && searchTypes.platform.indexOf(beacon?.host?.platform) > -1) {
                    match = true;
                }
                else {
                    return false;
                }
            }

            if (searchTypes.transport.length > 0) {
                if (beacon?.transport && searchTypes.transport.indexOf(beacon?.transport) > -1) {
                    match = true;
                }
                else {
                    return false;
                }
            }

            if (searchTypes.primaryIP.length > 0) {
                if (beacon?.host?.primaryIP && searchTypes.primaryIP.indexOf(beacon?.host?.primaryIP) > -1) {
                    match = true;
                }
                else {
                    return false;
                }
            }

            return match;
        });
    }, [typeFilters]);

    const filterBySelected = useCallback((beacons: Array<BeaconNode>, selectedBeacons: SelectedBeacons) => {
        if (viewOnlySelected) {
            return beacons.filter((beacon: BeaconNode) => selectedBeacons[beacon?.id]);
        }
        else {
            return beacons;
        }
    }, [viewOnlySelected]);

    const filterByOnePerHost = useCallback((beacons: Array<BeaconNode>) => {
        if (viewOnlyOnePerHost) {
            const principals = Object.values(PrincipalAdminTypes) as Array<string>;
            const hosts = {} as { [key: string]: BeaconNode };

            // Transport priority: GRPC > HTTP1 > DNS
            const getTransportPriority = (transport: string | undefined): number => {
                if (!transport) return 0;
                switch (transport) {
                    case SupportedTransports.GRPC:
                        return 3;
                    case SupportedTransports.HTTP1:
                        return 2;
                    case SupportedTransports.DNS:
                        return 1;
                    default:
                        return 0;
                }
            };

            const isAdmin = (principal: string) => principals.indexOf(principal) !== -1;

            const shouldReplace = (current: BeaconNode, candidate: BeaconNode): boolean => {
                const currentIsAdmin = isAdmin(current.principal);
                const candidateIsAdmin = isAdmin(candidate.principal);

                // If candidate is admin and current is not, replace
                if (!currentIsAdmin && candidateIsAdmin) {
                    return true;
                }

                // If both have same admin status, prefer better transport
                if (currentIsAdmin === candidateIsAdmin) {
                    const currentPriority = getTransportPriority(current.transport);
                    const candidatePriority = getTransportPriority(candidate.transport);
                    return candidatePriority > currentPriority;
                }

                return false;
            };

            for (let beaconIndex in beacons) {
                const hostId = beacons[beaconIndex]?.host?.id;

                if (hostId && !(hostId in hosts)) {
                    hosts[hostId] = beacons[beaconIndex];
                }
                else if (hostId && shouldReplace(hosts[hostId], beacons[beaconIndex])) {
                    hosts[hostId] = beacons[beaconIndex];
                }
            }
            return Object.values(hosts);
        }
        else {
            return beacons;
        }
    }, [viewOnlyOnePerHost]);

    useEffect(() => {
        let filteredBeacons = filterBySelected(beacons, selectedBeacons);
        filteredBeacons = filterByOnePerHost(filteredBeacons);
        filteredBeacons = filterByTypes(filteredBeacons);
        setFilteredBeacons(
            filteredBeacons
        );
    }, [beacons, selectedBeacons, typeFilters, viewOnlySelected, viewOnlyOnePerHost, filterBySelected, filterByOnePerHost, filterByTypes]);

    return {
        filteredBeacons,
        setTypeFilters,
        viewOnlySelected,
        setViewOnlySelected,
        setViewOnlyOnePerHost,
        typeFilters
    }
}
