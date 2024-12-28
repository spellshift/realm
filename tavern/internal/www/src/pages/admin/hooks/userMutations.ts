import { gql, useMutation } from "@apollo/client"
import { GraphQLErrors, NetworkError } from "@apollo/client/errors";
import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { UpdateUserProps } from "../../../utils/consts";
import { GET_USER_QUERY } from "../../../utils/queries";

export const useUpdateUser = () => {
    const [error, setError] = useState(false);
    const navigate = useNavigate();

    const ACTIVATE_USER_MUTATION = gql`
        mutation ActivateUser ($id: ID!, $input: UpdateUserInput!) {
            updateUser(userID: $id, input: $input) {
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
        navigate('/admin');
    }

    const [activateUserMutation, {loading, reset}] = useMutation(ACTIVATE_USER_MUTATION, {onCompleted: handleOnCompleted, onError: handleError, refetchQueries: [
        GET_USER_QUERY, // DocumentNode object parsed with gql
        'GetUserQuery' // Query name
    ]});

    const submitUpdateUser = (props: UpdateUserProps) => {
        const formatVariables = {
            "variables": {
                "id": props.id,
                "input": {
                    "isActivated": props.activated,
                    "isAdmin": props.admin,
                }
            }
        };
        activateUserMutation(formatVariables);
    }

    return {
        submitUpdateUser,
        loading,
        error,
        reset
    }
}
