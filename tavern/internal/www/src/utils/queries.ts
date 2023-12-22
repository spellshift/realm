import { gql, } from "@apollo/client";


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
    query GetTasks($where: TaskWhereInput) {
            tasks(where: $where){
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
`;

const DEFAULTTAGFILTERS = { 
    "groupTag": { "kind": "group" },
    "serviceTag": { "kind": "service" }
};

export const GET_SEARCH_FILTERS = gql`
    query GetSearchFilters($groupTag: TagWhereInput, $serviceTag: TagWhereInput){
        groupTags:tags(where: $groupTag) {
            label:name
            value:id   
        },
        serviceTags:tags(where: $serviceTag) {
            label:name
            value:id   
        },
        beacons{
            label:name
            value:id 
        },
        hosts{
            label:name
            value:id 
        }
    }
`;