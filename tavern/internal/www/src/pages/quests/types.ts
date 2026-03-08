import { QueryPageInfo, TaskWhereInput, OrderByField, Cursor } from "../../utils/interfacesQuery";

// Quest ID response (minimal data)
export interface QuestIdNode {
    id: string;
}

export interface QuestIdEdge {
    node: QuestIdNode;
}

export interface QuestIdsQueryResponse {
    totalCount: number;
    pageInfo: QueryPageInfo;
    edges: QuestIdEdge[];
}

export interface QuestIdsQueryTopLevel {
    quests: QuestIdsQueryResponse;
}

// Extended types for quest detail
export interface QuestDetailTomeNode {
    id: string;
    name: string;
    description: string;
    eldritch: string;
    tactic: string;
    paramDefs: string;
    supportModel: string;
}

export interface QuestDetailUserNode {
    id: string;
    name: string;
    photoURL: string;
    isActivated: boolean;
    isAdmin: boolean;
}

export interface QuestDetailBeaconNode {
    id: string;
}

export interface QuestDetailTaskNode {
    id: string;
    beacon: QuestDetailBeaconNode;
}

// Quest detail response (full data for a single quest)
export interface QuestDetailNode {
    id: string;
    name: string;
    parameters: string | null;
    tome: QuestDetailTomeNode;
    creator: QuestDetailUserNode;
    lastUpdatedTask: {
        edges: Array<{
            node: {
                lastModifiedAt: string;
            };
        }>;
    };
    tasks: {
        edges: Array<{
            node: QuestDetailTaskNode;
        }>;
    };
    tasksTotal: {
        totalCount: number;
    };
    tasksFinished: {
        totalCount: number;
    };
    tasksOutput: {
        totalCount: number;
    };
    tasksError: {
        totalCount: number;
    };
}

export interface QuestDetailEdge {
    node: QuestDetailNode;
}

export interface QuestDetailQueryResponse {
    quests: {
        totalCount: number;
        pageInfo: QueryPageInfo;
        edges: QuestDetailEdge[];
    };
}

// Query variables
export interface GetQuestIdsQueryVariables {
    where?: Record<string, unknown>;
    first?: number;
    last?: number;
    after?: Cursor;
    before?: Cursor;
    orderBy?: OrderByField[];
}

export interface GetQuestDetailQueryVariables {
    id: string;
    whereTotalTask?: TaskWhereInput;
    whereFinishedTask?: TaskWhereInput;
    whereOutputTask?: TaskWhereInput;
    whereErrorTask?: TaskWhereInput;
    firstTask?: number;
    orderByTask?: OrderByField[];
}
