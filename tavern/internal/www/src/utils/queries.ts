import { gql } from "@apollo/client";

export const GET_HOST_QUERY = gql`
    query GetHosts($where: HostWhereInput) {
        hosts(where: $where){
            id
            name
            primaryIP
            platform
            lastSeenAt
            tags{
                name
                id
                kind
            }
            beacons{
                id
                name
                principal
                interval
                lastSeenAt
            }
        }
}`;

export const GET_TOMES_QUERY = gql`
    query GetTomes($where: TomeWhereInput) {
        tomes(where: $where){
        id
        name
        paramDefs
        tactic
        eldritch
        supportModel
        description
        uploader{
            id
            name
            photoURL
        }
    }
}`;

export const GET_REPOSITORY_QUERY = gql`
    query GetRepository($orderBy: [RepositoryOrder!]){
        repositories(orderBy: $orderBy){
            edges{
                node{
                    id
                    lastModifiedAt
                    url
                    publicKey
                    tomes{
                        id
                        name
                        paramDefs
                        tactic
                        eldritch
                        supportModel
                    }
                    owner{
                        id
                        name
                        photoURL
                    }
                }
            }
        }
}`;

export const GET_HOST_TASK_SUMMARY = gql`
    query GetTasks($where: TaskWhereInput) {
            tasks(where: $where){
        	    totalCount
                edges{
                    node{
                        execFinishedAt
                        outputSize
                    }
                }
        }
    }
`;

export const GET_QUEST_QUERY = gql`
    query GetQuests(
        $where: QuestWhereInput,
        $whereTotalTask: TaskWhereInput,
        $whereFinishedTask: TaskWhereInput,
        $whereOutputTask: TaskWhereInput,
        $whereErrorTask: TaskWhereInput,
        $firstTask: Int,
        $orderByTask: [TaskOrder!],
        $first: Int,
        $last:Int,
        $after: Cursor,
        $before:Cursor,
        $orderBy: [QuestOrder!]
    ) {
        quests(where: $where, first: $first, last: $last, after: $after, before:$before, orderBy: $orderBy){
            totalCount
            pageInfo{
                hasNextPage
                hasPreviousPage
                startCursor
                endCursor
            }
            edges{
                node{
                    id
                    name
                    parameters
                    lastUpdatedTask:tasks(first: $firstTask, orderBy: $orderByTask){
                        edges{
                            node{
                                lastModifiedAt
                            }
                        }
                    }
                    tasksTotal:tasks(where: $whereTotalTask){
                        totalCount
                    }
                    tasksOutput:tasks(where: $whereOutputTask){
                        totalCount
                    }
                    tasksError:tasks(where: $whereErrorTask){
                        totalCount
                    }
                    tasksFinished:tasks(where: $whereFinishedTask){
                        totalCount
                    }
                    tome{
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

export const GET_QUEST_BY_ID_QUERY = gql`
    query GetQuests(
        $where: QuestWhereInput,
        $whereTotalTask: TaskWhereInput,
        $whereFinishedTask: TaskWhereInput,
        $whereOutputTask: TaskWhereInput,
        $whereErrorTask: TaskWhereInput,
        $firstTask: Int,
        $orderByTask: [TaskOrder!],
        $first: Int,
        $last:Int,
        $after: Cursor,
        $before:Cursor,
        $orderBy: [QuestOrder!]
    ) {
        quests(where: $where, first: $first, last: $last, after: $after, before:$before, orderBy: $orderBy){
            totalCount
            pageInfo{
                hasNextPage
                hasPreviousPage
                startCursor
                endCursor
            }
            edges{
                node{
                    id
                    name
                    parameters
                    lastUpdatedTask:tasks(first: $firstTask, orderBy: $orderByTask){
                        edges{
                            node{
                                lastModifiedAt
                            }
                        }
                    }
                    tasksTotal:tasks(where: $whereTotalTask){
                        totalCount
                        edges{
                            node{
                                beacon{
                                    id
                                    lastSeenAt
                                    interval
                                }
                            }
                        }
                    }
                    tasksOutput:tasks(where: $whereOutputTask){
                        totalCount
                        edges{
                            node{
                                beacon{
                                    id
                                    lastSeenAt
                                    interval
                                }
                            }
                        }
                    }
                    tasksError:tasks(where: $whereErrorTask){
                        totalCount
                    }
                    tasksFinished:tasks(where: $whereFinishedTask){
                        totalCount
                    }
                    tome{
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

export const GET_TASK_QUERY = gql`
    query GetTasks($where: TaskWhereInput, $first: Int, $last:Int, $after: Cursor, $before:Cursor, $orderBy: [TaskOrder!]) {
            tasks(where: $where, first: $first, last: $last, after: $after, before:$before, orderBy: $orderBy){
                pageInfo{
                    hasNextPage
                    hasPreviousPage
                    startCursor
                    endCursor
                }
        	    totalCount
                edges{
                    node{
                        id
                        lastModifiedAt
                        outputSize
                        execStartedAt
                        execFinishedAt
                        createdAt
                        claimedAt
                        error
                        quest{
                            id
                            name
                            creator{
                                id
                                name
                                photoURL
                            }
                            tome{
                                id
                                name
                                description
                                eldritch
                                tactic
                                paramDefs
                                supportModel
                            }
                            parameters
                        }
                        beacon {
                            id
                            name
                            principal
                            lastSeenAt
                            interval
                                host{
                                id
                                name
                                primaryIP
                                platform
                                tags {
                                    id
                                    name
                                    kind
                                }
                            }
                        }
                    }
                }
        }
    }
`;

export const GET_TASK_OUTPUT_QUERY = gql`
    query GetOutputForTask($where: TaskWhereInput) {
            tasks(where: $where, ){
    				edges{
                        node{
                            id
                            output
                        }
            }
    }
}`

export const GET_SEARCH_FILTERS = gql`
    query GetSearchFilters($groupTag: TagWhereInput, $serviceTag: TagWhereInput){
        groupTags:tags(where: $groupTag) {
            label:name
            value:id
            id
            name
            kind
        },
        serviceTags:tags(where: $serviceTag) {
            label:name
            value:id
            id
            name
            kind
        },
        beacons{
            label:name
            value:id
            id
            name
        }
        hosts{
            label:name
            value:id
        }
    }
`;
