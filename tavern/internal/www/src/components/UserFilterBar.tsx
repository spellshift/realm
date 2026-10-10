import { useQuery } from "@apollo/client";
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

    const selectedOption = options.find((option) => option.value === filters.userId);

    return (
        <div className="flex flex-col gap-1">
            <label className="text-gray-700 font-medium">User</label>
            <Select
                isDisabled={loading}
                isLoading={loading}
                isClearable
                placeholder="Filter by User"
                options={options}
                value={selectedOption}
                onChange={(option) => {
                    updateFilters({ userId: option ? option.value : "" });
                }}
            />
        </div>
    );
};

export default UserFilterBar;
