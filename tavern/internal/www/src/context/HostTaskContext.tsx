import React, { createContext, Dispatch, SetStateAction } from "react";
import { useParams } from "react-router-dom";
import { useTasks } from "../hooks/useTasks";
import { DEFAULT_QUERY_TYPE } from "../utils/enums";

export type HostTaskContextQueryType = {
    data: undefined | any;
    loading: boolean;
    error: any;
    page: number,
    filtersSelected: Array<any>,
    setPage: Dispatch<SetStateAction<number>>,
    setSearch: (search: string) => void,
    setFiltersSelected: (filters: Array<any>) => void,
    updateTaskList: () => void
}

const defaultValue = {
    data: undefined,
    loading: false,
    error: false,
    page: 1,
    filtersSelected: [],
    setPage: null,
    setSearch: null,
    setFiltersSelected: null,
    updateTaskList: null,
} as any;

export const HostTaskContext = createContext<HostTaskContextQueryType>(defaultValue);

export const HostTaskContextProvider = ({ children }: { children: React.ReactNode }) => {
    const { hostId } = useParams();

    const {
        data,
        loading,
        error,
        setSearch,
        setFiltersSelected,
        filtersSelected,
        updateTaskList,
        page,
        setPage
    } = useTasks(DEFAULT_QUERY_TYPE.hostIDQuery, hostId);

    return (
        <HostTaskContext.Provider value={{ data, loading, error, setSearch, setFiltersSelected, filtersSelected, updateTaskList, page, setPage }}>
            {children}
        </HostTaskContext.Provider>
    );
};
