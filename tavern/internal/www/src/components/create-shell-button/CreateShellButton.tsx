import React, { useMemo, useCallback } from 'react';
import { useMutation, useQuery } from '@apollo/client';
import { Tooltip, useToast } from '@chakra-ui/react';
import { Terminal } from 'lucide-react';
import { useNavigate } from 'react-router-dom';
import { add } from 'date-fns';

import Button from '../tavern-base-ui/button/Button';
import { checkIfBeaconOffline } from '../../utils/utils';
import { PrincipalAdminTypes } from '../../utils/enums';
import { CREATE_SHELL_MUTATION, GET_BEACONS_FOR_HOST_QUERY } from './queries';

interface CreateShellButtonProps {
    hostId?: string;
    beaconId?: string;
}

interface BeaconCandidate {
    id: string;
    principal: string;
    interval: number;
    lastSeenAt: string;
    nextSeenAt?: string;
}

export const CreateShellButton: React.FC<CreateShellButtonProps> = ({ hostId, beaconId }) => {
    const navigate = useNavigate();
    const toast = useToast();

    // If hostId is provided, fetch beacons
    const { data: beaconsData, loading: beaconsLoading } = useQuery(GET_BEACONS_FOR_HOST_QUERY, {
        variables: { hostId },
        skip: !hostId || !!beaconId,
        fetchPolicy: 'cache-and-network',
    });

    const [createShell, { loading: mutationLoading }] = useMutation(CREATE_SHELL_MUTATION, {
        onCompleted: (data) => {
            const shellId = data.createShell.id;
            navigate(`/shellv2/${shellId}`);
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

    const selectedBeaconId = useMemo(() => {
        if (beaconId) return beaconId;
        if (!hostId || !beaconsData?.beacons?.edges) return null;

        const beacons: BeaconCandidate[] = beaconsData.beacons.edges.map((edge: any) => edge.node);

        // Filter active beacons
        const activeBeacons = beacons.filter(beacon => !checkIfBeaconOffline({
            lastSeenAt: beacon.lastSeenAt,
            interval: beacon.interval
        }));

        if (activeBeacons.length === 0) return null;

        // Sort candidates
        // 1. Principal priority: root, administrator, system
        // 2. Shortest interval
        // 3. Soonest nextSeenAt (or lastSeenAt + interval)

        const highPriorityPrincipals = [
            PrincipalAdminTypes.root,
            PrincipalAdminTypes.Administrator,
            PrincipalAdminTypes.SYSTEM
        ] as string[];

        // Clone array before sorting to avoid mutating read-only objects from Apollo
        return [...activeBeacons].sort((a, b) => {
            // 1. Principal Priority
            const aIsHigh = highPriorityPrincipals.includes(a.principal);
            const bIsHigh = highPriorityPrincipals.includes(b.principal);

            if (aIsHigh && !bIsHigh) return -1;
            if (!aIsHigh && bIsHigh) return 1;

            // 2. Shortest Interval
            if (a.interval !== b.interval) {
                return a.interval - b.interval;
            }

            // 3. Soonest Next Check-in
            const aNext = a.nextSeenAt
                ? new Date(a.nextSeenAt).getTime()
                : add(new Date(a.lastSeenAt), { seconds: a.interval }).getTime();

            const bNext = b.nextSeenAt
                ? new Date(b.nextSeenAt).getTime()
                : add(new Date(b.lastSeenAt), { seconds: b.interval }).getTime();

            return aNext - bNext;
        })[0]?.id || null;

    }, [beaconId, hostId, beaconsData]);

    const handleCreateShell = useCallback((e: React.MouseEvent) => {
        e.stopPropagation();
        if (selectedBeaconId) {
            createShell({ variables: { beaconId: selectedBeaconId } });
        }
    }, [createShell, selectedBeaconId]);

    if (!selectedBeaconId) {
        if (hostId && beaconsLoading) {
             return (
                 <Button
                    buttonVariant="ghost"
                    buttonStyle={{ color: "gray", size: 'xs' }}
                    disabled
                    aria-label="Loading shell options"
                >
                    <Terminal className="w-4 h-4 animate-pulse" />
                </Button>
             );
        }
        return null;
    }

    return (
        <Tooltip label="Create Shell" bg="white" color="black" hasArrow>
            <span>
                <Button
                    buttonVariant="ghost"
                    buttonStyle={{ color: "gray", size: 'xs' }}
                    onClick={handleCreateShell}
                    isLoading={mutationLoading}
                    aria-label="Create Shell"
                >
                    <Terminal className="w-4 h-4" />
                </Button>
            </span>
        </Tooltip>
    );
};
