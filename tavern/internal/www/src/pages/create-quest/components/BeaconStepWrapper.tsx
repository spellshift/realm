import { useState } from "react";
import { EmptyState, EmptyStateType } from "../../../components/tavern-base-ui/EmptyState";
import { useTags } from "../../../context/TagContext";
import { getOnlineBeacons, isBeaconSelected } from "../../../utils/utils";
import BeaconStep from "./BeaconStep";
import Button from "../../../components/tavern-base-ui/button/Button";
import { SelectedBeacons } from "../../../utils/interfacesUI";
import { QuestFormikProps } from "../types";

type Props = {
    setCurrStep: (step: number) => void;
    formik: QuestFormikProps;
}
export const BeaconStepWrapper = (props: Props) => {
    const { setCurrStep, formik } = props;
    const [selectedBeacons, setSelectedBeacons] = useState<SelectedBeacons>({});

    const { data, isLoading } = useTags();

    const onlineBeacons = getOnlineBeacons(data.beacons);

    const hasBeaconSelected = isBeaconSelected(selectedBeacons);

    const handleClickContinue = () => {
        const beaconToSubmit: string[] = [];
        for (const key in selectedBeacons) {
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
            {isLoading ?
                (
                    <EmptyState type={EmptyStateType.loading} label="Loading beacons..." />
                ) : (
                    <BeaconStep beacons={onlineBeacons} groups={data.groupTags} services={data.serviceTags} hosts={data.hosts} selectedBeacons={selectedBeacons} setSelectedBeacons={setSelectedBeacons} />
                )}
            <div className="flex flex-row gap-2">
                <Button
                    buttonStyle={{ color: "purple", size: "md" }}
                    onClick={handleClickContinue}
                    disabled={!hasBeaconSelected}
                    aria-label="continue beacon step"
                >
                    Continue
                </Button>
            </div>
        </div>
    );
}
export default BeaconStepWrapper;
