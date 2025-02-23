import { useQuery } from "@apollo/client";
import { useEffect, useState } from "react";
import { RepositoryRow } from "../../../utils/consts";
import { GET_ALL_TOMES_QUERY } from "../../../utils/queries";

export const useRepositoryView = () => {
    const [firstParty, setFirstParty] = useState<RepositoryRow | null>(null);
    const [noRepo, setNoRepo] = useState<RepositoryRow | null>(null);
    const [repositories, setRepositories] = useState<Array<RepositoryRow>>([]);

    const {loading, data, error} = useQuery(GET_ALL_TOMES_QUERY);

    useEffect(()=> {
        if(!firstParty && data && data.first_party && data.first_party?.length > 0){
            const firstPartyRepo =
            {node:{
                url: "https://github.com/spellshift/realm/tree/main/tavern/tomes",
                repoType: "FIRST_PARTY",
                tomes: data.first_party
            }}
            setFirstParty(
                firstPartyRepo
            );
        }
    },[data, firstParty]);

    useEffect(()=> {
        if(!noRepo && data && data.no_repo && data.no_repo?.length > 0){
            const noRepoData =
            {node:{
                url: "No Repository",
                repoType: "",
                tomes: data.no_repo
            }}
            setNoRepo(
                noRepoData
            );
        }
    },[data, noRepo]);

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
        if(noRepo){
            repos.push(noRepo);
        }
        setRepositories(
            repos
        );
    },[data, firstParty, noRepo]);

    return {
        loading,
        repositories,
        error
    };
}
