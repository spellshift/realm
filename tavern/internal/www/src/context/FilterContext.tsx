import React, { createContext, useContext, useEffect, useState } from 'react'
import { FilterBarOption } from '../utils/interfacesUI'

const STORAGE_KEY = 'realm-filters-v1.1'

export type Filters = {
    filtersEnabled: boolean,
    questName: string,
    taskOutput: string,
    beaconFields: Array<FilterBarOption>,
    tomeFields: Array<FilterBarOption>,
    tomeMultiSearch: string,
}

const defaultFilters: Filters = {
    filtersEnabled: true,
    questName: "",
    taskOutput: "",
    beaconFields: [],
    tomeFields: [],
    tomeMultiSearch: ""
}

type FilterContextType = {
    filters: Filters
    updateFilters: (updates: Partial<Filters>) => void
    resetFilters: () => void
}

const FilterContext = createContext<FilterContextType | undefined>(undefined)

export function FilterProvider({ children }: { children: React.ReactNode }) {

    const [filters, setFilters] = useState<Filters>(() => {
        if (typeof window !== 'undefined') {
            const stored = sessionStorage.getItem(STORAGE_KEY)
            return stored ? JSON.parse(stored) : defaultFilters
        }
        return defaultFilters
    });

    const updateFilters = (updates: Partial<Filters>) => {
        setFilters(prevFilters => ({
            ...prevFilters,
            ...updates
        }))
    }

    const resetFilters = () => {
        setFilters(defaultFilters)
        sessionStorage.removeItem(STORAGE_KEY)
    };

    useEffect(() => {
        if (typeof window !== 'undefined') {
            sessionStorage.setItem(STORAGE_KEY, JSON.stringify(filters))
        }
    }, [filters]); // Don't collapse these useEffects, this helps fix race condition

    useEffect(() => {
        const handleStorage = (event: StorageEvent) => {
            if (event.key === STORAGE_KEY && event.newValue) {
                setFilters(JSON.parse(event.newValue))
            }
        }
        window.addEventListener('storage', handleStorage)
        return () => window.removeEventListener('storage', handleStorage)
    }, [])

    return (
        <FilterContext.Provider value={{ filters, updateFilters, resetFilters }}>
            {children}
        </FilterContext.Provider>
    )
}

export function useFilters() {
    const context = useContext(FilterContext)
    if (!context) {
        throw new Error('useFilters must be used within a FilterProvider')
    }
    return context
}
