import { gql } from "@apollo/client";

export const GET_TAG_FILTERS = gql`

query GetSearchFilters($groupTag: TagWhereInput, $serviceTag: TagWhereInput){
        groupTags:tags(where: $groupTag) {
            edges{
              node{
                id
                name
            	kind
              }
            }
        },
        serviceTags:tags(where: $serviceTag) {
            edges{
              node{
                id
                name
            	kind
              }
            }
        },
        beacons {
          edges{
            node{
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
                    edges{
                      node{
                        id
                        kind
                        name
                      }
                    }
                  }
              }
            }
          }
        },
        hosts{
            edges{
                node{
                    id
                    name
                    primaryIP
                }
            }
        }
    }
`;

export const GET_HOST_QUERY = gql`
    query GetHosts(
        $where: HostWhereInput,
        $first: Int,
        $last: Int,
        $after: Cursor,
        $before:Cursor,
        $orderBy: [HostOrder!]
        ) {
        hosts(
            where: $where
            first: $first,
            last: $last,
            after: $after,
            before:$before,
            orderBy: $orderBy
        ){
            pageInfo {
                hasNextPage
                hasPreviousPage
                startCursor
                endCursor
            }
            totalCount
            edges {
            node {
                id
                name
                primaryIP
                platform
                lastSeenAt
                tags {
                    edges {
                        node {
                            name
                            id
                            kind
                        }
                    }
                }
                beacons {
                    edges {
                        node {
                            id
                            name
                            principal
                            interval
                            lastSeenAt
                        }
                    }
                }
                credentials {
                    edges {
                        node {
                            id
                        }
                    }
                }
            }
            }
        }
    }
`;

export const GET_TOMES_QUERY = gql`
    query GetTomes($where: TomeWhereInput) {
        tomes(where: $where){
            edges{
                node{
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
            }
        }
    }
`;

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
                      edges{
                        node{
                            id
                            name
                            paramDefs
                            tactic
                            eldritch
                            supportModel
                        }
                      }
                    }
                    owner{
                        id
                        name
                        photoURL
                    }
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
                    tasks{
                        edges{
                            node{
                                id
                                beacon {
                                    id
                                }
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

export const GET_HOST_TASK_COUNT = gql`
    query GetHostTaskCount($where: TaskWhereInput){
        tasks(where: $where){
            totalCount
        }
    }
`;

export const GET_TASK_QUERY = gql`
    query GetTasks($where: TaskWhereInput, $first: Int, $last: Int, $after: Cursor, $before: Cursor, $orderBy: [TaskOrder!]) {
        tasks(
            where: $where
            first: $first
            last: $last
            after: $after
            before: $before
            orderBy: $orderBy
        ) {
            pageInfo {
                hasNextPage
                hasPreviousPage
                startCursor
                endCursor
            }
            totalCount
            edges {
                node {
                    id
                    lastModifiedAt
                    outputSize
                    execStartedAt
                    execFinishedAt
                    createdAt
                    claimedAt
                    error
                    output
                    shells {
                        edges{
                            node{
                                id
                                closedAt
                                activeUsers {
                                    edges{
                                        node{
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
                    }
                    quest {
                        id
                        name
                        creator {
                            id
                            name
                            photoURL
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
                        parameters
                    }
                    beacon {
                        id
                        name
                        principal
                        lastSeenAt
                        interval
                        host {
                            id
                            name
                            primaryIP
                            platform
                            tags {
                                edges{
                                    node{
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
        }
    }
`;


export const GET_USER_QUERY = gql`
    query GetUserQuery($where: UserWhereInput){
        users(where: $where) {
            totalCount
            edges{
                node{
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

export const GET_HOST_CREDENTIALS = gql`
    query GetHostCredentials($where: HostWhereInput){
        hosts(where: $where) {
            edges{
                node{
                    credentials {
                      edges{
                        node{
                          createdAt
                          lastModifiedAt
                          principal
                          kind
                          secret
                        }
                      }
                    }
                }
            }
        }
}`;
