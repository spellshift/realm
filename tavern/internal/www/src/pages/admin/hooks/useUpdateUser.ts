import { gql, useMutation, ApolloError } from "@apollo/client"
import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { GET_USER_QUERY } from "../../../utils/queries";
import { GET_USER_IDS_QUERY, GET_USER_DETAIL_QUERY } from "../queries";
import { Steps, useToast } from "@chakra-ui/react";

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
    const toast = useToast();

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
        const loadingToast = toast({
            title: "Modifying user",
            status: "loading",
            position: "bottom-right",
        });
        const { errors } = await activateUserMutation(formatVariables);
        toast.close(loadingToast);
        if(errors){
            toast({
                title: "Error",
                description: "There was an error modifying the user",
                status: "error",
                duration: 3000,
                isClosable: true,
                position: "bottom-right",
            });
            return;
        }
        toast({
            title: "Success",
            description: "User was modified successfully",
            status: "success",
            duration: 3000,
            isClosable: true,
            position: "bottom-right",
        });
    }

    return {
        submitUpdateUser,
        loading,
        error,
        reset
    }
}
