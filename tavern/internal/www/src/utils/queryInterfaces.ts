export type Cursor = string | null;

export interface QueryPageInfo {
    hasNextPage: boolean;
    hasPreviousPage: boolean;
    startCursor: Cursor;
    endCursor: Cursor;
}

export interface TagNode {
    name: string;
    id: string;
    kind: string;
}

export interface TagEdge {
    node: TagNode;
}

export interface BeaconNode {
    id: string;
    name: string;
    principal: string;
    interval: number;
    lastSeenAt: string;
    host?: HostNode
}

export interface BeaconEdge {
    node: BeaconNode;
}

export interface CredentialNode {
    id: string;
}

export interface CredentialEdge {
    node: CredentialNode;
}

export interface HostNode {
    id: string;
    name: string;
    primaryIP?: string;
    platform?: string;
    lastSeenAt?: string;
    tags?: {
        edges: TagEdge[];
    };
    beacons?: {
        edges: BeaconEdge[];
    };
    credentials?: {
        edges: CredentialEdge[];
    };
}

export interface HostEdge {
    node: HostNode;
}

export interface HostQueryTopLevel {
    hosts: HostQueryResponse;
}

export interface HostQueryResponse {
    pageInfo: QueryPageInfo;
    totalCount: number;
    edges: HostEdge[];
}

export interface TagContextQueryResponse {
    serviceTags: {edges: TagEdge[]};
    groupTags: {edges:  TagEdge[]};
    beacons: {edges: BeaconEdge[]};
    hosts: { edges: HostEdge[] };
}

export interface GetHostQueryVariables {
    where?: any; // HostWhereInput
    first?: number;
    last?: number;
    after?: string;
    before?: string;
    orderBy?: any[]; // HostOrder
}
export interface OnlineOfflineStatus {
    online: number;
    offline: number;
}

export interface TagContextProps {
    beacons: Array<BeaconNode>;
    groupTags: Array<TagNode>;
    serviceTags: Array<TagNode>;
    hosts: Array<HostNode>;
}
