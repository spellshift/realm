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

export type TomeInputParams = {
    name: string;
    label: string;
    type: string;
    placeholder: string;
    value?: any;
}
