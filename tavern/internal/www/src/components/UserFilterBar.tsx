import { gql, useQuery } from "@apollo/client";
import Select from "react-select";
import { UserQueryTopLevel } from "../utils/interfacesQuery";
import { useFilters } from "../context/FilterContext/FilterContext";
import { GET_USER_QUERY } from "../utils/queries";

const UserFilterBar = () => {
    const { filters, updateFilters } = useFilters();
    const { data, loading } = useQuery<UserQueryTopLevel>(GET_USER_QUERY);

    const options = data?.users.edges.map((edge) => ({
        value: edge.node.id,
        label: edge.node.name,
    })) || [];

    const selectedOption = options.find((option) => option.value === filters.creatorId);

    return (
        <div className="w-64">
            <Select
                isDisabled={filters.isLocked || loading}
                isLoading={loading}
                isClearable
                placeholder="Filter by Creator"
                options={options}
                value={selectedOption}
                onChange={(option) => {
                    updateFilters({ creatorId: option ? option.value : "" });
                }}
                className="text-sm"
                styles={{
                    control: (base: any) => ({
                        ...base,
                        borderColor: "#E2E8F0",
                        "&:hover": {
                            borderColor: "#CBD5E0",
                        },
                    }),
                    menu: (base: any) => ({
                        ...base,
                        zIndex: 9999,
                    }),
                }}
            />
        </div>
    );
};

export default UserFilterBar;
