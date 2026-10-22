import Select from "react-select";
import { useFilters } from "../context/FilterContext/FilterContext";
import { BugAntIcon } from "@heroicons/react/24/outline";

const TYPE_OPTIONS = [
    { value: "agents", label: "Agents" },
];

export const AgentFilterBar = () => {
    const { filters, updateFilters } = useFilters();
    const selectedOption = filters.agentOnly ? TYPE_OPTIONS[0] : null;

    return (
        <div className="flex flex-col gap-1" data-testid="agent-filter-bar">
            <label className="text-gray-700 font-medium flex items-center gap-1.5">
                <BugAntIcon className="w-4 h-4 text-black" />
                Type
            </label>
            <Select
                isClearable
                placeholder="Filter by Type"
                options={TYPE_OPTIONS}
                value={selectedOption}
                onChange={(option) => {
                    updateFilters({ agentOnly: option?.value === "agents" });
                }}
            />
        </div>
    );
};

export default AgentFilterBar;
