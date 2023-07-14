import { gql } from "@apollo/client";

export const GET_JOB_QUERY = gql`
    query GetJobs {
        jobs{
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