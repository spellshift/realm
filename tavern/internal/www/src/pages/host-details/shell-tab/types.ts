import {
    QueryPageInfo,
    OrderByField,
    Cursor,
    UserNode
} from "../../../utils/interfacesQuery";

export interface ShellBeaconNode {
    id: string;
    name: string;
    lastSeenAt: string;
    interval: number;
}

export interface ShellNode {
    id: string;
    closedAt: string | null;
    beacon: ShellBeaconNode;
    owner: UserNode;
    activeUsers: {
        edges: { node: UserNode }[];
    };
    shellTasks: {
        totalCount: number;
    };
}

export interface ShellEdge {
    node: ShellNode;
}

export interface ShellsQueryResponse {
    totalCount: number;
    pageInfo: QueryPageInfo;
    edges: ShellEdge[];
}

export interface ShellsQueryTopLevel {
    shells: ShellsQueryResponse;
}

// Shell IDs query (for pagination)
export interface ShellIdNode {
    id: string;
}

export interface ShellIdEdge {
    node: ShellIdNode;
}

export interface ShellIdsQueryResponse {
    totalCount: number;
    pageInfo: QueryPageInfo;
    edges: ShellIdEdge[];
}

export interface ShellIdsQueryTopLevel {
    shells: ShellIdsQueryResponse;
}

// Query variables
export interface GetShellIdsQueryVariables {
    where?: Record<string, unknown>;
    first?: number;
    last?: number;
    after?: Cursor;
    before?: Cursor;
    orderBy?: OrderByField[];
}

export interface GetShellDetailQueryVariables {
    id: string;
}
