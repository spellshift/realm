import React from "react";
import {Heading} from "@chakra-ui/react";
import Select,  { createFilter } from "react-select"
import { BeaconType, TomeTag } from "../../utils/consts";
import { SupportedPlatforms } from "../../utils/enums";

type Props = {
    setFiltersSelected: (arg1: any) => void;
    beacons: Array<BeaconType>;
    groups: Array<TomeTag>;
    services: Array<TomeTag>;
}
export const BeaconFilterBar = (props: Props) => {
    // TODO add host to filter

    const {setFiltersSelected, beacons, groups, services} = props;
    const supportedPlatformsList = Object.values(SupportedPlatforms);

    const getFormattedOptions = (beacons: Array<BeaconType>, groups: Array<TomeTag>, services: Array<TomeTag>) => {
        return [
            {
                label: "Platform",
                options: supportedPlatformsList.map(function(platform: string){
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
                options: services.map(function(service: TomeTag){
                    return {
                        ...service,
                        value: service?.id,
                        label: service?.name,
                        kind: service?.kind
                    }})
            },
            {
                label: "Group",
                options: groups.map(function(group: TomeTag){
                    return {
                        ...group,
                        value: group?.id,
                        label: group?.name,
                        kind: group?.kind
                    };
                })
            },
            {
                label: "Beacon",
                options: beacons.map(function(beacon: BeaconType){
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
        <div>
            <Heading size="sm" mb={2}> Filter by platform, service, group, and beacon</Heading>
            <Select
                isSearchable={true}
                isMulti
                options={getFormattedOptions(beacons, groups, services)}
                onChange={setFiltersSelected}
                filterOption={createFilter({
                    matchFrom: 'any',
                    stringify: option => `${option.label}`,
                  })}
            />
        </div>
    );
}