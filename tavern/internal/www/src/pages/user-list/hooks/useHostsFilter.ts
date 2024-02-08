import { useCallback, useEffect, useState } from "react"
import { HostType } from "../../../utils/consts";

export const useHostsFilter = (hosts: Array<HostType>) => {
    const [loading, setLoading] = useState(false);
    const [filteredHosts, setFilteredHosts] = useState(hosts);

    const [typeFilters, setTypeFilters] = useState([]) as Array<any>;

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

    const filterByTypes = useCallback((filteredHosts: Array<HostType>) => {
        if(typeFilters.length < 1){
            return filteredHosts;
        }

        const searchTypes = getSearchTypes(typeFilters);

        return filteredHosts.filter( (host) => {
            let group = host?.tags ? (host?.tags).find( (obj : any) => {
                return obj?.kind === "group"
            }) : null;

            let service = host?.tags ? (host?.tags).find( (obj : any) => {
                return obj?.kind === "service"
            }) : null;

            let match = true;

            if(searchTypes.beacon.length > 0){
                // If a beacon filter is applied ignore other filters to just match the beacon
                if(host?.beacons?.some(beacon=> searchTypes.beacon.includes(beacon.id))){
                    match = true;
                }
                else{
                    return false;
                }
            }

            if(searchTypes.host.length > 0){
                if(searchTypes.host.indexOf(host?.id) > -1){
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
                if(searchTypes.platform.indexOf(host?.platform) > -1){
                    match = true;
                }
                else{
                    return false;
                }
            }

            return match;
        });
    },[typeFilters]);

    useEffect(()=> {
        if(hosts.length > 0){
            setLoading(true);

            const filtered = filterByTypes(hosts);
            setFilteredHosts(
                filtered
            );
            setLoading(false);
        }
    },[hosts, typeFilters, filterByTypes]);

    return {
        loading,
        filteredHosts,
        setTypeFilters
    }
}
