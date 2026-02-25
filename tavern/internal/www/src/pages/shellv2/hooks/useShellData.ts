import { useQuery } from "@apollo/client";
import { useEffect, useState } from "react";
import { GET_BEACON_STATUS, GET_PORTAL_STATUS, GET_SHELL } from "../graphql";

export const useShellData = (shellId: string | undefined) => {
    const [portalId, setPortalId] = useState<number | null>(null);

    // Fetch shell details first
    const { loading, error, data: shellData } = useQuery(GET_SHELL, {
        variables: { id: shellId },
        skip: !shellId,
        fetchPolicy: "network-only"
    });

    // Fetch beacon status
    const beaconId = shellData?.node?.beacon?.id;
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
        if (shellData?.node?.portals) {
            const activePortal = shellData.node.portals.find((p: any) => !p.closedAt);
            if (activePortal) {
                setPortalId(parseInt(activePortal.id));
            }
        }
    }, [shellData]);

    // Check if portal closed
    useEffect(() => {
        if (portalData?.node?.closedAt) {
            setPortalId(null);
        }
    }, [portalData]);

    return {
        loading,
        error,
        shellData,
        beaconData,
        portalData,
        portalId,
        setPortalId
    };
};
