import { useCallback, useMemo } from "react";
import { useLazyQuery, useQuery } from "@apollo/client";
import { sub } from "date-fns";
import { useHost } from "../../../context/HostContext";
import { CreateQuestInitialData } from "../../../context/CreateQuestModalContext";
import { Filters } from "../../../context/FilterContext";
import { BeaconIdEdge, TomeIdsQueryTopLevel } from "../../../utils/interfacesQuery";
import { FieldInputParams } from "../../../utils/interfacesUI";
import { GET_TOME_IDS_QUERY } from "../../../utils/queries";
import { getPriotizedBeaconId, safelyJsonParse } from "../../../utils/utils";
import { GET_ONLINE_HOST_BEACONS_QUERY } from "../files-tab/queries";

const TOME_NAME = "Report screenshot";

interface BeaconQueryResponse {
    beacons: { edges: BeaconIdEdge[] };
}

export const useScreenshotQuestFormData = () => {
    const { data: host } = useHost();
    const { data: tomeData } = useQuery<TomeIdsQueryTopLevel>(GET_TOME_IDS_QUERY, {
        variables: { where: { name: TOME_NAME } },
    });
    const [fetchBeacons, { loading }] = useLazyQuery<BeaconQueryResponse>(
        GET_ONLINE_HOST_BEACONS_QUERY,
        { fetchPolicy: "network-only" }
    );

    const initialFilters = useMemo<Partial<Filters> | undefined>(() => {
        if (!host) return undefined;
        return {
            beaconFields: [{ id: host.id, name: host.name, value: host.id, label: host.name, kind: "host" }],
            tomeMultiSearch: TOME_NAME,
        };
    }, [host]);

    const fetchFormData = useCallback(async (): Promise<CreateQuestInitialData | undefined> => {
        const tome = tomeData?.tomes?.edges?.[0]?.node;
        if (!host?.id || !tome) return undefined;

        const { data: beaconData } = await fetchBeacons({
            variables: {
                where: {
                    hasHostWith: { id: host.id },
                    nextSeenAtGTE: sub(new Date(), { seconds: 30 }).toISOString(),
                },
            },
        });
        const beacons = beaconData?.beacons?.edges?.map(edge => edge.node) || [];
        const beaconId = getPriotizedBeaconId(beacons);
        const { params = [] } = safelyJsonParse(tome.paramDefs || "");

        return {
            name: `Populate ${host.name} ${TOME_NAME}`,
            tomeId: tome.id,
            params: params.map((param: FieldInputParams) => ({ ...param, value: "" })),
            beacons: beaconId ? [beaconId] : [],
            initialStep: 1,
            initialFilters,
        };
    }, [host?.id, host?.name, tomeData, fetchBeacons, initialFilters]);

    return { fetchFormData, loading };
};
