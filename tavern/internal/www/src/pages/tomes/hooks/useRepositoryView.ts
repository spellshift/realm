import { useQuery } from "@apollo/client";
import { useEffect, useState } from "react";
import { RepositoryRow } from "../../../utils/consts";
import { GET_REPOSITORY_QUERY, GET_TOMES_QUERY } from "../../../utils/queries";

export const useRepositoryView = () => {
    const [firstParty, setFirstParty] = useState<RepositoryRow | null>(null);
    const [repositories, setRepositories] = useState<Array<RepositoryRow>>([]);
    const { loading: firstPartyTomeLoading, data: firstPartyTome, error: firstPartyTomeError } = useQuery(GET_TOMES_QUERY, {
        variables:
        {
            "where": {
              "supportModel": "FIRST_PARTY"
            },
        }
    });

    const {loading, data, error} = useQuery(GET_REPOSITORY_QUERY, {
        variables:
        {
            "orderBy": [{
                "direction": "DESC",
                "field": "LAST_MODIFIED_AT"
            }]
        }
    });

    useEffect(()=> {
        if(!firstParty && firstPartyTome && firstPartyTome?.tomes?.length > 0){
            const firstPartyRepo =
            {node:{
                url: "https://github.com/spellshift/realm/tree/main/tavern/tomes",
                repoType: "FIRST_PARTY",
                tomes: firstPartyTome?.tomes
            }}
            setFirstParty(
                firstPartyRepo
            );
        }
    },[firstPartyTome, firstParty]);

    useEffect(()=>{
        const repos = [] as Array<RepositoryRow>;
        if(firstParty){
            repos.push(firstParty);
        }
        if(data?.repositories?.edges && data.repositories.edges.length > 0){
            repos.push(
                ...data?.repositories?.edges
            );
        }
        setRepositories(
            repos
        );
    },[data, firstParty]);

    return {
        loading: firstPartyTomeLoading || loading,
        repositories: repositories,
        error: firstPartyTomeError || error
    };
}
