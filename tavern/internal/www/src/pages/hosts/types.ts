import {
    QueryPageInfo,
    OrderByField,
    Cursor,
    HostEdge
} from "../../utils/interfacesQuery";

// Host ID response (minimal data)
export interface HostIdNode {
    id: string;
}

export interface HostIdEdge {
    node: HostIdNode;
}

export interface HostIdsQueryResponse {
    totalCount: number;
    pageInfo: QueryPageInfo;
    edges: HostIdEdge[];
}

export interface HostIdsQueryTopLevel {
    hosts: HostIdsQueryResponse;
}

// Host detail response (reuses HostNode from shared types)
export interface HostDetailQueryResponse {
    hosts: {
        totalCount: number;
        pageInfo: QueryPageInfo;
        edges: HostEdge[];
    };
}

// Query variables
export interface GetHostIdsQueryVariables {
    where?: Record<string, unknown>;
    first?: number;
    last?: number;
    after?: Cursor;
    before?: Cursor;
    orderBy?: OrderByField[];
}

export interface GetHostDetailQueryVariables {
    id: string;
}
