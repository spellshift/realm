import { gql, useMutation } from "@apollo/client";
import { GET_TOMES_QUERY } from "../../../utils/queries";

export const CREATE_TOME_MUTATION = gql`
    mutation CreateTome($input: CreateTomeInput!){
        createTome(input: $input){
            id
            name
            description
            author
            tactic
            supportModel
            paramDefs
            eldritch
            createdAt
        }
    }
`;

export const useCreateTome = () => {
    const [createTomeMutation, { loading, error }] = useMutation(CREATE_TOME_MUTATION, {
        refetchQueries: [
            GET_TOMES_QUERY,
            'GetTomes'
        ]
    });

    const createTome = async (input: any) => {
        return createTomeMutation({ variables: { input } });
    };

    return {
        createTome,
        loading,
        error
    };
};
