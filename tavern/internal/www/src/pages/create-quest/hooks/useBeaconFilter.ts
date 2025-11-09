import { useCallback, useEffect, useState } from "react"
import { BeaconType } from "../../../utils/consts";
import { PrincipalAdminTypes } from "../../../utils/enums";
import { useFilters } from "../../../context/FilterContext";

export const useBeaconFilter = (beacons: Array<BeaconType>, selectedBeacons: any) => {
    const {filters} = useFilters();
    const initialFilters = (filters.filtersEnabled && filters.beaconFields) ? filters.beaconFields : [];

    const [filteredBeacons, setFilteredBeacons] = useState(beacons);

    const [typeFilters, setTypeFilters] = useState(initialFilters);

    const [viewOnlySelected, setViewOnlySelected] = useState(false);

    const [viewOnlyOnePerHost, setViewOnlyOnePerHost] = useState(false);

    function getSearchTypes(typeFilters: any){
        return typeFilters.reduce((accumulator:any, currentValue:any) => {
            if(currentValue.kind === "beacon"){
                accumulator.beacon.push(currentValue.value);
            }
            else if(currentValue.kind === "platform"){
                accumulator.platform.push(currentValue.value);
            }
            else if(currentValue.kind === "service"){
                accumulator.service.push(currentValue.value);
            }
            else if(currentValue.kind === "group"){
                accumulator.group.push(currentValue.value);
            }
            else if(currentValue.kind === "host"){
                accumulator.host.push(currentValue.value);
            }
            return accumulator;
        },
        {
            "beacon": [],
            "service": [],
            "host": [],
            "group": [],
            "platform": []
        });
    };

    const filterByTypes = useCallback((filteredBeacons: Array<BeaconType>) => {
        if(typeFilters.length < 1){
            return filteredBeacons;
        }

        const searchTypes = getSearchTypes(typeFilters);

        return filteredBeacons.filter( (beacon) => {
            let group = beacon?.host?.tags ? (beacon?.host?.tags).find( (obj : any) => {
                return obj?.kind === "group"
            }) : null;

            let service = beacon?.host?.tags ? (beacon?.host?.tags).find( (obj : any) => {
                return obj?.kind === "service"
            }) : null;

            let match = true;

            if(searchTypes.beacon.length > 0){
                // If a beacon filter is applied ignore other filters to just match the beacon
                if(searchTypes.beacon.indexOf(beacon.id) > -1){
                    match = true;
                }
                else{
                    return false;
                }
            }

            if(searchTypes.host.length > 0){
                if(searchTypes.host.indexOf(beacon?.host?.id) > -1){
                    match = true;
                }
                else{
                    return false;
                }
            }

            if(searchTypes.service.length > 0){
                if(service && searchTypes.service.indexOf(service?.id) > -1){
                    match = true;
                }
                else{
                    return false;
                }
            }

            if(searchTypes.group.length > 0){
                if(group && searchTypes.group.indexOf(group?.id) > -1){
                    match = true;
                }
                else{
                    return false;
                }
            }

            if(searchTypes.platform.length > 0){
                if(searchTypes.platform.indexOf(beacon?.host?.platform) > -1){
                    match = true;
                }
                else{
                    return false;
                }
            }

            return match;
        });
    },[typeFilters]);

    const filterBySelected = useCallback((beacons: Array<BeaconType>, selectedBeacons: any) => {
        if(viewOnlySelected){
            return beacons.filter((beacon: BeaconType)=> selectedBeacons[beacon?.id]);
        }
        else{
            return beacons;
        }
    },[viewOnlySelected]);

    const filterByOnePerHost = useCallback((beacons: Array<BeaconType>) => {
        if(viewOnlyOnePerHost){
            const princials = Object.values(PrincipalAdminTypes) as Array<string>;
            const hosts = {} as {[key: string]: BeaconType};

            for(let beaconIndex in beacons){
                const hostId = beacons[beaconIndex].host.id;

                if( !(hostId in hosts) ){
                    hosts[hostId] = beacons[beaconIndex];
                }
                else if((princials.indexOf(hosts[hostId].principal) === -1) && (princials.indexOf(beacons[beaconIndex].principal) !== -1)){
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
    },[beacons, selectedBeacons, typeFilters, viewOnlySelected, viewOnlyOnePerHost]);

    return {
        filteredBeacons,
        setTypeFilters,
        viewOnlySelected,
        setViewOnlySelected,
        setViewOnlyOnePerHost,
        typeFilters
    }
}
