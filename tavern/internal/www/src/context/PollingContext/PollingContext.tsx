import { useApolloClient } from "@apollo/client";
import { createContext, useContext, useEffect, useState } from "react";

const PollingContext = createContext<{ secondsUntilNextPoll: number } | undefined>(undefined);

export const PollingProvider: React.FC<{ children: React.ReactNode }> = ({ children }) => {
    const apolloClient = useApolloClient();
    const [secondsUntilNextPoll, setSecondsUntilNextPoll] = useState(30);

    useEffect(() => {
        setSecondsUntilNextPoll(30);

        const pollInterval = setInterval(() => {
            apolloClient.refetchQueries({
                include: "active",
            });
            setSecondsUntilNextPoll(30);
        }, 30000);

        const countdownTimer = setInterval(() => {
            setSecondsUntilNextPoll((prev) => {
                if (prev <= 1) {
                    return 30; // Safety fallback, shouldn't happen if synced
                }
                return prev - 1;
            });
        }, 1000);

        return () => {
            clearInterval(pollInterval);
            clearInterval(countdownTimer);
        };
    }, [apolloClient]);

    return (
        <PollingContext.Provider value={{ secondsUntilNextPoll }}>
            {children}
        </PollingContext.Provider>
    );
};

export const usePolling = () => {
    const context = useContext(PollingContext);
    if (context === undefined) {
        throw new Error('usePolling must be used within a PollingContextProvider');
    }
    return context;
};
