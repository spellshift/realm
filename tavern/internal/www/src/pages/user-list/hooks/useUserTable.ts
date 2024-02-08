import { useQuery } from "@apollo/client";
import { formatDistance } from "date-fns";
import { useCallback, useEffect, useState } from "react";
import { UserType } from "../../../utils/consts";
import { PrincipalAdminTypes } from "../../../utils/enums";
import { GET_USER_QUERY } from "../../../utils/queries";
import { convertArrayOfObjectsToObject, getOfflineOnlineStatus } from "../../../utils/utils";

export const useUserTable = () => {
    const [users, setUsers] = useState([]);
    const { loading, data, error, startPolling, stopPolling } = useQuery(GET_USER_QUERY, {
        variables: {
            "where": {

            }
        }
    });

    useEffect(() => {
        startPolling(60000);
        return () => {
            stopPolling();
        }
    }, [startPolling, stopPolling])

    useEffect(()=> {
        if(data) {
            console.log(data.users);
            setUsers(data.users);
        }
    },[data])

    return {
        loading,
        users,
        error
    }
}
