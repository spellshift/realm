import { useCallback, useEffect, useState } from "react"
import { PrincipalAdminTypes } from "../../../utils/enums";
import { useFilters } from "../../../context/FilterContext";
import { BeaconNode, TagEdge } from "../../../utils/interfacesQuery";
import { SelectedBeacons } from "../../../utils/interfacesUI";
import { getBeaconFilterNameByTypes } from "../../../utils/utils";

export const useBeaconFilter = (beacons: Array<BeaconNode>, selectedBeacons: SelectedBeacons) => {
    const {filters} = useFilters();
    const initialFilters = (filters.filtersEnabled && filters.beaconFields) ? filters.beaconFields : [];

    const [filteredBeacons, setFilteredBeacons] = useState(beacons);

    const [typeFilters, setTypeFilters] = useState(initialFilters);

    const [viewOnlySelected, setViewOnlySelected] = useState(false);

    const [viewOnlyOnePerHost, setViewOnlyOnePerHost] = useState(false);

    const filterByTypes = useCallback((filteredBeacons: Array<BeaconNode>) => {
        if(typeFilters.length < 1){
            return filteredBeacons;
        }

        const searchTypes = getBeaconFilterNameByTypes(typeFilters);

        return filteredBeacons.filter( (beacon: BeaconNode) => {
            let group = beacon?.host?.tags ? (beacon?.host?.tags?.edges).find( (obj : TagEdge) => {
                return obj?.node.kind === "group"
            }) : null;

            let service = beacon?.host?.tags ? (beacon?.host?.tags?.edges).find( (obj : TagEdge) => {
                return obj?.node.kind === "service"
            }) : null;

            let match = true;

            if(searchTypes.beacon.length > 0){
                // If a beacon filter is applied ignore other filters to just match the beacon
                if(searchTypes.beacon.indexOf(beacon.name) > -1){
                    match = true;
                }
                else{
                    return false;
                }
            }

            if(searchTypes.principal.length > 0){
                if(searchTypes.principal.indexOf(beacon.principal) > -1){
                    match = true;
                }
                else{
                    return false;
                }
            }

            if(searchTypes.host.length > 0){
                if(beacon?.host?.id && searchTypes.host.indexOf(beacon?.host?.name) > -1){
                    match = true;
                }
                else{
                    return false;
                }
            }

            if(searchTypes.service.length > 0){
                if(service && searchTypes.service.indexOf(service?.node.name) > -1){
                    match = true;
                }
                else{
                    return false;
                }
            }

            if(searchTypes.group.length > 0){
                if(group && searchTypes.group.indexOf(group?.node.name) > -1){
                    match = true;
                }
                else{
                    return false;
                }
            }

            if(searchTypes.platform.length > 0){
                if(beacon?.host?.platform && searchTypes.platform.indexOf(beacon?.host?.platform) > -1){
                    match = true;
                }
                else{
                    return false;
                }
            }

            if(searchTypes.transport.length > 0){
                if(beacon?.transport && searchTypes.transport.indexOf(beacon?.transport) > -1){
                    match = true;
                }
                else{
                    return false;
                }
            }

            if(searchTypes.primaryIP.length > 0){
                if(beacon?.host?.primaryIP && searchTypes.primaryIP.indexOf(beacon?.host?.primaryIP) > -1){
                    match = true;
                }
                else{
                    return false;
                }
            }

            return match;
        });
    },[typeFilters]);

    const filterBySelected = useCallback((beacons: Array<BeaconNode>, selectedBeacons: SelectedBeacons) => {
        if(viewOnlySelected){
            return beacons.filter((beacon: BeaconNode)=> selectedBeacons[beacon?.id]);
        }
        else{
            return beacons;
        }
    },[viewOnlySelected]);

    const filterByOnePerHost = useCallback((beacons: Array<BeaconNode>) => {
        if(viewOnlyOnePerHost){
            const principals = Object.values(PrincipalAdminTypes) as Array<string>;
            const hosts = {} as {[key: string]: BeaconNode};

            for(let beaconIndex in beacons){
                const hostId = beacons[beaconIndex]?.host?.id;

                if( hostId && !(hostId in hosts) ){
                    hosts[hostId] = beacons[beaconIndex];
                }
                else if(hostId && (principals.indexOf(hosts[hostId].principal) === -1) && (principals.indexOf(beacons[beaconIndex].principal) !== -1)){
                    hosts[hostId] = beacons[beaconIndex];
                }
            }
            return Object.values(hosts);
        }
        else{
            return beacons;
        }
    },[viewOnlyOnePerHost]);

    useEffect(()=> {
       let filteredBeacons = filterBySelected(beacons, selectedBeacons);
       filteredBeacons = filterByOnePerHost(filteredBeacons);
       filteredBeacons = filterByTypes(filteredBeacons);
       setFilteredBeacons(
        filteredBeacons
       );
    },[beacons, selectedBeacons, typeFilters, viewOnlySelected, viewOnlyOnePerHost, filterBySelected, filterByOnePerHost, filterByTypes]);

    return {
        filteredBeacons,
        setTypeFilters,
        viewOnlySelected,
        setViewOnlySelected,
        setViewOnlyOnePerHost,
        typeFilters
    }
}
