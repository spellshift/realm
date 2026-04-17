import { gql } from "@apollo/client";

export const GET_NOTIFICATIONS = gql`
    query GetNotifications($first: Int) {
        me {
            id
            notifications(first: $first, orderBy: { field: CREATED_AT, direction: DESC }) {
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
                        createdAt
                        priority
                        read
                        archived
                        event {
                            id
                            kind
                            timestamp
                            host {
                                id
                                name
                            }
                            beacon {
                                id
                                name
                                host {
                                    id
                                }
                            }
                            quest {
                                id
                                name
                            }
                        }
                    }
                }
            }
        }
    }
`;

export const MARK_NOTIFICATIONS_AS_READ = gql`
    mutation MarkNotificationsAsRead($notificationIDs: [ID!]!) {
        markNotificationsAsRead(notificationIDs: $notificationIDs) {
            id
            read
        }
    }
`;
