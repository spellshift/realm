import React, { createContext, useContext } from "react";
import { ApolloError, gql, useQuery } from "@apollo/client";
import { UserNode } from "../utils/interfacesQuery";

export type AuthorizationContextType = {
    me: UserNode;
}

export type AuthorizationContextQueryType = {
    data: AuthorizationContextType | undefined;
    isLoading: boolean;
    error: ApolloError | undefined;
}

export const AuthorizationContext = createContext<AuthorizationContextQueryType | undefined>(undefined);

export const AuthorizationContextProvider = ({ children }: { children: React.ReactNode }) => {

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

    const { loading: isLoading, error, data } = useQuery(GET_USER_INFO);

    return (
        <AuthorizationContext.Provider value={{ data, isLoading, error }}>
            {children}
        </AuthorizationContext.Provider>
    );
};

export const useAuthorization = () => {
    const context = useContext(AuthorizationContext);
    if (context === undefined) {
        throw new Error('useAuthorization must be used within an AuthorizationContextProvider');
    }
    return context;
};
