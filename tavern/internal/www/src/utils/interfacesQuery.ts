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
    id?: string;
    createdAt: string;
    lastModifiedAt: string;
    principal: string;
    kind: string;
    secret: string;
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

export interface GetQueryVariables {
    where?: any;
    first?: number;
    last?: number;
    after?: string;
    before?: string;
    orderBy?: any[];
}

export interface TagContextProps {
    beacons: Array<BeaconNode>;
    groupTags: Array<TagNode>;
    serviceTags: Array<TagNode>;
    hosts: Array<HostNode>;
}

export interface UserNode {
    id: string;
    name: string;
    photoURL: string;
    isActivated: boolean;
    isAdmin: boolean;
}

export interface UserEdge {
    node: UserNode;
}

export interface ShellNode {
    id: string;
    closedAt: string | null;
    activeUsers: {
        edges: UserEdge[];
    };
}

export interface ShellEdge {
    node: ShellNode;
}

export interface TomeNode {
    id: string;
    name: string;
    description: string;
    eldritch: string;
    tactic: string;
    paramDefs: string | null;
    supportModel: string;
}

export interface QuestNode {
    id: string;
    name: string;
    creator: UserNode;
    tome: TomeNode;
    parameters: string | null;
}

export interface TaskNode {
    id: string;
    lastModifiedAt: string;
    outputSize: number;
    execStartedAt: string | null;
    execFinishedAt: string | null;
    createdAt: string;
    claimedAt: string | null;
    error: string | null;
    output: string | null;
    shells: {
        edges: ShellEdge[];
    };
    quest: QuestNode;
    beacon: BeaconNode;
}

export interface TaskEdge {
    node: TaskNode;
}

export interface TaskQueryResponse {
    pageInfo: QueryPageInfo;
    totalCount: number;
    edges: TaskEdge[];
}

export interface TaskQueryTopLevel {
    tasks: TaskQueryResponse;
}

export interface HostCredentialsNode {
    credentials: {
        edges: CredentialEdge[];
    };
}

export interface HostCredentialsEdge {
    node: HostCredentialsNode;
}

export interface HostCredentialsQueryResponse {
    edges: HostCredentialsEdge[];
}

export interface HostCredentialsQueryTopLevel {
    hosts: HostCredentialsQueryResponse;
}
