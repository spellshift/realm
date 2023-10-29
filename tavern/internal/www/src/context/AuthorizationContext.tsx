import React, { createContext } from "react";
import { ApolloError, gql, useQuery } from "@apollo/client";
import { EmptyState, EmptyStateType } from "../components/tavern-base-ui/EmptyState";

export type UserType = {
    id: string;
    name: string;
    photoURL: string;
    isActivated: boolean;
    isAdmin: boolean;
}
export type AuthorizationContextType = {
    me: UserType;
}
export type AuthorizationContextQueryType = {
    data: undefined | AuthorizationContextType;
    isLoading: boolean;
    error: ApolloError | undefined;
}

const defaultValue = {data: undefined, isLoading: false, error: undefined} as AuthorizationContextQueryType;

export const AuthorizationContext = createContext(defaultValue);

export const AuthorizationContextProvider = ({children}: {children: React.ReactNode}) => {

    const GET_USER_INFO = gql`
        query GetMe{
            me {
                id
                name
                photoURL,
                isActivated,
                isAdmin
            }
        }
    `;

    const { loading: isLoading, error: error, data: data } = useQuery(GET_USER_INFO);

    function renderBasedOnState(
        data: undefined | AuthorizationContextType,
        isLoading: boolean,
        error: ApolloError | undefined
    ) : React.ReactNode {

        if(isLoading){
            return (
                <div className="flex flex-row w-sceen h-screen justify-center items-center">
                    <EmptyState label="Loading authroization state" type={EmptyStateType.loading}/>
                </div>
            );
        }

        if(error){
            return (
                <div className="flex flex-row w-sceen h-screen justify-center items-center">
                    <EmptyState label="Error fetching authroization state" type={EmptyStateType.error} details="Please contact your admin to diagnose the issue."/>
                </div>
            );
        }
        
        if(data?.me?.isActivated){
            return children;
        }

        return (
            <div className="flex flex-row w-sceen h-screen justify-center items-center">
                <EmptyState label="Account not approved" details={`Gain approval by providing your id (${data?.me?.id}) to an admin.`} type={EmptyStateType.noData}/>
            </div>
        );
    }
  
    return (
      <AuthorizationContext.Provider value={{ data, isLoading, error }}>
        {renderBasedOnState(data,isLoading, error)}
      </AuthorizationContext.Provider>
    );
};