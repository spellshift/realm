import React, { createContext, useContext, useEffect, useState } from 'react'
import { HostOrderField, OrderDirection, PageNavItem, QuestOrderField, TaskOrderField } from '../utils/enums'
import { OrderByField } from '../utils/interfacesQuery'

const STORAGE_KEY = 'realm-sorting-v1.0'

export type Sorts = {
    [PageNavItem.hosts]: OrderByField,
    [PageNavItem.quests]: OrderByField,
    [PageNavItem.tasks]: OrderByField
}

const defaultSorts: Sorts = {
    [PageNavItem.hosts]: {
        direction: OrderDirection.Desc,
        field: HostOrderField.CreatedAt
    },
    [PageNavItem.quests]: {
        direction: OrderDirection.Desc,
        field: QuestOrderField.CreatedAt
    },
    [PageNavItem.tasks]: {
        direction: OrderDirection.Desc,
        field: TaskOrderField.LastModifiedAt
    }
}

type SortsContextType = {
    sorts: Sorts
    updateSorts: (updates: Partial<Sorts>) => void
    resetSorts: () => void
}

const SortsContext = createContext<SortsContextType | undefined>(undefined)

export function SortsProvider({ children }: { children: React.ReactNode }) {

    const [sorts, setSorts] = useState<Sorts>(() => {
        if (typeof window !== 'undefined') {
            const stored = sessionStorage.getItem(STORAGE_KEY)
            return stored ? JSON.parse(stored) : defaultSorts
        }
        return defaultSorts
    });

    const updateSorts = (updates: Partial<Sorts>) => {
        setSorts(prevSorts => ({
            ...prevSorts,
            ...updates
        }))
    }

    const resetSorts = () => {
        setSorts(defaultSorts)
        sessionStorage.removeItem(STORAGE_KEY)
    };

    useEffect(() => {
        if (typeof window !== 'undefined') {
            sessionStorage.setItem(STORAGE_KEY, JSON.stringify(sorts))
        }
    }, [sorts]); // Don't collapse these useEffects, this helps fix race condition

    useEffect(() => {
        const handleStorage = (event: StorageEvent) => {
            if (event.key === STORAGE_KEY && event.newValue) {
                setSorts(JSON.parse(event.newValue))
            }
        }
        window.addEventListener('storage', handleStorage)
        return () => window.removeEventListener('storage', handleStorage)
    }, [])

    return (
        <SortsContext.Provider value={{ sorts, updateSorts, resetSorts }}>
            {children}
        </SortsContext.Provider>
    )
}

export function useSorts() {
    const context = useContext(SortsContext)
    if (!context) {
        throw new Error('useSorts must be used within a SortProvider')
    }
    return context
}
