import { useEffect, useState } from "react";
import { useQuery } from "@apollo/client";
import moment from "moment";
import { GET_SHELL, GET_BEACON_STATUS, GET_PORTAL_STATUS } from "../graphql";

export const useShellStatus = (shellId: string | undefined) => {
    const [portalId, setPortalId] = useState<number | null>(null);
    const [timeUntilCallback, setTimeUntilCallback] = useState<string>("");
    const [isMissedCallback, setIsMissedCallback] = useState(false);

    // Fetch shell details first
    const { loading, error, data } = useQuery(GET_SHELL, {
        variables: { id: shellId },
        skip: !shellId,
        fetchPolicy: "network-only"
    });

    // Fetch beacon status
    const beaconId = data?.node?.beacon?.id;
    const { data: beaconData } = useQuery(GET_BEACON_STATUS, {
        variables: { id: beaconId },
        skip: !beaconId,
        pollInterval: 2000
    });

    // Fetch portal status
    const { data: portalData } = useQuery(GET_PORTAL_STATUS, {
        variables: { id: portalId },
        skip: !portalId,
        pollInterval: 2000
    });

    // Initialize portalId from shell data
    useEffect(() => {
        if (data?.node?.portals) {
            const activePortal = data.node.portals.find((p: any) => !p.closedAt);
            if (activePortal) {
                setPortalId(parseInt(activePortal.id));
            }
        }
    }, [data]);

    // Check if portal closed
    useEffect(() => {
        if (portalData?.node?.closedAt) {
            setPortalId(null);
        }
    }, [portalData]);

    // Update callback timer
    useEffect(() => {
        const updateTimer = () => {
            // @ts-ignore
            if (!beaconData?.node?.nextSeenAt) return;
            // @ts-ignore
            const next = moment(beaconData.node.nextSeenAt);
            const now = moment();
            const diff = next.diff(now, 'seconds');

            if (diff > 0) {
                setTimeUntilCallback(`in ${diff} seconds`);
                setIsMissedCallback(false);
            } else {
                const missedSeconds = Math.abs(diff);
                setTimeUntilCallback(`expected ${missedSeconds} seconds ago`);
                setIsMissedCallback(true);
            }
        };

        updateTimer(); // Initial call
        const intervalId = setInterval(updateTimer, 1000);
        return () => clearInterval(intervalId);
    }, [beaconData]);

    return {
        loading,
        error,
        data,
        portalId,
        setPortalId,
        timeUntilCallback,
        isMissedCallback,
        beaconData,
    };
};
