import { Cursor, AssetEdge, QueryPageInfo } from "../../utils/interfacesQuery";

export interface GetAssetIdsQueryVariables {
    where?: any;
    first?: number;
    last?: number;
    after?: Cursor;
    before?: Cursor;
    orderBy?: any[];
}

export interface AssetIdsQueryTopLevel {
    assets: {
        totalCount: number;
        pageInfo: QueryPageInfo;
        edges: Array<{ node: { id: string } }>;
    };
}

export interface AssetDetailQueryResponse {
    assets: {
        totalCount: number;
        pageInfo: QueryPageInfo;
        edges: AssetEdge[];
    };
}
