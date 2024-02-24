import { useQuery } from "@apollo/client";
import { useEffect, useState } from "react";
import { RepositoryRow } from "../../../utils/consts";
import { GET_REPOSITORY_QUERY, GET_TOMES_QUERY } from "../../../utils/queries";

export const useRepositoryView = () => {
    const [repositories, setRepositories] = useState<Array<RepositoryRow>>([]);
    const { loading: firstPartyTomeLoading, data: firstPartyTome, error: firstPartyTomeError } = useQuery(GET_TOMES_QUERY, {
        variables:
        {
            "where": {
              "supportModel": "FIRST_PARTY"
            }
        }
    });

    const {loading, data, error} = useQuery(GET_REPOSITORY_QUERY);

    useEffect(()=> {
        if(firstPartyTome?.tomes){
            setRepositories((prevState) =>
                [{node:{
                    url: "https://github.com/spellshift/realm/tree/main/tavern/tomes",
                    repoType: "FIRST_PARTY",
                    tomes: firstPartyTome?.tomes
            }},...prevState]);
        }
    },[firstPartyTome]);

    useEffect(()=>{
        if(data?.repositories?.edges && data.repositories.edges.length > 0){
            setRepositories((prevState) =>[
                ...prevState,
                ...data?.repositories?.edges
            ]);
        }
    },[data]);

    return {
        loading: firstPartyTomeLoading || loading,
        repositories,
        error: firstPartyTomeError || error
    };
}
