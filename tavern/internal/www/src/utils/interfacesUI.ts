import { TomeNode, UserNode } from "./interfacesQuery";

export type KindOfTag = 'service' | 'group';

export interface FilterBarOption {
    label?: string;
    value?: string;
    kind: string;
    id: string;
    name: string;
}

export interface OnlineOfflineStatus {
    online: number;
    offline: number;
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
