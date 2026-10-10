import { useState } from "react";

import { EmptyState, EmptyStateType } from "../../tavern-base-ui/EmptyState";
import FreeTextSearch from "../../tavern-base-ui/FreeTextSearch";
import { TomeFilterBar } from "../../TomeFilterBar";
import { FilterBarOption } from "../../../utils/interfacesUI";
import { useFilters } from "../../../context/FilterContext";
import { useTomeIds } from "./useTomeIds";
import { TomeTable } from "./TomeTable";
import { TomeTableWrapperProps } from "./types";

export const TomeTableWrapper = ({
    tomeIds: providedTomeIds,
    selectable = false,
    selectedTomeId,
    onSelectTome,
    showFilters = true,
    emptyMessage = "No tomes available.",
    initialFilters,
}: TomeTableWrapperProps) => {
    const { filters } = useFilters();
    const [tomeMultiSearch, setTomeMultiSearch] = useState(
        initialFilters?.tomeMultiSearch ?? filters.tomeMultiSearch
    );
    const [tomeFields, setTomeFields] = useState<FilterBarOption[]>(
        initialFilters?.tomeFields ?? filters.tomeFields
    );

    const { tomeIds: fetchedTomeIds, initialLoading } = useTomeIds(
        showFilters ? tomeFields : [],
        showFilters ? tomeMultiSearch : ""
    );

    const tomeIds = providedTomeIds ?? fetchedTomeIds;
    const hasActiveFilters = tomeFields.length > 0 || tomeMultiSearch;
    const isLoading = !providedTomeIds && initialLoading;

    if (isLoading) {
        return <EmptyState type={EmptyStateType.loading} label="Loading tomes..." />;
    }

    return (
        <div className="flex flex-col gap-2">
            {showFilters && (
                <div className="grid grid-cols-2 gap-2">
                    <FreeTextSearch
                        placeholder="Tome name, description & params"
                        defaultValue={initialFilters?.tomeMultiSearch ?? filters.tomeMultiSearch}
                        setSearch={setTomeMultiSearch}
                    />
                    <TomeFilterBar setFiltersSelected={setTomeFields} filtersSelected={tomeFields} />
                </div>
            )}
            {tomeIds.length === 0 ? (
                <div className="flex items-center justify-center py-8 text-gray-500 h-[300px]">
                    {hasActiveFilters ? "No tomes matching your search." : emptyMessage}
                </div>
            ) : (
                <TomeTable
                    tomeIds={tomeIds}
                    selectable={selectable}
                    selectedTomeId={selectedTomeId}
                    onSelectTome={onSelectTome}
                />
            )}
        </div>
    );
};

export default TomeTableWrapper;
