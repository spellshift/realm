import { useQuery } from "@apollo/client";
import { TomeNode } from "../../../utils/interfacesQuery";
import { GET_TOME_DETAIL_QUERY } from "../tome-selection/queries";
import { TomeDetailQueryResponse } from "../tome-selection/types";

interface UseTomeByIdResult {
    tome: TomeNode | null;
    loading: boolean;
    error: Error | undefined;
}

export const useTomeById = (tomeId: string | null | undefined): UseTomeByIdResult => {
    const { data, loading, error } = useQuery<TomeDetailQueryResponse>(
        GET_TOME_DETAIL_QUERY,
        {
            variables: { id: tomeId },
            skip: !tomeId,
        }
    );

    const tome = data?.tomes?.edges?.[0]?.node || null;

    return {
        tome,
        loading,
        error: error as Error | undefined,
    };
};
