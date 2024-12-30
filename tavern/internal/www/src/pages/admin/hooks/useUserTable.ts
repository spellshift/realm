import { useQuery } from "@apollo/client";
import { UserType } from "../../../utils/consts";
import { GET_USER_QUERY } from "../../../utils/queries";

export const useUserTable = () => {
    const { loading, data, error } = useQuery(GET_USER_QUERY, {
        variables: {
            "where": {

            }
        }
    });

    const users: UserType[] = data?.users || [];

    return {
        loading,
        users,
        error
    };
}
