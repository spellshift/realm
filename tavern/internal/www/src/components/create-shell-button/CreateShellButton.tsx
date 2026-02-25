import { useState, useEffect } from "react";
import { useQuery, useMutation } from "@apollo/client";
import { useNavigate } from "react-router-dom";
import { useToast } from "@chakra-ui/react";
import { Terminal } from "lucide-react";
import { GET_BEACONS_FOR_HOST_QUERY, CREATE_SHELL_MUTATION } from "./queries";
import Button from "../tavern-base-ui/button/Button";
import { isBeaconActive } from "../../utils/utils";

interface CreateShellButtonProps {
    hostId?: string;
    beaconId?: string;
}

export const CreateShellButton = ({ hostId, beaconId }: CreateShellButtonProps) => {
    const navigate = useNavigate();
    const toast = useToast();
    const [targetBeaconId, setTargetBeaconId] = useState<string | null>(null);

    // If hostId is provided, we fetch beacons
    const { data: beaconsData, loading: queryLoading, error: queryError } = useQuery(GET_BEACONS_FOR_HOST_QUERY, {
        variables: { hostId },
        skip: !hostId || !!beaconId, // skip if no hostId or if beaconId is already provided
        fetchPolicy: "network-only"
    });

    const [createShell, { loading: mutationLoading }] = useMutation(CREATE_SHELL_MUTATION, {
        onCompleted: (data) => {
            navigate(`/shellv2/${data.createShell.id}`);
        },
        onError: (error) => {
            toast({
                title: "Failed to create shell",
                description: error.message,
                status: "error",
                duration: 5000,
                isClosable: true,
            });
        }
    });

    // Effect to handle query errors
    useEffect(() => {
        if (queryError) {
            toast({
                title: "Failed to fetch beacons",
                description: queryError.message,
                status: "error",
                duration: 5000,
                isClosable: true,
            });
        }
    }, [queryError, toast]);

    // Effect to select best beacon when data arrives
    useEffect(() => {
        if (beaconId) {
            setTargetBeaconId(beaconId);
            return;
        }

        if (beaconsData?.beacons?.edges) {
            const beacons = beaconsData.beacons.edges.map((e: any) => e.node);

            // 1. Filter active beacons (missed checkin <= 10s)
            const activeBeacons = beacons.filter((b: any) => isBeaconActive(b, 10));

            if (activeBeacons.length === 0) {
                setTargetBeaconId(null);
                return;
            }

            // 2. Prioritize Principal (root, administrator, system)
            // 3. Prioritize Shortest Interval
            // 4. Prioritize Expected to be seen next

            const highPrivPrincipals = ["root", "administrator", "system"];

            const sortedBeacons = activeBeacons.sort((a: any, b: any) => {
                // Priority 1: Principal
                const aPriv = highPrivPrincipals.includes(a.principal?.toLowerCase());
                const bPriv = highPrivPrincipals.includes(b.principal?.toLowerCase());
                if (aPriv && !bPriv) return -1;
                if (!aPriv && bPriv) return 1;

                // Priority 2: Interval (ASC)
                if (a.interval !== b.interval) {
                    return (a.interval || 0) - (b.interval || 0);
                }

                // Priority 3: Next checkin (Soonest)
                // Use nextSeenAt if available, else lastSeenAt + interval
                const getNextSeen = (beacon: any) => {
                    if (beacon.nextSeenAt) return new Date(beacon.nextSeenAt).getTime();
                    return new Date(beacon.lastSeenAt).getTime() + ((beacon.interval || 0) * 1000);
                };

                return getNextSeen(a) - getNextSeen(b);
            });

            setTargetBeaconId(sortedBeacons[0].id);
        }
    }, [beaconsData, beaconId]);

    const handleClick = () => {
        if (targetBeaconId) {
            createShell({ variables: { input: { beaconID: targetBeaconId } } });
        }
    };

    if (queryLoading) {
        return (
             <Button
                isLoading
                disabled
                buttonVariant="ghost"
                buttonStyle={{ size: 'xs', color: 'gray' }}
                data-testid="create-shell-button-loading"
                leftIcon={<Terminal className="w-4 h-4" />}
                aria-label="New Shell"
            />
        );
    }

    if (queryError) {
        return null;
    }

    if (!targetBeaconId) return null;

    return (
        <Button
            onClick={(e) => {
                e.stopPropagation(); // Prevent row click propagation
                handleClick();
            }}
            isLoading={mutationLoading}
            buttonVariant="ghost"
            buttonStyle={{ size: 'xs', color: 'gray' }}
            data-testid="create-shell-button"
            leftIcon={<Terminal className="w-4 h-4" />}
            aria-label="New Shell"
        />
    );
};
