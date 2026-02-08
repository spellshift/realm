import { useQuery } from "@apollo/client";
import Select from "react-select";
import { UserQueryTopLevel } from "../utils/interfacesQuery";
import { GET_USER_QUERY } from "../utils/queries";

export interface UserFilterProps {
    value: string;
    onChange: (value: string) => void;
    label?: string;
    placeholder?: string;
    isDisabled?: boolean;
}

const UserFilter = ({
    value,
    onChange,
    label = "User",
    placeholder = "Filter by User",
    isDisabled = false,
}: UserFilterProps) => {
    const { data, loading } = useQuery<UserQueryTopLevel>(GET_USER_QUERY);

    const options = data?.users.edges.map((edge) => ({
        value: edge.node.id,
        label: edge.node.name,
    })) || [];

    const selectedOption = options.find((option) => option.value === value);

    return (
        <div className="flex flex-col gap-1">
            <label className="text-gray-700">{label}</label>
            <Select
                isDisabled={isDisabled || loading}
                isLoading={loading}
                isClearable
                placeholder={placeholder}
                options={options}
                value={selectedOption}
                onChange={(option) => {
                    onChange(option ? option.value : "");
                }}
            />
        </div>
    );
};

export default UserFilter;
