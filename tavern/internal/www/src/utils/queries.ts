import { gql } from "@apollo/client";

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