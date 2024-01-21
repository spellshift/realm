import { useQuery } from "@apollo/client";
import { formatDistance } from "date-fns";
import { useCallback, useEffect, useState } from "react";
import { HostType } from "../../../utils/consts";
import { PrincipalAdminTypes } from "../../../utils/enums";
import { GET_HOST_QUERY } from "../../../utils/queries";
import { convertArrayOfObjectsToObject, getOfflineOnlineStatus } from "../../../utils/utils";

export const useHostTable = () => {
    const [hosts, setHosts] = useState([]);
    const { loading, data, error, startPolling, stopPolling } = useQuery(GET_HOST_QUERY);

    const formatPrincipalData = useCallback((host: HostType) => {
        const uniqueObject = convertArrayOfObjectsToObject(host.beacons || [], 'principal');
        const princialUserList = Object.values(PrincipalAdminTypes) as Array<string>;
        const finalList = [] as Array<string>;
        for (const property in uniqueObject) {
            if(princialUserList.indexOf(property) !== -1){
                finalList.unshift(property);
            }
            else{
                finalList.push(property);
            }
        }
        return finalList;
    },[]);

    const getformattedHosts = useCallback((data: any)=> {
        const currentDate = new Date();

        if(!data || !data.hosts){
            return;
        }

        const hosts = data?.hosts?.map((host: any)=>{
            return {
                ...host,
                beaconPrincipals: formatPrincipalData(host),
                beaconStatus: getOfflineOnlineStatus(host.beacons),
                formattedLastSeenAt: formatDistance(new Date(host.lastSeenAt), currentDate)
            }
        })
        setHosts(hosts);
    },[formatPrincipalData]) as any;


    useEffect(() => {
        //Update host data
        startPolling(60000);
        return () => {
            stopPolling();
        }
    }, [startPolling, stopPolling])

    useEffect(()=> {
        if(data){
            getformattedHosts(data);
        }
    },[data, getformattedHosts])

    return {
        loading,
        hosts,
        error
    }
}
