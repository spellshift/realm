import { Cursor, OrderByField } from "../../../utils/interfacesQuery";

export interface ScreenshotNode {
    id: string;
    name: string;
    size: number;
    createdAt: string;
    lastModifiedAt: string;
    task?: { id: string; quest: { name: string } } | null;
    shellTask?: { id: string; input: string } | null;
}

export interface ScreenshotEdge {
    node: ScreenshotNode;
}

export interface ScreenshotConnection {
    totalCount: number;
    pageInfo: { endCursor: Cursor; hasNextPage: boolean };
    edges: ScreenshotEdge[];
}

export interface HostScreenshotsQueryResponse {
    hosts: { edges: Array<{ node: { screenshots: ScreenshotConnection } }> };
}

export interface GetHostScreenshotsQueryVariables {
    hostId: string;
    first: number;
    after?: Cursor;
    orderBy: OrderByField[];
    where?: { nameContainsFold: string };
}
