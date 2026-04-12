import {
    QueryPageInfo,
    OrderByField,
    Cursor,
    UserNode
} from "../../../utils/interfacesQuery";

export interface PortalBeaconNode {
    id: string;
    name: string;
    lastSeenAt: string;
    interval: number;
}

export interface PortalNode {
    id: string;
    closedAt: string | null;
    beacon: PortalBeaconNode;
    owner: UserNode;
    activeUsers: {
        edges: { node: UserNode }[];
    };
}

export interface PortalEdge {
    node: PortalNode;
}

export interface PortalsQueryResponse {
    totalCount: number;
    pageInfo: QueryPageInfo;
    edges: PortalEdge[];
}

export interface PortalsQueryTopLevel {
    portals: PortalsQueryResponse;
}

// Portal IDs query (for pagination)
export interface PortalIdNode {
    id: string;
}

export interface PortalIdEdge {
    node: PortalIdNode;
}

export interface PortalIdsQueryResponse {
    totalCount: number;
    pageInfo: QueryPageInfo;
    edges: PortalIdEdge[];
}

export interface PortalIdsQueryTopLevel {
    portals: PortalIdsQueryResponse;
}

// Query variables
export interface GetPortalIdsQueryVariables {
    where?: Record<string, unknown>;
    first?: number;
    last?: number;
    after?: Cursor;
    before?: Cursor;
    orderBy?: OrderByField[];
}

export interface GetPortalDetailQueryVariables {
    id: string;
}
