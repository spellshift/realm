import { useQuery, NetworkStatus } from "@apollo/client";
import { useMemo} from "react";
import { GET_TOME_IDS_QUERY } from "./queries";
import {
    TomeIdsQueryTopLevel,
    GetTomeIdsQueryVariables,
    UseTomesResult,
} from "./types";
import { FilterBarOption } from "../../../utils/interfacesUI";
import { getTomeFilterNameByTypes } from "../../../utils/utils";

export function useTomeIds(tomeFields: FilterBarOption[], tomeMultiSearch: string): UseTomesResult {

    const queryVariables = useMemo(
        () => buildTomeIdsQuery(tomeFields, tomeMultiSearch),
        [tomeFields, tomeMultiSearch]
    );

    const { data, previousData, error, networkStatus, refetch } =
        useQuery<TomeIdsQueryTopLevel>(GET_TOME_IDS_QUERY, {
            variables: queryVariables,
            notifyOnNetworkStatusChange: true,
            fetchPolicy: "cache-and-network",
        });

    const currentData = data ?? previousData;

    const tomeIds = useMemo(() => {
        return currentData?.tomes?.edges?.map((edge) => edge?.node?.id) || [];
    }, [currentData]);


    return {
        tomeIds,
        initialLoading: networkStatus === NetworkStatus.loading && !currentData,
        error,
        refetch,
    };
}

function buildTomeIdsQuery(tomeFields: FilterBarOption[], tomeMultiSearch: string): GetTomeIdsQueryVariables {
    const conditions: Record<string, unknown>[] = [];

    if (tomeFields.length > 0) {
        const { Tactic, SupportModel } = getTomeFilterNameByTypes(tomeFields);
        if (Tactic.length > 0) {
            conditions.push({ tacticIn: Tactic });
        }
        if (SupportModel.length > 0) {
            conditions.push({ supportModelIn: SupportModel });
        }
    }

    if (tomeMultiSearch) {
        conditions.push({
            or: [
                { paramDefsContains: tomeMultiSearch },
                { nameContains: tomeMultiSearch },
                { descriptionContains: tomeMultiSearch },
            ],
        });
    }

    if (conditions.length === 0) {
        return {};
    }

    if (conditions.length === 1) {
        return { where: conditions[0] };
    }

    return { where: { and: conditions } };
}
