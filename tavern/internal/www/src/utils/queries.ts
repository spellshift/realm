import { gql, } from "@apollo/client";

export const GET_HOST_QUERY = gql`
    query GetHosts($where: HostWhereInput) {
        hosts(where: $where){
            id,
            name,
            primaryIP,
            platform,
            lastSeenAt,
            tags{
                name,
                id,
                kind
            }
            beacons{
                id,
                name,
                interval,
                lastSeenAt
            }
        }
}`;

export const GET_QUEST_QUERY = gql`
    query GetQuests($where: QuestWhereInput) {
        quests(where: $where){
            id
            name
            tasks{
                id
                lastModifiedAt
                output
                execStartedAt
                execFinishedAt
                createdAt
                beacon {
                    id
                    name
					host{
                      id
                      name
                      primaryIP
                      tags {
                        id
                        name
                        kind
                      }
                    }
                }
            }
            tome{
                id
                name
                paramDefs
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
`;

export const GET_TASK_QUERY = gql`
    query GetTasks($where: TaskWhereInput, $first: Int, $last:Int, $after: Cursor, $before:Cursor, $orderBy: [TaskOrder!]) {
            tasks(where: $where, first: $first, last: $last, after: $after, before:$before, orderBy: $orderBy){
                pageInfo{
                    hasNextPage,
                    hasPreviousPage,
                    startCursor,
                    endCursor
                },
        	    totalCount,
                edges{
                    node{
                        id
                        lastModifiedAt
                        output
                        execStartedAt
                        execFinishedAt
                        createdAt
                        claimedAt
                        error
                        quest{
                            name
                            creator{
                                id
                                name
                            }
                            tome{
                                name
                                description
                            }
                            parameters
                        }
                        beacon {
                            id
                            name
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
        },
        hosts{
            label:name
            value:id
        }
    }
`;
