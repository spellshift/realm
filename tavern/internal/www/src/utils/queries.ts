import { gql } from "@apollo/client";

export const GET_QUEST_QUERY = gql`
    query GetQuests {
        quests{
            id
            name
            tasks{
                id
                lastModifiedAt
                output
                execStartedAt
                execFinishedAt
            }
            tome{
                id
                name
                paramDefs
            }
        }
    }
`;