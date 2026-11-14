import { useMemo } from "react";
import { useQuery } from "@apollo/client";
import { UserQueryTopLevel } from "../../utils/interfacesQuery";
import { FilterBarOption } from "../../utils/interfacesUI";
import { GET_USER_QUERY } from "../../utils/queries";
import { ASSET_TYPE_OPTIONS } from "./constants";
import { FilterOptionGroup, UseAssetFilterBarResult } from "./types";

export function useAssetFilterBar(): UseAssetFilterBarResult {
    const { data, loading: isLoading, error } = useQuery<UserQueryTopLevel>(GET_USER_QUERY);

    const options = useMemo(() => {
        const users: FilterBarOption[] = (data?.users.edges ?? []).map(({ node }) => ({
            id: node.id,
            name: node.name,
            value: node.id,
            label: node.name,
            kind: "user",
        }));

        // Users first, then agent type -- per the combined filter's layout.
        const groups: FilterOptionGroup[] = [
            { label: "Users", options: users },
            { label: "Type", options: ASSET_TYPE_OPTIONS },
        ];

        return groups;
    }, [data]);

    return { options, isLoading, error };
}
