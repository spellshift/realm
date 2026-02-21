import { gql, useMutation } from "@apollo/client";
import { ApolloError } from "@apollo/client/errors";
import { Steps, useToast } from "@chakra-ui/react";
import { useState } from "react";
import { GET_REPOSITORY_DETAIL_QUERY } from "../queries";

export const useFetchRepositoryTome = (handleOnSuccess?:()=>void, showToast?: boolean) => {
    const [error, setError] = useState("");
    const toast = useToast();

    const IMPORT_REPOSITORY_TOMES_MUTATION = gql`
        mutation ImportRepositoryTomes($repoId: ID!){
            importRepository(repoID: $repoId){
                id
                createdAt
                lastModifiedAt
                url
                publicKey
                owner{
                    id
                    name
                    photoURL
                }
            }
        }
    `;

    const handleError = (error: ApolloError) => {
        if(error){
            setError(error?.message);
        }
        if(error && showToast){
            toast({
                title: 'Error importing tomes',
                description: error?.message,
                status: 'error',
                duration: 6000,
                isClosable: true,
            });
        }
    }
    const handleSuccess = () => {
        if(showToast){
            toast({
                title: 'Successfully imported tomes',
                status: 'success',
                duration: 3000,
                isClosable: true,
            });
        }
        if(handleOnSuccess){
            handleOnSuccess();
        }
    }

    const [createRepositoryLinkMutation, {loading}] = useMutation(IMPORT_REPOSITORY_TOMES_MUTATION, {onCompleted: handleSuccess, onError: handleError,
        refetchQueries: [
            GET_REPOSITORY_DETAIL_QUERY,
            'GetRepositoryDetail'
        ]
    });

    const importRepositoryTomes = (repoId: string) => {
        setError("");
        const variables = {
            "variables": {
                "repoId": repoId
            }
        };
        createRepositoryLinkMutation(variables);
    };

    return {
        importRepositoryTomes,
        loading,
        error,
    }
};
