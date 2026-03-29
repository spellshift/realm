import { useCallback } from "react";
import { useMutation } from "@apollo/client";
import { useToast } from "@chakra-ui/react";
import { gql } from "@apollo/client";

const CREATE_SHELL_MUTATION = gql`
    mutation CreateShell($beaconId: ID!) {
        createShell(input: { beaconID: $beaconId }) {
            id
        }
    }
`;

export function useOpenShell(beaconId: string) {
    const toast = useToast();

    const [createShell, { loading }] = useMutation(CREATE_SHELL_MUTATION, {
        onCompleted: (data) => {
            const shellId = data.createShell.id;
            window.open(`/shellv2/${shellId}`, "_blank");
        },
        onError: (error) => {
            toast({
                title: "Error creating shell",
                description: error.message,
                status: "error",
                duration: 5000,
                isClosable: true,
            });
        },
    });

    const handleOpenShell = useCallback(() => {
        createShell({ variables: { beaconId } });
    }, [createShell, beaconId]);

    return { handleOpenShell, loading };
}
