
import React, { useContext } from "react"
import { BeaconFilterBar } from "../../../components/beacon-filter-bar";
import { TagContext } from "../../../context/TagContext";
import { FilterBarOption } from "../../../utils/consts";


const HostFilter = (
    {
        setFiltersSelected,
        typeFilters
    }:
        {
            setFiltersSelected: (arg: Array<any>) => void,
            typeFilters: Array<FilterBarOption>
        }
) => {
    const { data, isLoading, error } = useContext(TagContext);

    return (
        <div>
            {(!isLoading && !error && data) && (
                <div className="p-4 bg-white rounded-lg shadow-lg mt-2">
                    <BeaconFilterBar beacons={data?.beacons || []} groups={data?.groupTags || []} services={data?.serviceTags || []} hosts={data?.hosts || []} setFiltersSelected={setFiltersSelected} filtersSelected={typeFilters} />
                </div>
            )}
        </div>
    );
};
export default HostFilter;
