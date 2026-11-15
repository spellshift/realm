import { useCallback, useMemo } from "react";
import Select, { createFilter, MultiValue } from "react-select";
import { FilterBarOption } from "../../utils/interfacesUI";
import { useAssetFilterBar } from "./useAssetFilterBar";
import { AssetFilterBarProps } from "./types";

// Combined "Asset fields" filter -- same single grouped-Select idiom as
// BeaconFilterBar (see components/beacon-filter-bar), replacing the
// separate User and Type selects that used to sit side by side. Users
// come first in the option groups, then the agent type filter.
export function AssetFilterBar({ userId, agentOnly, onChange, isDisabled }: AssetFilterBarProps) {
    const { options, isLoading } = useAssetFilterBar();

    const value = useMemo(() => {
        const flatOptions = options.flatMap((group) => group.options);
        return flatOptions.filter(
            (option) =>
                (option.kind === "user" && option.value === userId) ||
                (option.kind === "agent" && agentOnly)
        );
    }, [options, userId, agentOnly]);

    const handleChange = useCallback(
        (selected: MultiValue<FilterBarOption>) => {
            // At most one "user" option is ever meaningful -- take the most
            // recently selected one so picking a new user replaces the old.
            const selectedUser = [...selected].reverse().find((option) => option.kind === "user");
            const hasAgentOption = selected.some((option) => option.kind === "agent");
            onChange({ userId: selectedUser?.value ?? "", agentOnly: hasAgentOption });
        },
        [onChange]
    );

    return (
        <div className="flex flex-col gap-1">
            <label className="font-medium text-gray-700">Asset fields</label>
            <Select
                isDisabled={isDisabled}
                isLoading={isLoading}
                isSearchable
                isMulti
                options={options}
                onChange={handleChange}
                filterOption={createFilter({
                    matchFrom: "any",
                    stringify: (option) => `${option.label}`,
                })}
                value={value}
            />
        </div>
    );
}
