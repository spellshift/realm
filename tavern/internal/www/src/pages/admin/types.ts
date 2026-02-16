import {
    QueryPageInfo,
    Cursor,
    UserEdge
} from "../../utils/interfacesQuery";

// User ID response (minimal data)
export interface UserIdNode {
    id: string;
}

export interface UserIdEdge {
    node: UserIdNode;
}

export interface UserIdsQueryResponse {
    totalCount: number;
    pageInfo: QueryPageInfo;
    edges: UserIdEdge[];
}

export interface UserIdsQueryTopLevel {
    users: UserIdsQueryResponse;
}

// User detail response (reuses UserNode from shared types)
export interface UserDetailQueryResponse {
    users: {
        totalCount: number;
        pageInfo: QueryPageInfo;
        edges: UserEdge[];
    };
}

// Query variables
export interface GetUserIdsQueryVariables {
    where?: Record<string, unknown>;
    first?: number;
    last?: number;
    after?: Cursor;
    before?: Cursor;
}

export interface GetUserDetailQueryVariables {
    id: string;
}
