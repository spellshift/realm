import { gql } from "@apollo/client";

export const GET_HOST_SCREENSHOTS = gql`
    query GetHostScreenshots(
        $hostId: ID!
        $first: Int!
        $after: Cursor
        $orderBy: [ScreenshotOrder!]
    ) {
        hosts(where: { id: $hostId }) {
            edges {
                node {
                    screenshots(first: $first, after: $after, orderBy: $orderBy) {
                        totalCount
                        pageInfo { endCursor hasNextPage }
                        edges {
                            node {
                                id
                                name
                                size
                                createdAt
                                lastModifiedAt
                                task { id quest { name } }
                                shellTask { id input }
                            }
                        }
                    }
                }
            }
        }
    }
`;
