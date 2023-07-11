import { gql, useMutation } from "@apollo/client"
import { JobProps, Tome } from "../utils/consts";

export const useSubmitJob = () => {
    
    const CREATE_JOB_MUTATION = gql`
        mutation CreateJob ($IDs: [ID!]!, $input: CreateJobInput!) {
            createJob(sessionIDs: $IDs, input: $input) {
                id
            }
        }
    `;

    const [createJobMutation, {data, loading, error, reset }] = useMutation(CREATE_JOB_MUTATION);

    const submitJob = (props: JobProps) => {
        console.log("Here in submit");
        const formatVariables = {
            "variables": {
                "IDs": props.sessions,
                "input": {
                    "name": props?.name, 
                    "tomeID": props.tome?.id
                }
            }
        };
        createJobMutation(formatVariables);
    }

    return {
        submitJob,
        data,
        loading,
        error,
        reset
    }
}