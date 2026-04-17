import { useCallback, useMemo } from "react";
import { ApolloError, gql, useMutation, useQuery } from "@apollo/client";
import { useToast, Tooltip } from "@chakra-ui/react";
import { Rss } from "lucide-react";
import Button from "../../../components/tavern-base-ui/button/Button";

const GET_SUBSCRIPTION_STATUS = gql`
    query GetHostSubscriptionStatus($hostId: ID!) {
        me {
            id
        }
        hosts(where: { id: $hostId }) {
            edges {
                node {
                    id
                    subscribers {
                        edges {
                            node {
                                id
                            }
                        }
                    }
                }
            }
        }
    }
`;

const SUBSCRIBE_TO_HOST = gql`
    mutation SubscribeToHost($hostId: ID!) {
        subscribeToHost(hostID: $hostId) {
            id
        }
    }
`;

const UNSUBSCRIBE_FROM_HOST = gql`
    mutation UnsubscribeFromHost($hostId: ID!) {
        unsubscribeFromHost(hostID: $hostId) {
            id
        }
    }
`;

interface HostSubscriptionButtonProps {
    hostId: string;
}

const HostSubscriptionButton = ({ hostId }: HostSubscriptionButtonProps) => {
    const toast = useToast();

    const handleMutationError = useCallback((title: string) => (error: ApolloError) => {
        toast({
            title,
            description: error.message,
            status: "error",
            duration: 5000,
            isClosable: true,
        });
    }, [toast]);

    const { data, loading: queryLoading } = useQuery(GET_SUBSCRIPTION_STATUS, {
        variables: { hostId },
        skip: !hostId,
    });

    const isSubscribed = useMemo(() => {
        const meId = data?.me?.id;
        const subscribers = data?.hosts?.edges?.[0]?.node?.subscribers?.edges;
        if (!meId || !subscribers) return false;
        return subscribers.some((edge: { node: { id: string } }) => edge.node.id === meId);
    }, [data]);

    const refetchQueries = [{ query: GET_SUBSCRIPTION_STATUS, variables: { hostId } }];

    const [subscribe, { loading: subscribeLoading }] = useMutation(SUBSCRIBE_TO_HOST, {
        variables: { hostId },
        refetchQueries,
        onError: handleMutationError("Error subscribing to host"),
    });

    const [unsubscribe, { loading: unsubscribeLoading }] = useMutation(UNSUBSCRIBE_FROM_HOST, {
        variables: { hostId },
        refetchQueries,
        onError: handleMutationError("Error unsubscribing from host"),
    });

    const mutationLoading = subscribeLoading || unsubscribeLoading;

    const handleClick = () => {
        if (isSubscribed) {
            unsubscribe();
        } else {
            subscribe();
        }
    };

    const subscribedStyle = "bg-green-600 text-white hover:bg-green-700";
    const unsubscribedStyle = "text-gray-900 ring-1 ring-gray-500 hover:bg-gray-100";

    return (
        <Tooltip label={isSubscribed ? "Unsubscribe from host" : "Subscribe to host"} bg="white" color="black" hasArrow>
            <span>
                <Button
                    buttonVariant="solid"
                    buttonStyle={{ size: "md" }}
                    className={isSubscribed ? subscribedStyle : unsubscribedStyle}
                    onClick={handleClick}
                    isLoading={mutationLoading}
                    disabled={queryLoading}
                    aria-label={isSubscribed ? "Unsubscribe from host" : "Subscribe to host"}
                >
                    <Rss className="w-5 h-5" />
                </Button>
            </span>
        </Tooltip>
    );
};

export default HostSubscriptionButton;
