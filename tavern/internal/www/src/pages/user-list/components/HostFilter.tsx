
import React, { useContext } from "react"
import { BeaconFilterBar } from "../../../components/beacon-filter-bar";
import { TagContext } from "../../../context/TagContext";


const HostFilter = (
    { setFiltersSelected }:
        { setFiltersSelected: (arg: Array<any>) => void }
) => {

    const { data, isLoading, error } = useContext(TagContext);

    return (
        <div>
            {(!isLoading && !error && data) && (
                <div className="p-4 bg-white rounded-lg shadow-lg mt-2">
                    <BeaconFilterBar beacons={data?.beacons || []} groups={data?.groupTags || []} services={data?.serviceTags || []} hosts={data?.hosts || []} setFiltersSelected={setFiltersSelected} />
                </div>
            )}
        </div>
    );
};
export default HostFilter;
