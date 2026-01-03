import { useMemo } from "react";
import Select, { createFilter, } from "react-select"
import { useTags } from "../../context/TagContext";

type Props = {
    setFiltersSelected: (arg1: any) => void;
    filtersSelected?: any;
    initialFilters?: any;
    isDisabled?: boolean;
}
export const BeaconFilterBar = (props: Props) => {
    const { data } = useTags();
    const { beacons, groupTags, serviceTags, hosts, principals, primaryIPs, platforms, transports, onlineOfflineStatus } = data;

    const { setFiltersSelected, filtersSelected, isDisabled, initialFilters } = props;

    const options = useMemo(() => [
        {
            label: "Platform",
            options: platforms
        },
        {
            label: "Transport",
            options: transports
        },
        {
            label: "Status",
            options: onlineOfflineStatus
        },
        {
            label: "Service",
            options: serviceTags
        },
        {
            label: "Group",
            options: groupTags
        },
        {
            label: "Principal",
            options: principals
        },
        {
            label: "PrimaryIPs",
            options: primaryIPs
        },
        {
            label: "Host",
            options: hosts
        },
        {
            label: "Beacon",
            options: beacons
        }
    ], [platforms, serviceTags, groupTags, principals, primaryIPs, hosts, beacons, transports, onlineOfflineStatus]);


    return (
        <div className="flex flex-col gap-1">
            <label className=" font-medium text-gray-700">Beacon fields</label>
            <Select
                isDisabled={isDisabled}
                isSearchable={true}
                isMulti
                options={options}
                onChange={setFiltersSelected}
                filterOption={createFilter({
                    matchFrom: 'any',
                    stringify: option => `${option.label}`,
                })}
                value={filtersSelected || undefined}
                defaultValue={initialFilters || undefined}
            />
        </div>
    );
}
