import {
    QueryPageInfo,
    OrderByField,
    Cursor,
    ProcessEdge,
} from "../../../utils/interfacesQuery";

export interface ProcessIdNode {
    id: string;
}

export interface ProcessIdEdge {
    node: ProcessIdNode;
}

export interface ProcessConnection {
    totalCount: number;
    pageInfo: QueryPageInfo;
    edges: ProcessIdEdge[];
}

export interface HostWithProcesses {
    processes: ProcessConnection;
}

export interface HostEdgeWithProcesses {
    node: HostWithProcesses;
}

export interface ProcessIdsQueryResponse {
    hosts: {
        edges: HostEdgeWithProcesses[];
    };
}

// Process detail types
export interface ProcessDetailConnection {
    edges: ProcessEdge[];
}

export interface HostWithProcessDetail {
    processes: ProcessDetailConnection;
}

export interface HostEdgeWithProcessDetail {
    node: HostWithProcessDetail;
}

export interface ProcessDetailQueryResponse {
    hosts: {
        edges: HostEdgeWithProcessDetail[];
    };
}

// Query variables
export interface HostProcessWhereInput {
    or?: HostProcessWhereInput[];
    nameContainsFold?: string;
    pathContainsFold?: string;
}

export interface GetProcessIdsQueryVariables {
    hostId: string;
    first?: number;
    after?: Cursor;
    orderBy?: OrderByField[];
    where?: HostProcessWhereInput;
}

export interface GetProcessDetailQueryVariables {
    hostId: string;
    processId: string;
}
