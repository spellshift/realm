import { useQuery } from "@apollo/client";
import { useCallback, useEffect, useState } from "react";
import { GET_HOST_CREDENTIALS } from "../../../utils/queries";
import { CredentialType, HostType } from "../../../utils/consts";

export const useCredsTable = (hostId: number) => {
    const [creds, setCreds] = useState([] as CredentialType[]);
    const { loading, data, error, startPolling, stopPolling } = useQuery(GET_HOST_CREDENTIALS, {
        variables: {
            "where": {
                "id": hostId
                }
            }
        });

    const getCreds = useCallback((data: any)=> {
        if(!data || !data.hosts) {
            return;
        }

        const hosts: HostType[] = data?.hosts

        const creds: CredentialType[] = hosts[0].credentials!.map(((cred) => {
            return {
                ...cred,
                kind: cred.kind.replace(/^KIND_/, "")
            }
        }));
    
        setCreds(creds);
    }, []) as any;


    useEffect(() => {
        //Update host data
        startPolling(60000);
        return () => {
            stopPolling();
        }
    }, [startPolling, stopPolling])

    useEffect(()=> {
        if(data){
            getCreds(data);
        }
    },[data, getCreds])

    return {
        loading,
        creds,
        error
    }
}
