import React, { createContext } from "react";
import { ApolloError, gql, useQuery } from "@apollo/client";
import { TagContextType } from "../utils/consts";

const defaultValue = {data: undefined, isLoading: false, error: undefined} as {data: undefined | TagContextType, isLoading: boolean, error: ApolloError | undefined};

export const TagContext = createContext(defaultValue);
  
export const TagContextProvider = ({children}: {children: React.ReactNode}) => {

    const GET_TAG_FILTERS = gql`
        query GetSearchFilters($groupTag: TagWhereInput, $serviceTag: TagWhereInput){
            groupTags:tags(where: $groupTag) {
                id
                name
                kind   
            },
            serviceTags:tags(where: $serviceTag) {
                id
                name
                kind   
            },
            beacons {
            id
            name
            principal
            host{
                name
                primaryIP
                tags {
                    id
                    kind
                    name
                }  
            }
        }
    }
    `;
    const PARAMS = {
        variables: { 
            groupTag: { kind: "group" },
            serviceTag: { kind: "service" },
        }
    }
    const { loading: isLoading, error: error, data: data } = useQuery(GET_TAG_FILTERS, PARAMS);

  
    return (
      <TagContext.Provider value={{ data, isLoading, error }}>
        {children}
      </TagContext.Provider>
    );
};