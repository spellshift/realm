import { gql, useMutation } from "@apollo/client"
import { GraphQLErrors, NetworkError } from "@apollo/client/errors";
import { GraphQLError } from "graphql";
import { useEffect, useState } from "react";
import { useNavigate } from "react-router-dom";
import { CreateQuestProps, Tome } from "../utils/consts";
import { GET_QUEST_QUERY } from "../utils/queries";

export const useSubmitQuest = () => {
    const [error, setError] = useState(false);
    const navigate = useNavigate();

    const CREATE_QUEST_MUTATION = gql`
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

    const handleOnCompleted = (result: any) => {
        navigate(`/quests/${result?.createQuest?.id}`);
    }

    const [createQuestMutation, {loading, reset}] = useMutation(CREATE_QUEST_MUTATION, {onCompleted: handleOnCompleted, onError: handleError, refetchQueries: [
        GET_QUEST_QUERY, // DocumentNode object parsed with gql
        'GetQuests' // Query name
      ]});

    const submitQuest = (props: CreateQuestProps) => {
        const param_array = []
        for (var param of props.params) {
            param_array.push({
                [param.name]: param.value
            })
        }
        console.log(param_array)
        const formatVariables = {
            "variables": {
                "IDs": props.beacons,
                "input": {
                    "name": props?.name,
                    "tomeID": props.tome?.id,
                    "parameter": props.params,
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