import React, { createContext, useContext, useEffect, useState } from 'react'
import { FilterBarOption } from '../../utils/interfacesUI'

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

function isValidFilterBarOption(item: any): item is FilterBarOption {
    return (
        typeof item === 'object' &&
        item !== null &&
        typeof item.kind === 'string' &&
        typeof item.id === 'string' &&
        typeof item.name === 'string' &&
        (item.label === undefined || typeof item.label === 'string') &&
        (item.value === undefined || typeof item.value === 'string')
    )
}

function validateStoredFilters(data: any): Filters {
    if (!data || typeof data !== 'object') {
        return defaultFilters
    }

    const schema: Record<keyof Filters, (value: any) => boolean> = {
        filtersEnabled: (v) => typeof v === 'boolean',
        questName: (v) => typeof v === 'string',
        taskOutput: (v) => typeof v === 'string',
        tomeMultiSearch: (v) => typeof v === 'string',
        beaconFields: (v) => Array.isArray(v) && v.every(isValidFilterBarOption),
        tomeFields: (v) => Array.isArray(v) && v.every(isValidFilterBarOption),
    }

    for (const [key, validator] of Object.entries(schema)) {
        if (!(key in data) || !validator(data[key])) {
            return defaultFilters
        }
    }

    return data as Filters
}

function loadFiltersFromStorage(): Filters {
    if (typeof window === 'undefined') {
        return defaultFilters
    }

    const stored = sessionStorage.getItem(STORAGE_KEY)
    if (!stored) {
        return defaultFilters
    }

    try {
        return validateStoredFilters(JSON.parse(stored))
    } catch {
        return defaultFilters
    }
}

type FilterContextType = {
    filters: Filters
    updateFilters: (updates: Partial<Filters>) => void
    resetFilters: () => void
}

export const FilterContext = createContext<FilterContextType | undefined>(undefined)

export function FilterProvider({ children }: { children: React.ReactNode }) {

    const [filters, setFilters] = useState<Filters>(loadFiltersFromStorage);

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
                setFilters(validateStoredFilters(JSON.parse(event.newValue)))
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
