import { gql, useMutation } from "@apollo/client";
import { ApolloError } from "@apollo/client/errors";
import { toaster } from "@/components/ui/toaster";
import { useState } from "react";
import { GET_REPOSITORY_DETAIL_QUERY } from "../queries";

export const useFetchRepositoryTome = (handleOnSuccess?:()=>void, showToast?: boolean) => {
    const [error, setError] = useState("");

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
            toaster.create({
                title: 'Error importing tomes',
                description: error?.message,
                type: 'error',
                duration: 6000,
                closable: true,
            });
        }
    }
    const handleSuccess = () => {
        if(showToast){
            toaster.create({
                title: 'Successfully imported tomes',
                type: 'success',
                duration: 3000,
                closable: true,
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
