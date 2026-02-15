import { TomeNode, UserNode, RepositoryOrderBy } from "../../utils/interfacesQuery";

// Repository ID response (minimal data)
export interface RepositoryIdNode {
    id: string;
}

export interface RepositoryIdEdge {
    node: RepositoryIdNode;
}

export interface RepositoryIdsQueryResponse {
    edges: RepositoryIdEdge[];
}

export interface RepositoryIdsQueryTopLevel {
    repositories: RepositoryIdsQueryResponse;
}

// Repository detail response
export interface RepositoryDetailNode {
    id: string;
    lastModifiedAt: string;
    url: string;
    publicKey: string;
    tomes: {
        edges: Array<{ node: TomeNode }>;
    };
    owner: UserNode | null;
}

export interface RepositoryDetailEdge {
    node: RepositoryDetailNode;
}

export interface RepositoryDetailQueryResponse {
    repositories: {
        edges: RepositoryDetailEdge[];
    };
}

// First party tomes response
export interface FirstPartyTomesQueryResponse {
    tomes: {
        edges: Array<{ node: TomeNode }>;
    };
}

// Query variables
export interface GetRepositoryIdsQueryVariables {
    orderBy?: RepositoryOrderBy[];
}

export interface GetRepositoryDetailQueryVariables {
    id: string;
}

// First party repository synthetic data type
export interface FirstPartyRepositoryData {
    id: string;
    url: string;
    repoType: "FIRST_PARTY";
    lastModifiedAt: string;
    publicKey: string;
    tomes: TomeNode[];
    owner: null;
}

// Unified repository data for display
export interface RepositoryDisplayData {
    id: string;
    url: string;
    lastModifiedAt?: string;
    publicKey?: string;
    tomes: TomeNode[];
    owner: UserNode | null;
    isFirstParty: boolean;
}

// Special ID for first party repository
export const FIRST_PARTY_REPO_ID = "first-party";
