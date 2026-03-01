import { gql } from "@apollo/client";

export const GET_TOME_IDS_QUERY = gql`
    query GetTomeIds($where: TomeWhereInput) {
        tomes(where: $where) {
            edges {
                node {
                    id
                    name
                    paramDefs
                }
            }
        }
    }
`;

export const GET_TOME_DETAIL_QUERY = gql`
    query GetTomeDetail($id: ID!) {
        tomes(where: { id: $id }, first: 1) {
            edges {
                node {
                    id
                    name
                    paramDefs
                    tactic
                    eldritch
                    supportModel
                    description
                    uploader {
                        id
                        name
                        photoURL
                    }
                }
            }
        }
    }
`;
