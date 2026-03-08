import { gql } from "@apollo/client";

// Query to fetch only quest IDs for pagination
export const GET_QUEST_IDS_QUERY = gql`
    query GetQuestIds(
        $where: QuestWhereInput,
        $first: Int,
        $last: Int,
        $after: Cursor,
        $before: Cursor,
        $orderBy: [QuestOrder!]
    ) {
        quests(where: $where, first: $first, last: $last, after: $after, before: $before, orderBy: $orderBy) {
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

// Query to fetch detailed data for a single quest
export const GET_QUEST_DETAIL_QUERY = gql`
    query GetQuestDetail(
        $id: ID!,
        $whereTotalTask: TaskWhereInput,
        $whereFinishedTask: TaskWhereInput,
        $whereOutputTask: TaskWhereInput,
        $whereErrorTask: TaskWhereInput,
        $firstTask: Int,
        $orderByTask: [TaskOrder!]
    ) {
        quests(where: { id: $id }, first: 1) {
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
                    parameters
                    lastUpdatedTask: tasks(first: $firstTask, orderBy: $orderByTask) {
                        edges {
                            node {
                                lastModifiedAt
                            }
                        }
                    }
                    tasks {
                        edges {
                            node {
                                id
                                beacon {
                                    id
                                }
                            }
                        }
                    }
                    tasksTotal: tasks(where: $whereTotalTask) {
                        totalCount
                    }
                    tasksOutput: tasks(where: $whereOutputTask) {
                        totalCount
                    }
                    tasksError: tasks(where: $whereErrorTask) {
                        totalCount
                    }
                    tasksFinished: tasks(where: $whereFinishedTask) {
                        totalCount
                    }
                    tome {
                        id
                        name
                        description
                        eldritch
                        tactic
                        paramDefs
                        supportModel
                    }
                    creator {
                        id
                        name
                        photoURL
                        isActivated
                        isAdmin
                    }
                }
            }
        }
    }
`;
