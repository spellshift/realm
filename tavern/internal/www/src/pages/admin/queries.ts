import { gql } from "@apollo/client";

export const GET_USER_IDS_QUERY = gql`
    query GetUserIds(
        $where: UserWhereInput,
        $first: Int,
        $last: Int,
        $after: Cursor,
        $before: Cursor
    ) {
        users(where: $where, first: $first, last: $last, after: $after, before: $before) {
            totalCount
            pageInfo {
                hasNextPage
                hasPreviousPage
                startCursor
                endCursor
            }
            edges {
                node {
                    id
                }
            }
        }
    }
`;

export const GET_USER_DETAIL_QUERY = gql`
    query GetUserDetail($id: ID!) {
        users(where: { id: $id }, first: 1) {
            totalCount
            pageInfo {
                hasNextPage
                hasPreviousPage
                startCursor
                endCursor
            }
            edges {
                node {
                    id
                    name
                    photoURL
                    isActivated
                    isAdmin
                }
            }
        }
    }
`;
