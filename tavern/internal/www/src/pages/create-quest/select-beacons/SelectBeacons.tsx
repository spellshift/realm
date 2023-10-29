import React, { useContext, useState } from "react";
import { EmptyState, EmptyStateType } from "../../../components/tavern-base-ui/EmptyState";
import { TagContext } from "../../../context/TagContext";
import { SelectedBeacons } from "../../../utils/consts";
import { BeaconView } from "./beacon-view";

type Props = {
    setCurrStep: (arg1: number) => void;
    formik: any;
}
export const SelectBeacons = (props: Props) => {
    const {setCurrStep, formik} = props;
    const [selectedBeacons, setSelectedBeacons] = useState<any>({});

    const {data, isLoading, error } = useContext(TagContext);

    function isBeaconSelected(){
        for (let key in selectedBeacons) {
            if (selectedBeacons[key] === true) {
                return true;
            }
        }
        return false;
    }
    const hasBeaconSelected = isBeaconSelected();

    const handleClickContinue = (selectedBeacons: SelectedBeacons) => {
        const beaconToSubmit = [] as Array<string>;
        for (let key in selectedBeacons) {
            if (selectedBeacons[key] === true) {
               beaconToSubmit.push(key);
            }
        }
        formik.setFieldValue('beacons', beaconToSubmit);
        formik.handleSubmit();
    }

    return (
        <div className="flex flex-col gap-6">
            <h2 className="text-xl font-semibold text-gray-900">Select agent beacons</h2>
            {isLoading || data === undefined ?
            (
                <EmptyState type={EmptyStateType.loading} label="Loading beacons..." />
            ): (
                <BeaconView beacons={data?.beacons || []} groups={data?.groupTags || []} services={data?.serviceTags || []} selectedBeacons={selectedBeacons} setSelectedBeacons={setSelectedBeacons} />
            )}
             <div className="flex flex-row gap-2">
                <button
                    className="inline-flex items-center rounded-md bg-gray-50 py-3 px-4 text-sm font-semibold text-purple-600 shadow-sm hover:bg-purple-100"
                    onClick={()=> setCurrStep(0)}
                >
                     Back
                 </button>
                 <button
                    className="btn-primary"
                    onClick={(event) => {
                        event.preventDefault();
                        handleClickContinue(selectedBeacons);
                    }}
                    disabled={!hasBeaconSelected}
                    type="submit"
                >
                    Submit
                </button>
             </div>
        </div>
    );
}