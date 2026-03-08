import { ApolloError } from "@apollo/client";
import { FilterBarOption } from "../../utils/interfacesUI";

export interface FilterOptionGroup {
    label: string;
    options: FilterBarOption[];
}

export interface BeaconFilterBarProps {
    value?: FilterBarOption[];
    defaultValue?: FilterBarOption[];
    onChange: (selected: FilterBarOption[]) => void;
    isDisabled?: boolean;
    hideStatusFilter?: boolean;
}

export interface UseBeaconFilterBarProps {
    hideStatusFilter?: boolean;
}

export interface UseBeaconFilterBarResult {
    options: FilterOptionGroup[];
    isLoading: boolean;
    error: ApolloError | undefined;
}

export interface BeaconFilterOptionsResponse {
    groupTags: { edges: { node: { id: string; name: string; kind: string } }[] };
    serviceTags: { edges: { node: { id: string; name: string; kind: string } }[] };
    beacons: { edges: { node: { id: string; name: string; principal: string } }[] };
    hosts: { edges: { node: { id: string; name: string; primaryIP: string | null } }[] };
}