import React, { useContext, useState } from "react";
import { EmptyState, EmptyStateType } from "../../../components/tavern-base-ui/EmptyState";
import { TagContext } from "../../../context/TagContext";
import { SelectedBeacons } from "../../../utils/consts";
import { getOnlineBeacons, isBeaconSelected } from "../../../utils/utils";
import BeaconStep from "./BeaconStep";
import Button from "../../../components/tavern-base-ui/button/Button";

type Props = {
    setCurrStep: (arg1: number) => void;
    formik: any;
}
export const BeaconStepWrapper = (props: Props) => {
    const { setCurrStep, formik } = props;
    const [selectedBeacons, setSelectedBeacons] = useState<any>({});

    const { data, isLoading } = useContext(TagContext);

    //filter to only show online beacons
    const onlineBeacons = getOnlineBeacons(data?.beacons || []);

    const hasBeaconSelected = isBeaconSelected(selectedBeacons);

    const handleClickContinue = (selectedBeacons: SelectedBeacons) => {
        const beaconToSubmit = [] as Array<string>;
        for (let key in selectedBeacons) {
            if (selectedBeacons[key] === true) {
                beaconToSubmit.push(key);
            }
        }
        formik.setFieldValue('beacons', beaconToSubmit);
        setCurrStep(1);
    }

    return (
        <div className="flex flex-col gap-6">
            <div className="flex flex-col gap-1">
                <h2 className="text-xl font-semibold text-gray-900">Select agent beacons</h2>
                <p className="text-sm text-gray-700 italic">Only active beacons are available for selection</p>
            </div>
            {isLoading || data === undefined ?
                (
                    <EmptyState type={EmptyStateType.loading} label="Loading beacons..." />
                ) : (
                    <BeaconStep beacons={onlineBeacons} groups={data?.groupTags || []} services={data?.serviceTags || []} hosts={data?.hosts || []} selectedBeacons={selectedBeacons} setSelectedBeacons={setSelectedBeacons} />
                )}
            <div className="flex flex-row gap-2">
                <Button
                    buttonStyle={{ color: "purple", size: "md" }}
                    onClick={() => handleClickContinue(selectedBeacons)}
                    disabled={!hasBeaconSelected}
                >
                    Continue
                </Button>
            </div>
        </div>
    );
}
export default BeaconStepWrapper;
