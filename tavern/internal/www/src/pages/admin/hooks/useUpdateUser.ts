import { gql, useMutation, ApolloError } from "@apollo/client"
import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { GET_USER_QUERY } from "../../../utils/queries";
import { GET_USER_IDS_QUERY, GET_USER_DETAIL_QUERY } from "../queries";
import { toaster } from "@/components/ui/toaster";

interface UpdateUserMutationResponse {
    updateUser: {
        id: string;
    };
}

interface UpdateUserMutationVariables {
    id: string;
    input: {
        isActivated: boolean;
        isAdmin: boolean;
    };
}

interface UpdateUserProps{
    id: number,
    activated: boolean,
    admin: boolean,
};

interface UseUpdateUserReturn {
    submitUpdateUser: (props: UpdateUserProps) => Promise<void>;
    loading: boolean;
    error: boolean;
    reset: () => void;
}

export const useUpdateUser = (): UseUpdateUserReturn => {
    const [error, setError] = useState<boolean>(false);
    const navigate = useNavigate();

    const ACTIVATE_USER_MUTATION = gql`
        mutation ActivateUser ($id: ID!, $input: UpdateUserInput!) {
            updateUser(userID: $id, input: $input) {
                id
            }
        }
    `;

    const handleError = (error: ApolloError) => {
        if(error){
            setError(true);
        }
    }

    const handleOnCompleted = () => {
        navigate('/admin');
    }

    const [activateUserMutation, {loading, reset}] = useMutation<UpdateUserMutationResponse, UpdateUserMutationVariables>(
        ACTIVATE_USER_MUTATION,
        {
            onCompleted: handleOnCompleted,
            onError: handleError,
            refetchQueries: [
                GET_USER_QUERY,
                GET_USER_IDS_QUERY,
                GET_USER_DETAIL_QUERY,
            ]
        }
    );

    const submitUpdateUser = async (props: UpdateUserProps): Promise<void> => {
        const formatVariables: { variables: UpdateUserMutationVariables } = {
            variables: {
                id: props.id.toString(),
                input: {
                    isActivated: props.activated,
                    isAdmin: props.admin,
                }
            }
        };
        const loadingToast = toaster.create({
            title: "Modifying user",
            type: "info", // "loading" is not a standard type in createToaster usually, checking usage
        });
        const { errors } = await activateUserMutation(formatVariables);
        toaster.dismiss(loadingToast);
        if(errors){
            toaster.create({
                title: "Error",
                description: "There was an error modifying the user",
                type: "error",
                duration: 3000,
                closable: true,
            });
            return;
        }
        toaster.create({
            title: "Success",
            description: "User was modified successfully",
            type: "success",
            duration: 3000,
            closable: true,
        });
    }

    return {
        submitUpdateUser,
        loading,
        error,
        reset
    }
}
