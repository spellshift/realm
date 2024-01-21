import { useQuery } from "@apollo/client";
import { formatDistance } from "date-fns";
import { useCallback, useEffect, useState } from "react";
import { GET_HOST_QUERY } from "../../../utils/queries";
import { getOfflineOnlineStatus } from "../../../utils/utils";

export const useHostTable = () => {
    const [hosts, setHosts] = useState([]);
    const { loading, data, error, startPolling, stopPolling } = useQuery(GET_HOST_QUERY);

    const getformattedHosts = useCallback((data: any)=> {
        const currentDate = new Date();

        if(!data || !data.hosts){
            return;
        }

        const hosts = data?.hosts?.map((host: any)=>{
            return {
                ...host,
                beaconStatus: getOfflineOnlineStatus(host.beacons),
                formattedLastSeenAt: formatDistance(new Date(host.lastSeenAt), currentDate)
            }
        })
        setHosts(hosts);
    },[]) as any;


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
    },[data])

    return {
        loading,
        hosts,
        error
    }
}
