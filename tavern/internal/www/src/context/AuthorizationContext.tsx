import React, { createContext } from "react";
import { gql, useQuery } from "@apollo/client";
import { UserType } from "../utils/consts";

export type AuthorizationContextType = {
    me: UserType;
}
export type AuthorizationContextQueryType = {
    data: undefined | AuthorizationContextType;
    isLoading: boolean;
    error: any;
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
  
    return (
      <AuthorizationContext.Provider value={{ data, isLoading, error }}>
        {children}
      </AuthorizationContext.Provider>
    );
};