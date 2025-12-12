import React, { createContext, useState } from "react";

// If this expands we will want to change how we store state/pass state
// In the future we could store preferences for light/dark theme

type UserPreferencesType = {
    sidebarMinimized: boolean
    setSidebarMinimized: React.Dispatch<React.SetStateAction<boolean>>
}

const defaultValue = { sidebarMinimized: false } as UserPreferencesType;

export const UserPreferencesContext = createContext(defaultValue);

export const UserPreferencesContextProvider = ({ children }: { children: React.ReactNode }) => {
    const [sidebarMinimized, setSidebarMinimized] = useState(false);

    return (
        <UserPreferencesContext.Provider value={{ sidebarMinimized, setSidebarMinimized }}>
            {children}
        </UserPreferencesContext.Provider>
    );
};
