import { BeaconNode, HostNode, TagNode, TomeNode, UserNode } from "./interfacesQuery";
import { TomeFilterFieldKind } from "./enums";

export type KindOfTag = 'service' | 'group';

export interface FilterBarOption {
    label?: string;
    value?: string;
    kind: string;
    id: string;
    name: string;
}

export interface FilterOptionGroup {
    label: string;
    options: FilterBarOption[];
}

export interface TagContextProps {
    beacons: Array<FilterBarOption & BeaconNode>;
    groupTags: Array<FilterBarOption & TagNode>;
    serviceTags: Array<FilterBarOption & TagNode>;
    hosts: Array<FilterBarOption & HostNode>;
    principals: Array<FilterBarOption>;
    primaryIPs: Array<FilterBarOption>;
    platforms: Array<FilterBarOption>;
    transports: Array<FilterBarOption>;
    onlineOfflineStatus: Array<FilterBarOption>;
}

export interface OnlineOfflineStatus {
    online: number;
    offline: number;
}

export type TomeFiltersByType = {
    [TomeFilterFieldKind.SupportModel]: Array<string>,
    [TomeFilterFieldKind.Tactic]: Array<string>
}

export type FieldInputParams = {
    name: string;
    label: string;
    type: string;
    placeholder: string;
    value?: any;
}
export interface RepositoryRow {
    node: {
        id?: string;
        url: string;
        tomes: TomeNode[];
        owner?: UserNode | null;
        repoType?: string;
        lastModifiedAt?: string;
        publicKey?: string;
    };
}
export type SelectedBeacons = {
    [beaconId: string]: boolean
};
