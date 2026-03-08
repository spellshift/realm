import { useCallback } from "react";
import Select, { createFilter, MultiValue } from "react-select";
import { FilterBarOption } from "../../utils/interfacesUI";
import { useBeaconFilterBar } from "./useBeaconFilterBar";
import { BeaconFilterBarProps } from "./types";

export function BeaconFilterBar({
    value,
    defaultValue,
    onChange,
    isDisabled,
    hideStatusFilter,
}: BeaconFilterBarProps) {
    const { options, isLoading } = useBeaconFilterBar({ hideStatusFilter });

    const handleChange = useCallback(
        (selected: MultiValue<FilterBarOption>) => {
            onChange([...selected]);
        },
        [onChange]
    );

    return (
        <div className="flex flex-col gap-1">
            <label className=" font-medium text-gray-700">Beacon fields</label>
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
                defaultValue={defaultValue}
            />
        </div>
    );
}
