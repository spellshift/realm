import { NetworkStatus, useQuery } from "@apollo/client";
import { useMemo, useState } from "react";
import { useSorts } from "../../../context/SortContext";
import { PageNavItem } from "../../../utils/enums";
import { GET_HOST_CREDENTIALS } from "../../../utils/queries";
import { groupBy } from "../../../utils/utils";
import { CredentialNode, HostCredentialsQueryTopLevel, OrderByField } from "../../../utils/interfacesQuery";

const toEpochMs = (isoTimestamp: string): number => {
    const parsed = new Date(isoTimestamp).getTime();
    return Number.isNaN(parsed) ? 0 : parsed;
};

export const groupCredentialsByPrincipalAndKind = (creds: CredentialNode[]): CredentialNode[] => {
    const principalGroups = Object.values(groupBy(creds, "principal"));
    return principalGroups
        .flatMap((principalGroup) => Object.values(groupBy(principalGroup, "kind")))
        .flatMap((group) => {
            if (!group || group.length === 0) {
                return [];
            }
            const mostRecent = group.reduce((a, b) =>
                toEpochMs(a.lastModifiedAt || a.createdAt) >= toEpochMs(b.lastModifiedAt || b.createdAt) ? a : b
            );
            const earliest = group.reduce((a, b) =>
                toEpochMs(a.createdAt) <= toEpochMs(b.createdAt) ? a : b
            );
            return [{
                ...mostRecent,
                createdAt: earliest.createdAt,
            }];
        });
};

const getHostCredentialsVariables = (
    hostId: number,
    search: string,
    sort?: OrderByField
) => {
    const trimmed = search.trim();
    return {
        where: { id: hostId },
        ...(trimmed && { credentialWhere: { principalContainsFold: trimmed } }),
        ...(sort && { orderBy: [sort] }),
    };
};

export const useCredsTable = (hostId: number) => {
    const { sorts } = useSorts();
    const credSort = sorts[PageNavItem.credentials];
    const [search, setSearch] = useState("");
    const [groupByPrincipal, setGroupByPrincipal] = useState(false);

    const queryVariables = useMemo(
        () => getHostCredentialsVariables(hostId, search, credSort),
        [hostId, search, credSort]
    );

    const { data, networkStatus, error } = useQuery<HostCredentialsQueryTopLevel>(
        GET_HOST_CREDENTIALS,
        {
            variables: queryVariables,
            fetchPolicy: "cache-and-network",
            pollInterval: 5000,
            skip: !hostId || Number.isNaN(hostId),
        }
    );

    const creds = useMemo(() => {
        const edges = data?.hosts?.edges?.[0]?.node?.credentials?.edges;
        if (!edges) {
            return [];
        }

        let result: CredentialNode[] = edges.map((credEdge) => {
            const cred = credEdge.node;
            return {
                ...cred,
                kind: cred.kind.replace(/^KIND_/, ""),
            };
        });

        if (search.trim()) {
            const normalizedSearch = search.trim().toLowerCase();
            result = result.filter((cred) => cred.principal.toLowerCase().includes(normalizedSearch));
        }

        if (groupByPrincipal) {
            result = groupCredentialsByPrincipalAndKind(result);
        }

        return result;
    }, [data, search, groupByPrincipal]);

    return {
        loading: networkStatus === NetworkStatus.loading && !data,
        creds,
        totalCount: data ? creds.length : undefined,
        error,
        search,
        setSearch,
        groupByPrincipal,
        setGroupByPrincipal,
    };
};
