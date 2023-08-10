import { gql } from "@apollo/client";

export const GET_JOB_QUERY = gql`
    query GetJobs($where: JobWhereInput) {
        jobs(where: $where){
            id
            name
            tasks{
                id
                lastModifiedAt
                output
                execStartedAt
                execFinishedAt
                createdAt
                session {
                    id
                    name
                    tags{
                        name
                        kind
                        id   
                    }
                }
            }
            tome{
                id
                name
                paramDefs
            }
        }
    }
`;