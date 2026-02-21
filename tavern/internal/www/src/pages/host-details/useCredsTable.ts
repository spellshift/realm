import { useQuery } from "@apollo/client";
import { useCallback, useEffect, useState } from "react";
import { GET_HOST_CREDENTIALS } from "../../utils/queries";
import { groupBy } from "../../utils/utils";
import { CredentialNode, HostCredentialsQueryTopLevel } from "../../utils/interfacesQuery";

export const useCredsTable = (hostId: number) => {
    const [creds, setCreds] = useState([] as CredentialNode[]);
    const [search, setSearch] = useState("");
    const [groupByPrincipal, setGroupByPrincipal] = useState(false);

    const { loading, data, error } = useQuery<HostCredentialsQueryTopLevel>(GET_HOST_CREDENTIALS, {
        variables: {
            "where": {
                "id": hostId
                }
            }
    });

    const getCreds = useCallback((data: HostCredentialsQueryTopLevel, search: string, groupByPrincipal: boolean)=> {
        if(!data || data?.hosts?.edges?.length === 0) {
            return;
        }

        const hostNode = data.hosts.edges[0]?.node;

        if (!hostNode?.credentials?.edges) {
            return;
        }

        let creds: CredentialNode[] = hostNode.credentials.edges.map((credEdge) => {
            const cred = credEdge.node;
            return {
                ...cred,
                kind: cred.kind.replace(/^KIND_/, "")
            };
        });

        if (search) {
            creds = creds.filter((cred) => cred.principal.toLowerCase().includes(search.toLowerCase()));
        }

        if (groupByPrincipal) {
            let principal_groups = Object.values(groupBy(creds, "principal"));
            creds = principal_groups
                .flatMap(
                    (principalGroup) => Object.values(groupBy(principalGroup, "kind"))
                ).flatMap(
                    (group) => {
                        if (group == null) {
                            return [];
                        }
                        const mostRecent = group.reduce((a, b) => (a.createdAt > b.createdAt ? a : b));
                        const earliest = group.reduce((a, b) => (a.createdAt < b.createdAt ? a : b));
                        return [{
                            ...mostRecent,
                            createdAt: earliest.createdAt
                        }]
                    }
                );
        }

        setCreds(creds);
    }, []);

    useEffect(()=> {
        if(data){
            getCreds(data, search, groupByPrincipal);
        }
    },[data, getCreds, search, groupByPrincipal]);

    return {
        loading,
        creds,
        error,
        setSearch,
        groupByPrincipal,
        setGroupByPrincipal
    }
}
