import { useQuery } from "@apollo/client";
import { useMemo } from "react";
import { OrderDirection, RepositoryOrderField } from "../../../utils/enums";
import { GET_REPOSITORY_IDS_QUERY, GET_FIRST_PARTY_TOMES_QUERY } from "../queries";
import {
    RepositoryIdsQueryTopLevel,
    GetRepositoryIdsQueryVariables,
    FirstPartyTomesQueryResponse,
    FIRST_PARTY_REPO_ID,
} from "../types";

export const useRepositoryIds = () => {
    // Fetch first party tomes to know if we need the first party row
    const {
        data: firstPartyData,
        loading: firstPartyLoading,
        error: firstPartyError,
    } = useQuery<FirstPartyTomesQueryResponse>(GET_FIRST_PARTY_TOMES_QUERY);

    // Fetch repository IDs
    const queryVariables: GetRepositoryIdsQueryVariables = useMemo(() => ({
        orderBy: [{
            direction: OrderDirection.Desc,
            field: RepositoryOrderField.LastModifiedAt,
        }],
    }), []);

    const {
        data: repositoriesData,
        loading: repositoriesLoading,
        error: repositoriesError,
        refetch,
    } = useQuery<RepositoryIdsQueryTopLevel>(GET_REPOSITORY_IDS_QUERY, {
        variables: queryVariables,
        fetchPolicy: 'cache-and-network',
    });

    // Build the combined list of IDs
    const repositoryIds = useMemo(() => {
        const ids: string[] = [];

        // Add first party repo ID if there are first party tomes
        const hasFirstPartyTomes = firstPartyData?.tomes?.edges && firstPartyData.tomes.edges.length > 0;
        if (hasFirstPartyTomes) {
            ids.push(FIRST_PARTY_REPO_ID);
        }

        // Add regular repository IDs
        if (repositoriesData?.repositories?.edges) {
            const repoIds = repositoriesData.repositories.edges.map(edge => edge.node.id);
            ids.push(...repoIds);
        }

        return ids;
    }, [firstPartyData, repositoriesData]);

    return {
        repositoryIds,
        loading: firstPartyLoading || repositoriesLoading,
        error: firstPartyError || repositoriesError,
        refetch,
    };
};
