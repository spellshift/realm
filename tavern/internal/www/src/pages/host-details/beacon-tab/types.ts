import {
    QueryPageInfo,
    OrderByField,
    Cursor,
    BeaconEdge
} from "../../../utils/interfacesQuery";

export interface BeaconIdNode {
    id: string;
}

export interface BeaconIdEdge {
    node: BeaconIdNode;
}

export interface BeaconIdsQueryResponse {
    totalCount: number;
    pageInfo: QueryPageInfo;
    edges: BeaconIdEdge[];
}

export interface BeaconIdsQueryTopLevel {
    beacons: BeaconIdsQueryResponse;
}

// Beacon detail response (reuses BeaconNode from shared types)
export interface BeaconDetailQueryResponse {
    beacons: {
        totalCount: number;
        pageInfo: QueryPageInfo;
        edges: BeaconEdge[];
    };
}

// Query variables
export interface GetBeaconIdsQueryVariables {
    where?: Record<string, unknown>;
    first?: number;
    last?: number;
    after?: Cursor;
    before?: Cursor;
    orderBy?: OrderByField[];
}

export interface GetBeaconDetailQueryVariables {
    id: string;
}
