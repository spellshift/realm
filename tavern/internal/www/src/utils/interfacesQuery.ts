import { OrderDirection, QuestOrderField, TaskOrderField, HostOrderField, RepositoryOrderField } from "./enums";

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

export interface UserQueryTopLevel {
    users: UserQueryResponse;
}

export interface UserQueryResponse {
    pageInfo: QueryPageInfo;
    totalCount: number;
    edges: UserEdge[];
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
    description?: string;
    eldritch: string;
    tactic: string;
    paramDefs: string | null;
    supportModel: string;
    uploader?: UserNode | null;
}

// Quest-related interfaces
export interface TaskCountOnly {
    totalCount: number;
}

export interface LastModifiedTaskNode {
    lastModifiedAt: string;
}

export interface LastModifiedTaskEdge {
    node: LastModifiedTaskNode;
}

export interface LastUpdatedTaskConnection {
    edges: LastModifiedTaskEdge[];
}

export interface QuestNode {
    id: string;
    name: string;
    creator: UserNode;
    tome: TomeNode;
    parameters: string | null;
    tasks: TaskQueryResponse;
    lastUpdatedTask?: LastUpdatedTaskConnection;
    tasksTotal?: TaskCountOnly;
    tasksOutput?: TaskCountOnly;
    tasksError?: TaskCountOnly;
    tasksFinished?: TaskCountOnly;
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

export interface QuestEdge {
    node: QuestNode;
}

export interface QuestQueryResponse {
    totalCount: number;
    pageInfo: QueryPageInfo;
    edges: QuestEdge[];
}

export interface QuestQueryTopLevel {
    quests: QuestQueryResponse;
}

export interface OrderByField<T = string> {
    direction: OrderDirection;
    field: T;
}

export type QuestOrderBy = OrderByField<QuestOrderField>;
export type TaskOrderBy = OrderByField<TaskOrderField>;
export type HostOrderBy = OrderByField<HostOrderField>;
export type RepositoryOrderBy = OrderByField<RepositoryOrderField>;

export interface QuestWhereInput {
    id?: string;
    nameContains?: string;
    [key: string]: unknown;
}

export interface TaskWhereInput {
    execFinishedAtNotNil?: boolean;
    outputSizeGT?: number;
    errorNotNil?: boolean;
    [key: string]: unknown;
}

export interface GetQuestQueryVariables {
    where?: QuestWhereInput;
    whereTotalTask?: TaskWhereInput;
    whereFinishedTask?: TaskWhereInput;
    whereOutputTask?: TaskWhereInput;
    whereErrorTask?: TaskWhereInput;
    firstTask?: number;
    orderByTask?: OrderByField[];
    first?: number;
    last?: number;
    after?: Cursor;
    before?: Cursor;
    orderBy?: OrderByField[];
}

export interface TomeEdge {
    node: TomeNode;
}

export interface TomeQueryResponse {
    edges: TomeEdge[];
}

export interface TomeQueryTopLevel {
    tomes: TomeQueryResponse;
}

export interface GetTomesQueryVariables {
    where?: any;
}

export interface RepositoryNode {
    id: string;
    lastModifiedAt: string;
    url: string;
    publicKey: string;
    tomes: {
        edges: TomeEdge[];
    };
    owner: UserNode | null;
}

export interface RepositoryEdge {
    node: RepositoryNode;
}

export interface RepositoryQueryResponse {
    edges: RepositoryEdge[];
}

export interface RepositoryQueryTopLevel {
    repositories: RepositoryQueryResponse;
}

export interface GetRepositoryQueryVariables {
    orderBy?: RepositoryOrderBy[];
}
