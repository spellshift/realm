import { useApolloClient } from "@apollo/client";
import { createContext, useContext, useEffect } from "react";

const PollingContext = createContext<{}>({});

export const PollingProvider: React.FC<{children: React.ReactNode}> = ({ children }) => {
    const apolloClient = useApolloClient();

    useEffect(() => {
        const interval = setInterval(() => {
            // Refetch all active queries (queries currently being watched by mounted components)
            apolloClient.refetchQueries({
                include: "active",
            });
        }, 60000);

        return () => clearInterval(interval);
    }, [apolloClient]);

    return (
        <PollingContext.Provider value={{}}>
            {children}
        </PollingContext.Provider>
    );
};

export const usePolling = () => useContext(PollingContext);
