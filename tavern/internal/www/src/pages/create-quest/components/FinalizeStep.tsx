import { Heading } from "@chakra-ui/react";
import { useContext } from "react";

import BeaconTile from "../../../components/BeaconTile";
import FormTextField from "../../../components/tavern-base-ui/FormTextField";
import TomeAccordion from "../../../components/TomeAccordion";
import { TagContext } from "../../../context/TagContext";
import { BeaconType } from "../../../utils/consts";
import { convertArrayToObject } from "../../../utils/utils";


type Props = {
    setCurrStep: (arg1: number) => void;
    formik: any;
}
const FinalizeStep = (props: Props) => {
    const { formik, setCurrStep } = props;

    const isContinueDisabled = formik?.values?.name === "";

    const handleNameQuest = (name: string) => {
        formik.setFieldValue('name', name);
    };

    const { data } = useContext(TagContext);

    function getSelectedBeacons(beacons: Array<BeaconType>, selectedBeaconIds: Array<string>) {
        const beaconSelectedObject = convertArrayToObject(selectedBeaconIds);
        return beacons.filter((beacon: BeaconType) => beaconSelectedObject[beacon?.id]);
    }

    const beaconsSelected = getSelectedBeacons(data?.beacons || [], formik.values.beacons);


    return (
        <div className="flex flex-col gap-6">
            <h2 className="text-xl font-semibold text-gray-900">Confirm quest details</h2>
            <div className="flex flex-col gap-3">
                <Heading size="sm" >Beacons ({formik?.values?.beacons?.length})</Heading>
                <div className="flex flex-col gap-2 max-h-80 overflow-scroll px-4">
                    {beaconsSelected.map((beacon) => {
                        return <BeaconTile key={`beaconTile_${beacon.id}`} beaconData={beacon} />
                    })}
                </div>
            </div>
            <div className="flex flex-col gap-3">
                <Heading size="sm" >Tome</Heading>
                <div className="flex flex-col gap-1">
                    <TomeAccordion tome={formik?.values?.tome} params={formik?.values?.params} />
                </div>
            </div>
            <FormTextField
                htmlFor="questName"
                label="Quest name"
                placeholder={"Provide a recognizable name to this quest"}
                value={formik?.values?.name}
                onChange={(event) => handleNameQuest(event?.target?.value)}
            />
            <div className="flex flex-row gap-2">
                <button
                    className="inline-flex items-center rounded-md bg-gray-50 py-3 px-4 text-sm font-semibold text-purple-600 shadow-sm hover:bg-purple-100"
                    onClick={() => setCurrStep(1)}
                >
                    Back
                </button>
                <button
                    className="btn-primary"
                    onClick={(event) => {
                        event.preventDefault();
                        formik.handleSubmit();
                    }}
                    disabled={isContinueDisabled}
                    type="submit"
                >
                    Submit
                </button>
            </div>
        </div>
    );
}
export default FinalizeStep;
