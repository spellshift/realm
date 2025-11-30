import { gql, useMutation } from "@apollo/client";
import { ApolloError } from "@apollo/client/errors";
import { useState } from "react";
import { RepositoryNode } from "../../../utils/interfacesQuery";
import { GET_REPOSITORY_QUERY } from "../../../utils/queries";

export const useCreateRepositoryLink = (setCurrStep: (arg: number)=>void, setNewRepository: (repository: RepositoryNode) => void) => {
    const [error, setError] = useState("");

    const CREATE_REPOSITORY_LINK_MUTATION = gql`
        mutation CreateRepositoryLink($input: CreateRepositoryInput!){
            createRepository(input: $input){
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
    }
    const handleOnCompleted = (result: any) => {
       setNewRepository(result?.createRepository);
       setCurrStep(1);
    }


    const [createRepositoryLinkMutation] = useMutation(CREATE_REPOSITORY_LINK_MUTATION, {onCompleted: handleOnCompleted, onError: handleError, refetchQueries: [
        GET_REPOSITORY_QUERY,
        'GetRepository'
    ]});

    const submitRepositoryLink = (props: {url: string}) => {
        setError("");
        const variables = {
            "variables": {
                "input": {
                    "url": props.url
                }
            }
        };
        createRepositoryLinkMutation(variables);
    };

    return {
        submitRepositoryLink,
        error,
    }
};
