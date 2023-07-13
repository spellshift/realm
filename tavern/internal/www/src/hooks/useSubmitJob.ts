import { gql, useMutation } from "@apollo/client"
import { GraphQLErrors, NetworkError } from "@apollo/client/errors";
import { GraphQLError } from "graphql";
import { useEffect, useState } from "react";
import { useNavigate } from "react-router-dom";
import { JobProps, Tome } from "../utils/consts";
import { GET_JOB_QUERY } from "../utils/queries";

export const useSubmitJob = () => {
    const [error, setError] = useState(false);
    const navigate = useNavigate();
    
    const CREATE_JOB_MUTATION = gql`
        mutation CreateJob ($IDs: [ID!]!, $input: CreateJobInput!) {
            createJob(sessionIDs: $IDs, input: $input) {
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

    const handleError = (error: NetworkError | GraphQLErrors) => {
        if(error){
            setError(true);
        }
    }

    const handleOnCompleted = () => {
        navigate("/jobs");
    }

    const [createJobMutation, {loading, reset}] = useMutation(CREATE_JOB_MUTATION, {onCompleted: handleOnCompleted, onError: handleError, refetchQueries: [
        GET_JOB_QUERY, // DocumentNode object parsed with gql
        'GetJobs' // Query name
      ]});

    const submitJob = (props: JobProps) => {
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
        loading,
        error,
        reset
    }
}