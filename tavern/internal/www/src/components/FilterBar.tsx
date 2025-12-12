
import React, { useContext } from "react"
import { BeaconFilterBar } from "./beacon-filter-bar";
import { TagContext } from "../context/TagContext";
import FreeTextSearch from "./tavern-base-ui/DebouncedFreeTextSearch";
import { FilterBarOption } from "../utils/consts";

type Props = {
    searchPlaceholder?: string;
    setSearch: (arg: string) => void;
    setFiltersSelected: (arg: Array<any>) => void;
    filtersSelected: Array<FilterBarOption>
}
const FilterBar = (props: Props) => {
    const { searchPlaceholder, setSearch, setFiltersSelected, filtersSelected } = props;
    const { data, isLoading, error } = useContext(TagContext);

    return (
        <div>
            {(!isLoading && !error && data) && (
                <div className="grid grid-cols-2 gap-2">
                    <FreeTextSearch setSearch={setSearch} placeholder={searchPlaceholder} />
                    <BeaconFilterBar beacons={data?.beacons || []} groups={data?.groupTags || []} services={data?.serviceTags || []} hosts={data?.hosts || []} setFiltersSelected={setFiltersSelected} filtersSelected={filtersSelected} />
                </div>
            )}
        </div>
    );
};
export default FilterBar;
