import { useCallback, useMemo } from "react";
import { useQuery, useLazyQuery } from "@apollo/client";
import { sub } from "date-fns";
import { useHost } from "../../../context/HostContext";
import { GET_TOME_IDS_QUERY } from "../../../utils/queries";
import { GET_ONLINE_HOST_BEACONS_QUERY } from "./queries";
import { getPriotizedBeaconId, safelyJsonParse } from "../../../utils/utils";
import { CreateQuestInitialData } from "../../../context/CreateQuestModalContext";
import { FieldInputParams } from "../../../utils/interfacesUI";
import {
    TomeIdNode,
    TomeIdsQueryTopLevel,
    BeaconIdNode,
    BeaconIdEdge,
} from "../../../utils/interfacesQuery";
import { Filters } from "../../../context/FilterContext";

interface BeaconQueryResponse {
    beacons: {
        edges: BeaconIdEdge[];
    };
}

function buildParams(paramDefs: string | null): FieldInputParams[] {
    const { params = [] } = safelyJsonParse(paramDefs || "");
    return params.map((param: FieldInputParams) => ({ ...param, value: "" }));
}

function buildFormData(
    hostName: string,
    tome: TomeIdNode,
    beacons: BeaconIdNode[],
    initialFilters?: Partial<Filters>
): CreateQuestInitialData {
    const beaconId = getPriotizedBeaconId(beacons);

    return {
        name: `Populate ${hostName} Process list`,
        tomeId: tome.id,
        params: buildParams(tome.paramDefs),
        beacons: beaconId ? [beaconId] : [],
        initialStep: 1,
        initialFilters: initialFilters
    };
}

export const useFilesQuestFormData = () => {
    const { data: host } = useHost();

    const { data: tomeData } = useQuery<TomeIdsQueryTopLevel>(GET_TOME_IDS_QUERY, {
        variables: { where: { nameContainsFold: "Report file" } },
    });

    const [fetchBeacons, { loading }] = useLazyQuery<BeaconQueryResponse>(
        GET_ONLINE_HOST_BEACONS_QUERY,
        { fetchPolicy: "network-only" }
    );

    const initialFilters = useMemo(() => {
        if (!host) return undefined;
        return {
            beaconFields: [{
                id: host.id,
                name: host.name,
                value: host.id,
                label: host.name,
                kind: "host",
            }],
            tomeMultiSearch: "Report file"
        };
    }, [host]);

    const fetchFormData = useCallback(async (): Promise<CreateQuestInitialData | undefined> => {
        const tome = tomeData?.tomes?.edges?.[0]?.node;
        if (!host?.id || !tome) return undefined;

        const onlineThreshold = sub(new Date(), { seconds: 30 }).toISOString();

        const { data: beaconData } = await fetchBeacons({
            variables: {
                where: {
                    hasHostWith: { id: host.id },
                    nextSeenAtGTE: onlineThreshold,
                },
            },
        });

        const beacons = beaconData?.beacons?.edges?.map((e) => e.node) || [];
        return buildFormData(host.name, tome, beacons, initialFilters);
    }, [host?.id, tomeData, fetchBeacons]);

    return { fetchFormData, loading };
};
