import { gql, useMutation } from "@apollo/client"
import { GraphQLErrors, NetworkError } from "@apollo/client/errors";
import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { GET_QUEST_QUERY } from "../../../utils/queries";
import { useFilters } from "../../../context/FilterContext";
import { QuestFormValues } from "../types";

export type CreateQuestProps = QuestFormValues;

export const useSubmitQuest = () => {
    const { updateFilters } = useFilters();
    const [error, setError] = useState(false);
    const navigate = useNavigate();

    const CREATE_QUEST_MUTATION = gql`
        mutation CreateQuest ($IDs: [ID!]!, $input: CreateQuestInput!) {
            createQuest(beaconIDs: $IDs, input: $input) {
                id
            }
        }
    `;

    const handleError = (error: NetworkError | GraphQLErrors) => {
        if(error){
            setError(true);
        }
    }

    const handleOnCompleted = (result: { createQuest: { id: string } }) => {
        updateFilters({'filtersEnabled': false});
        navigate(`/tasks/${result.createQuest.id}`);
    }

    const [createQuestMutation, {loading, reset}] = useMutation(CREATE_QUEST_MUTATION, {onCompleted: handleOnCompleted, onError: handleError, refetchQueries: [
        GET_QUEST_QUERY
      ]});

    const submitQuest = (props: CreateQuestProps) => {
        const param_obj = props.params.reduce((acc, param) => {
            acc[param.name] = param.value;
            return acc;
        }, {} as Record<string, any>);

        const formatVariables = {
            "variables": {
                "IDs": props.beacons,
                "input": {
                    "name": props.name,
                    "tomeID": props.tome!.id,
                    "parameters": JSON.stringify(param_obj),
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
