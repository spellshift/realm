
import React, { useContext } from "react"
import { BeaconFilterBar } from "../../components/beacon-filter-bar";
import { TagContext } from "../../context/TagContext";
import FreeTextSearch from "../../components/tavern-base-ui/DebouncedFreeTextSearch";

type Props = {
    setSearch: (arg: string) => void;
    setFiltersSelected: (arg: Array<any>) => void;
}
const FilterBar = (props: Props) => {
    const { setSearch, setFiltersSelected } = props;

    const { data, isLoading, error } = useContext(TagContext);

    return (
        <div>
            {(!isLoading && !error && data) && (
                <div className="grid grid-cols-2 gap-2">
                    <FreeTextSearch setSearch={setSearch} />
                    <BeaconFilterBar beacons={data?.beacons || []} groups={data?.groupTags || []} services={data?.serviceTags || []} hosts={data?.hosts || []} setFiltersSelected={setFiltersSelected} />
                </div>
            )}
        </div>
    );
};
export default FilterBar;
