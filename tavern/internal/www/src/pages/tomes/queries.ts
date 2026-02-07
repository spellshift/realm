import { gql } from "@apollo/client";

export const GET_REPOSITORY_IDS_QUERY = gql`
    query GetRepositoryIds($orderBy: [RepositoryOrder!]) {
        repositories(orderBy: $orderBy) {
            edges {
                node {
                    id
                }
            }
        }
    }
`;

export const GET_REPOSITORY_DETAIL_QUERY = gql`
    query GetRepositoryDetail($id: ID!) {
        repositories(where: { id: $id }, first: 1) {
            edges {
                node {
                    id
                    lastModifiedAt
                    url
                    publicKey
                    tomes {
                        edges {
                            node {
                                id
                                name
                                paramDefs
                                tactic
                                eldritch
                                supportModel
                                description
                            }
                        }
                    }
                    owner {
                        id
                        name
                        photoURL
                    }
                }
            }
        }
    }
`;

export const GET_FIRST_PARTY_TOMES_QUERY = gql`
    query GetFirstPartyTomes {
        tomes(where: { supportModel: FIRST_PARTY }) {
            edges {
                node {
                    id
                    name
                    paramDefs
                    tactic
                    eldritch
                    supportModel
                    description
                }
            }
        }
    }
`;
