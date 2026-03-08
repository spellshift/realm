import {
    QueryPageInfo,
    OrderByField,
    Cursor,
} from "../../../utils/interfacesQuery";

export interface FileNode {
    id: string;
    createdAt: string;
    lastModifiedAt: string;
    path: string;
    owner: string | null;
    group: string | null;
    permissions: string | null;
    size: number;
    hash: string | null;
}

export interface FileEdge {
    node: FileNode;
}

export interface FileIdNode {
    id: string;
}

export interface FileIdEdge {
    node: FileIdNode;
}

export interface FileConnection {
    totalCount: number;
    pageInfo: QueryPageInfo;
    edges: FileIdEdge[];
}

export interface HostWithFiles {
    files: FileConnection;
}

export interface HostEdgeWithFiles {
    node: HostWithFiles;
}

export interface FileIdsQueryResponse {
    hosts: {
        edges: HostEdgeWithFiles[];
    };
}

// File detail types
export interface FileDetailConnection {
    edges: FileEdge[];
}

export interface HostWithFileDetail {
    files: FileDetailConnection;
}

export interface HostEdgeWithFileDetail {
    node: HostWithFileDetail;
}

export interface FileDetailQueryResponse {
    hosts: {
        edges: HostEdgeWithFileDetail[];
    };
}

// Query variables
export interface HostFileWhereInput {
    or?: HostFileWhereInput[];
    pathContainsFold?: string;
    hashContainsFold?: string;
}

export interface GetFileIdsQueryVariables {
    hostId: string;
    first?: number;
    after?: Cursor;
    orderBy?: OrderByField[];
    where?: HostFileWhereInput;
}

export interface GetFileDetailQueryVariables {
    hostId: string;
    fileId: string;
}
