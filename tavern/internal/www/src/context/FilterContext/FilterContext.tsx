import React, { createContext, useContext, useEffect, useMemo, useRef, useState } from 'react'
import { FilterBarOption } from '../../utils/interfacesUI'
import { useLocation } from 'react-router-dom'

export enum FilterFieldType {
    BEACON_FIELDS = 'beaconFields',
    TASK_OUTPUT = 'taskOutput',
    QUEST_NAME = 'questName',
    TOME_FIELDS = 'tomeFields',
    TOME_MULTI_SEARCH = "tomeMultiSearch"
}

const STORAGE_KEY = 'realm-filters-v1.1'

export type Filters = {
    isLocked: boolean,
    questName: string,
    taskOutput: string,
    beaconFields: Array<FilterBarOption>,
    tomeFields: Array<FilterBarOption>,
    tomeMultiSearch: string,
}

const defaultFilters: Filters = {
    isLocked: false,
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
    //If isLocked is not the set state, reset filters
    if (!data || typeof data !== 'object' || !data.isLocked) {
        return defaultFilters
    }

    const schema: Record<keyof Filters, (value: any) => boolean> = {
        isLocked: (v) => typeof v === 'boolean',
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
        const validFilters = validateStoredFilters(JSON.parse(stored));
        if (!validFilters.isLocked) return defaultFilters;
        return validFilters
    } catch {
        return defaultFilters
    }
}

export function calculateFilterCount(filters: Filters, field: FilterFieldType): number {
    switch (field) {
        case FilterFieldType.QUEST_NAME:
            return filters.questName !== "" ? 1 : 0;
        case FilterFieldType.TASK_OUTPUT:
            return filters.taskOutput !== "" ? 1 : 0;
        case FilterFieldType.TOME_MULTI_SEARCH:
            return filters.tomeMultiSearch !== "" ? 1 : 0;
        case FilterFieldType.BEACON_FIELDS:
            return filters.beaconFields.length;
        case FilterFieldType.TOME_FIELDS:
            return filters.tomeFields.length;
        default:
            return 0;
    }
}

export function calculateTotalFilterCount(filters: Filters, fields: FilterFieldType[]): number {
    return fields.reduce((count, field) => count + calculateFilterCount(filters, field), 0);
}

type FilterContextType = {
    filters: Filters
    filterCount: number
    updateFilters: (updates: Partial<Filters>) => void
    resetFilters: () => void
}

const FilterContext = createContext<FilterContextType>({
    filters: defaultFilters,
    filterCount: 0,
    updateFilters: () => { },
    resetFilters: () => { },
});

export function FilterProvider({ children }: { children: React.ReactNode }) {
    const [filters, setFilters] = useState<Filters>(loadFiltersFromStorage);
    const allFields = Object.values(FilterFieldType);
    const filterCount = useMemo(() => calculateTotalFilterCount(filters, allFields), [filters, allFields]);

    const location = useLocation();
    const previousPathname = useRef(location.pathname);

    const updateFilters = (updates: Partial<Filters>) => {
        setFilters(prevFilters => ({
            ...prevFilters,
            ...updates
        }))
    }

    const resetFilters = () => {
        setFilters(defaultFilters);
    };

    useEffect(() => {
        if (previousPathname.current !== location.pathname) {
            if (!filters.isLocked) {
                resetFilters();
            }
            previousPathname.current = location.pathname;
        }
    }, [location.pathname, filters.isLocked]);

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
        <FilterContext.Provider value={{ filters, filterCount, updateFilters, resetFilters }}>
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
