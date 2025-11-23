import React from "react";
import Select, { createFilter, } from "react-select"
import { BeaconType, HostType, TomeTag } from "../../utils/consts";
import { SupportedPlatforms } from "../../utils/enums";

type Props = {
    setFiltersSelected: (arg1: any) => void;
    beacons: Array<BeaconType>;
    groups: Array<TomeTag>;
    services: Array<TomeTag>;
    hosts: Array<HostType>;
    filtersSelected?: any;
    initialFilters?: any;
    isDisabled?: boolean;
}
export const BeaconFilterBar = (props: Props) => {
    // TODO add host to filter

    const { setFiltersSelected, beacons, groups, services, hosts, filtersSelected, isDisabled, initialFilters } = props;
    const supportedPlatformsList = Object.values(SupportedPlatforms);

    // TODO: IN the future lets style things purple
    // const styles = {
    //     control: (base: any) => ({
    //         ...base,
    //         "&:focus": {
    //             borderColor: "#a855f7"
    //         },
    //         "&:hover": {
    //             borderColor: "#a855f7",
    //             color: "#a855f7"
    //         }
    //     }),
    //     dropdownIndicator: (base: any) => ({
    //         ...base,
    //         color: "inherit",
    //     }),
    //     singleValue: (base: any) => ({
    //         ...base,
    //         color: "inherit"
    //     }),
    //     option: (base: any, state: any) => ({
    //         ...base,
    //         "&:hover": {
    //             backgroundColor: "#a855f7",
    //             borderColor: "#a855f7",
    //             color: "white"
    //         }
    //     })
    // };

    const getFormattedOptions = (beacons: Array<BeaconType>, groups: Array<TomeTag>, services: Array<TomeTag>, hosts: Array<HostType>) => {
        return [
            {
                label: "Platform",
                options: supportedPlatformsList.map(function (platform: string) {
                    return {
                        name: platform,
                        value: platform,
                        label: platform,
                        kind: "platform"
                    };
                })
            },
            {
                label: "Service",
                options: services.map(function (service: TomeTag) {
                    return {
                        ...service,
                        value: service?.id,
                        label: service?.name,
                        kind: service?.kind
                    }
                })
            },
            {
                label: "Group",
                options: groups.map(function (group: TomeTag) {
                    return {
                        ...group,
                        value: group?.id,
                        label: group?.name,
                        kind: group?.kind
                    };
                })
            },
            {
                label: "Host",
                options: hosts.map(function (host: HostType) {
                    return {
                        ...host,
                        value: host?.id,
                        label: host?.name,
                        kind: "host"
                    };
                })
            },
            {
                label: "Beacon",
                options: beacons.map(function (beacon: BeaconType) {
                    return {
                        ...beacon,
                        value: beacon?.id,
                        label: beacon?.name,
                        kind: "beacon"
                    };
                })
            }
        ];
    };

    return (
        <div className="flex flex-col gap-1">
            <label className=" font-medium text-gray-700">Beacon fields</label>
            <Select
                isDisabled={isDisabled}
                isSearchable={true}
                isMulti
                options={getFormattedOptions(beacons, groups, services, hosts)}
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
