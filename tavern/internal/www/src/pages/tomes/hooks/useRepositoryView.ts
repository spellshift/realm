import { useQuery } from "@apollo/client";
import { useEffect, useState } from "react";
import {
    TomeQueryTopLevel,
    RepositoryQueryTopLevel,
    GetTomesQueryVariables,
    GetRepositoryQueryVariables,
    TomeNode,
} from "../../../utils/interfacesQuery";
import { GET_REPOSITORY_QUERY, GET_TOMES_QUERY } from "../../../utils/queries";
import { RepositoryRow } from "../../../utils/interfacesUI";

export const useRepositoryView = () => {
    const [firstParty, setFirstParty] = useState<RepositoryRow | null>(null);
    const [repositories, setRepositories] = useState<Array<RepositoryRow>>([]);

    const { loading: firstPartyTomeLoading, data: firstPartyTome, error: firstPartyTomeError } = useQuery<TomeQueryTopLevel, GetTomesQueryVariables>(GET_TOMES_QUERY, {
        variables:
        {
            "where": {
              "supportModel": "FIRST_PARTY"
            },
        }
    });

    const {loading, data, error} = useQuery<RepositoryQueryTopLevel, GetRepositoryQueryVariables>(GET_REPOSITORY_QUERY, {
        variables:
        {
            "orderBy": [{
                "direction": "DESC",
                "field": "LAST_MODIFIED_AT"
            }]
        }
    });

    useEffect(()=> {
        if(!firstParty && firstPartyTome?.tomes?.edges && firstPartyTome.tomes.edges.length > 0){
            // Extract tome nodes from edges
            const tomeNodes: TomeNode[] = firstPartyTome.tomes.edges.map(edge => edge.node);

            const firstPartyRepo: RepositoryRow = {
                node: {
                    url: "https://github.com/spellshift/realm/tree/main/tavern/tomes",
                    repoType: "FIRST_PARTY",
                    tomes: tomeNodes
                }
            };
            setFirstParty(firstPartyRepo);
        }
    },[firstPartyTome, firstParty]);

    useEffect(()=>{
        const repos = [] as Array<RepositoryRow>;
        if(firstParty){
            repos.push(firstParty);
        }
        if(data?.repositories?.edges && data.repositories.edges.length > 0){
            const repoRows = data.repositories.edges.map(edge => ({
                node: {
                    ...edge.node,
                    tomes: edge.node.tomes.edges.map(tomeEdge => tomeEdge.node)
                }
            }));
            repos.push(...repoRows);
        }
        setRepositories(repos);
    },[data, firstParty]);

    return {
        loading: firstPartyTomeLoading || loading,
        repositories: repositories,
        error: firstPartyTomeError || error
    };
}
