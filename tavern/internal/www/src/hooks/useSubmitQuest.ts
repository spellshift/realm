import { gql, useMutation } from "@apollo/client"
import { GraphQLErrors, NetworkError } from "@apollo/client/errors";
import { GraphQLError } from "graphql";
import { useEffect, useState } from "react";
import { useNavigate } from "react-router-dom";
import { QuestProps, Tome } from "../utils/consts";
import { GET_JOB_QUERY } from "../utils/queries";

export const useSubmitQuest = () => {
    const [error, setError] = useState(false);
    const navigate = useNavigate();
    
    const CREATE_JOB_MUTATION = gql`
        mutation CreateQuest ($IDs: [ID!]!, $input: CreateQuestInput!) {
            createQuest(beaconIDs: $IDs, input: $input) {
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
        navigate("/quests");
    }

    const [createQuestMutation, {loading, reset}] = useMutation(CREATE_JOB_MUTATION, {onCompleted: handleOnCompleted, onError: handleError, refetchQueries: [
        GET_JOB_QUERY, // DocumentNode object parsed with gql
        'GetQuests' // Query name
      ]});

    const submitQuest = (props: QuestProps) => {
        const formatVariables = {
            "variables": {
                "IDs": props.beacons,
                "input": {
                    "name": props?.name, 
                    "tomeID": props.tome?.id
                }
            }
        };
        createQuestMutation(formatVariables);
    }

    return {
        submitQuest,
        loading,
        error,
        reset
    }
}