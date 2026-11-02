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

// Hosts page modular search bar: one row is a Field + Predicate + Value triple.
// Rows AND together; multi-value predicates OR within the row.
export type HostFilterField =
    | 'platform'
    | 'transport'
    | 'status'
    | 'service'
    | 'group'
    | 'principal'
    | 'primaryIP'
    | 'externalIP'
    | 'host'
    | 'beacon';

export type HostFilterPredicate =
    | 'is'
    | 'is-not'
    | 'is-in'
    | 'is-not-in'
    | 'contains'
    | 'does-not-contain';

export interface HostFilterRow {
    id: string;
    field: HostFilterField;
    predicate: HostFilterPredicate;
    values: string[];
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
    externalIPs: Array<FilterBarOption>;
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
