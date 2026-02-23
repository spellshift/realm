import React, { createContext, useContext, useEffect, useState } from 'react'
import { OrderDirection } from '../../utils/enums'
import { OrderByField } from '../../utils/interfacesQuery'
import { defaultSorts, sortablePageNavItems, Sorts } from './sortingUtils'

const STORAGE_KEY = 'realm-sorting-v1.0'

function isValidOrderByField(item: any): item is OrderByField {
    return (
        typeof item === 'object' &&
        item !== null &&
        'direction' in item &&
        'field' in item &&
        typeof item.direction === 'string' &&
        typeof item.field === 'string' &&
        Object.values(OrderDirection).includes(item.direction)
    )
}

function validateStoredSorts(data: any): Sorts {
    if (!data || typeof data !== 'object') {
        return defaultSorts
    }

    for (const key of sortablePageNavItems) {
        if (!(key in data) || !isValidOrderByField(data[key])) {
            return defaultSorts
        }
    }

    return data as Sorts
}

function loadSortsFromStorage(): Sorts {
    if (typeof window === 'undefined') {
        return defaultSorts
    }

    const stored = sessionStorage.getItem(STORAGE_KEY)
    if (!stored) {
        return defaultSorts
    }

    try {
        return validateStoredSorts(JSON.parse(stored))
    } catch {
        return defaultSorts
    }
}

type SortsContextType = {
    sorts: Sorts
    updateSorts: (updates: Partial<Sorts>) => void
    resetSorts: () => void
}

const SortsContext = createContext<SortsContextType | undefined>(undefined)

export function SortsProvider({ children }: { children: React.ReactNode }) {

    const [sorts, setSorts] = useState<Sorts>(loadSortsFromStorage);

    const updateSorts = (updates: Partial<Sorts>) => {
        setSorts(prevSorts => ({
            ...prevSorts,
            ...updates
        }))
    }

    const resetSorts = () => {
        setSorts(defaultSorts)
    };

    useEffect(() => {
        if (typeof window !== 'undefined') {
            sessionStorage.setItem(STORAGE_KEY, JSON.stringify(sorts))
        }
    }, [sorts]); // Don't collapse these useEffects, this helps fix race condition

    useEffect(() => {
        const handleStorage = (event: StorageEvent) => {
            if (event.key === STORAGE_KEY && event.newValue) {
                try {
                    setSorts(validateStoredSorts(JSON.parse(event.newValue)))
                } catch {
                    setSorts(defaultSorts)
                }
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
