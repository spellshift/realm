import { gql, useMutation } from "@apollo/client";
import { GraphQLErrors, NetworkError } from "@apollo/client/errors";
import { useState } from "react";
import { GET_QUEST_IDS_QUERY } from "../../pages/quests/queries";
import { ModalQuestFormValues } from "./types";

export type CreateModalQuestProps = ModalQuestFormValues;

interface CreateQuestMutationData {
    createQuest: {
        id: string;
    };
}

interface CreateQuestMutationVariables {
    IDs: string[];
    input: {
        name: string;
        tomeID: string;
        parameters: string;
    };
}

export const useModalSubmitQuest = () => {
    const [error, setError] = useState(false);

    const CREATE_QUEST_MUTATION = gql`
        mutation CreateQuest($IDs: [ID!]!, $input: CreateQuestInput!) {
            createQuest(beaconIDs: $IDs, input: $input) {
                id
            }
        }
    `;

    const handleError = (error: NetworkError | GraphQLErrors) => {
        if (error) {
            setError(true);
        }
    };

    const [createQuestMutation, { loading, reset }] = useMutation<CreateQuestMutationData, CreateQuestMutationVariables>(CREATE_QUEST_MUTATION, {
        onError: handleError,
        refetchQueries: [GET_QUEST_IDS_QUERY],
        awaitRefetchQueries: true,
    });

    const submitQuest = async (props: CreateModalQuestProps) => {
        const param_obj = props.params.reduce(
            (acc, param) => {
                acc[param.name] = param.value;
                return acc;
            },
            {} as Record<string, any>
        );

        const formatVariables = {
            variables: {
                IDs: props.beacons,
                input: {
                    name: props.name,
                    tomeID: props.tomeId!,
                    parameters: JSON.stringify(param_obj),
                },
            },
        };
        return createQuestMutation(formatVariables);
    };

    return {
        submitQuest,
        loading,
        error,
        reset,
    };
};
