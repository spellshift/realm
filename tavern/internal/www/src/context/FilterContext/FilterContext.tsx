import React, { createContext, useContext, useEffect, useMemo, useRef, useState } from 'react'
import { FilterBarOption } from '../../utils/interfacesUI'
import { useLocation } from 'react-router-dom'

export enum FilterFieldType {
    BEACON_FIELDS = 'beaconFields',
    TASK_OUTPUT = 'taskOutput',
    QUEST_NAME = 'questName',
    TOME_FIELDS = 'tomeFields',
    TOME_MULTI_SEARCH = "tomeMultiSearch",
    ASSET_NAME = "assetName",
    USER = "user"
}

const STORAGE_KEY = 'realm-filters-v1.1'

export type Filters = {
    questName: string,
    taskOutput: string,
    beaconFields: Array<FilterBarOption>,
    tomeFields: Array<FilterBarOption>,
    tomeMultiSearch: string,
    assetName: string,
    userId: string,
}

// Storage format includes isLocked alongside filter values
type StoredFilters = Filters & { isLocked: boolean }

const defaultFilters: Filters = {
    questName: "",
    taskOutput: "",
    beaconFields: [],
    tomeFields: [],
    tomeMultiSearch: "",
    assetName: "",
    userId: ""
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

function validateStoredFilters(data: any): StoredFilters {
    //If isLocked is not set, reset filters
    if (!data || typeof data !== 'object' || !data.isLocked) {
        return { ...defaultFilters, isLocked: false }
    }

    const schema: Record<keyof StoredFilters, (value: any) => boolean> = {
        isLocked: (v) => typeof v === 'boolean',
        questName: (v) => typeof v === 'string',
        taskOutput: (v) => typeof v === 'string',
        tomeMultiSearch: (v) => typeof v === 'string',
        assetName: (v) => typeof v === 'string',
        userId: (v) => typeof v === 'string',
        beaconFields: (v) => Array.isArray(v) && v.every(isValidFilterBarOption),
        tomeFields: (v) => Array.isArray(v) && v.every(isValidFilterBarOption),
    }

    for (const [key, validator] of Object.entries(schema)) {
        if (!(key in data) || !validator(data[key])) {
            return { ...defaultFilters, isLocked: false }
        }
    }

    return data as StoredFilters
}

function loadFromStorage(): { filters: Filters, isLocked: boolean } {
    if (typeof window === 'undefined') {
        return { filters: defaultFilters, isLocked: false }
    }

    const stored = sessionStorage.getItem(STORAGE_KEY)
    if (!stored) {
        return { filters: defaultFilters, isLocked: false }
    }

    try {
        const validated = validateStoredFilters(JSON.parse(stored));
        // Only restore filters if they were locked
        if (!validated.isLocked) {
            return { filters: defaultFilters, isLocked: false }
        }
        const { isLocked, ...filters } = validated;
        return { filters, isLocked }
    } catch {
        return { filters: defaultFilters, isLocked: false }
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
        case FilterFieldType.ASSET_NAME:
            return filters.assetName !== "" ? 1 : 0;
        case FilterFieldType.USER:
            return filters.userId !== "" ? 1 : 0;
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
    isLocked: boolean
    setIsLocked: React.Dispatch<React.SetStateAction<boolean>> 
    updateFilters: (updates: Partial<Filters>) => void
    resetFilters: () => void
}

const FilterContext = createContext<FilterContextType>({
    filters: defaultFilters,
    filterCount: 0,
    isLocked: false,
    setIsLocked: () => { },
    updateFilters: () => { },
    resetFilters: () => { },
});

export function FilterProvider({ children }: { children: React.ReactNode }) {
    const initialState = loadFromStorage();
    const [filters, setFilters] = useState<Filters>(initialState.filters);
    const [isLocked, setIsLocked] = useState<boolean>(initialState.isLocked);

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
            if (!isLocked) {
                resetFilters();
            }
            previousPathname.current = location.pathname;
        }
    }, [location.pathname, isLocked]);

    useEffect(() => {
        if (typeof window !== 'undefined') {
            // Store both filters and isLocked together
            sessionStorage.setItem(STORAGE_KEY, JSON.stringify({ ...filters, isLocked }))
        }
    }, [filters, isLocked]); // Don't collapse these useEffects, this helps fix race condition

    useEffect(() => {
        const handleStorage = (event: StorageEvent) => {
            if (event.key === STORAGE_KEY && event.newValue) {
                const validated = validateStoredFilters(JSON.parse(event.newValue));
                const { isLocked: storedIsLocked, ...storedFilters } = validated;
                setFilters(storedFilters);
                setIsLocked(storedIsLocked);
            }
        }
        window.addEventListener('storage', handleStorage)
        return () => window.removeEventListener('storage', handleStorage)
    }, [])

    return (
        <FilterContext.Provider value={{ filters, filterCount, isLocked, setIsLocked, updateFilters, resetFilters }}>
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
