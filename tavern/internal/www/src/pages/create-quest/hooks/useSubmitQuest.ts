import { gql, useMutation } from "@apollo/client"
import { GraphQLErrors, NetworkError } from "@apollo/client/errors";
import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { CreateQuestProps } from "../../../utils/consts";
import { GET_QUEST_QUERY } from "../../../utils/queries";
import { useFilters } from "../../../context/FilterContext";

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

    const handleOnCompleted = (result: any) => {
        updateFilters({'filtersEnabled': false});
        navigate(`/tasks/${result?.createQuest?.id}`);
    }

    const [createQuestMutation, {loading, reset}] = useMutation(CREATE_QUEST_MUTATION, {onCompleted: handleOnCompleted, onError: handleError, refetchQueries: [
        GET_QUEST_QUERY, // DocumentNode object parsed with gql
        'GetQuests' // Query name
      ]});

    const submitQuest = (props: CreateQuestProps) => {
        var param_obj = {}
        for (var param of props.params) {
            var tmp_param = {
                [param.name]: param.value
            }
            param_obj = {
                ...tmp_param,
                ...param_obj,
            }
        }
        const formatVariables = {
            "variables": {
                "IDs": props.beacons,
                "input": {
                    "name": props?.name,
                    "tomeID": props.tome?.id,
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
