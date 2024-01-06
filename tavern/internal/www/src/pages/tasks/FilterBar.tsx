
import React, { useContext } from "react"
import { BeaconFilterBar } from "../../components/beacon-filter-bar";
import { TagContext } from "../../context/TagContext";
import FreeTextSearch from "./FreeTextSearch";

type Props = {
    setSearch: (arg: string) => void;
    setFiltersSelected: (arg: Array<any>) => void;
}
const FilterBar = (props: Props) => {
    const { setSearch, setFiltersSelected } = props;

    const {data, isLoading, error } = useContext(TagContext);

    return (
        <div>
            {(!isLoading && !error && data) && (
                <div className="grid grid-cols-2 gap-2 p-4 bg-white rounded-lg shadow-lg mt-2">
                    <FreeTextSearch setSearch={setSearch} />
                    <BeaconFilterBar beacons={data?.beacons || []} groups={data?.groupTags || []} services={data?.serviceTags || []} setFiltersSelected={setFiltersSelected} />
                </div>
            )}
        </div>
    );
};
export default FilterBar;