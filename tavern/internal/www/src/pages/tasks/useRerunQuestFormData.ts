import { useCallback } from "react";
import { useLazyQuery } from "@apollo/client";
import { checkIfBeaconOffline, constructTomeParams } from "../../utils/utils";
import { CreateQuestInitialData } from "../../context/CreateQuestModalContext";
import { QuestNode } from "../../utils/interfacesQuery";
import { GET_BEACONS_BY_IDS_QUERY } from "./queries";

interface BeaconWithOnlineStatus {
    id: string;
    lastSeenAt: string;
    interval: number;
}

interface BeaconsQueryResponse {
    beacons: {
        edges: Array<{ node: BeaconWithOnlineStatus }>;
    };
}

function getUniqueBeaconIds(quest: QuestNode): string[] {
    const beaconIds = new Set<string>();
    quest.tasks?.edges?.forEach((taskEdge) => {
        const beaconId = taskEdge?.node?.beacon?.id;
        if (beaconId) {
            beaconIds.add(beaconId);
        }
    });
    return Array.from(beaconIds);
}

function filterOnlineBeacons(beacons: BeaconWithOnlineStatus[]): string[] {
    return beacons
        .filter((beacon) => !checkIfBeaconOffline(beacon))
        .map((beacon) => beacon.id);
}

export function useRerunQuestFormData(quest: QuestNode | undefined) {
    const [fetchBeacons, { loading }] = useLazyQuery<BeaconsQueryResponse>(
        GET_BEACONS_BY_IDS_QUERY,
        { fetchPolicy: "network-only" }
    );

    const fetchOnlineBeacons = useCallback(async (): Promise<string[]> => {
        if (!quest) return [];

        const beaconIds = getUniqueBeaconIds(quest);
        if (beaconIds.length === 0) return [];

        const { data } = await fetchBeacons({
            variables: {
                where: { idIn: beaconIds },
            },
        });

        const beacons = data?.beacons?.edges?.map((e) => e.node) || [];
        return filterOnlineBeacons(beacons);
    }, [quest, fetchBeacons]);

    const buildFormDataWithSameTome = useCallback(
        async (): Promise<CreateQuestInitialData | undefined> => {
            if (!quest) return undefined;

            const onlineBeaconIds = await fetchOnlineBeacons();
            const params = constructTomeParams(quest.parameters, quest.tome?.paramDefs);

            return {
                name: quest.name,
                tomeId: quest.tome?.id,
                params,
                beacons: onlineBeaconIds,
            };
        },
        [quest, fetchOnlineBeacons]
    );

    const buildFormDataWithNewTome = useCallback(
        async (): Promise<CreateQuestInitialData | undefined> => {
            if (!quest) return undefined;

            const onlineBeaconIds = await fetchOnlineBeacons();

            return {
                name: quest.name,
                beacons: onlineBeaconIds,
                initialStep: 1,
            };
        },
        [quest, fetchOnlineBeacons]
    );

    return {
        buildFormDataWithSameTome,
        buildFormDataWithNewTome,
        loading,
    };
}
