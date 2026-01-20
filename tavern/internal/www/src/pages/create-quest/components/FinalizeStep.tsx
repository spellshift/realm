import { Heading } from "@chakra-ui/react";

import BeaconTile from "../../../components/BeaconTile";
import FormTextField from "../../../components/tavern-base-ui/FormTextField";
import TomeAccordion from "../../../components/TomeAccordion";
import { useTags } from "../../../context/TagContext";
import { convertArrayToObject } from "../../../utils/utils";
import Button from "../../../components/tavern-base-ui/button/Button";
import { BeaconNode } from "../../../utils/interfacesQuery";
import { QuestFormikProps } from "../types";
import { FilterBarOption } from "../../../utils/interfacesUI";


type Props = {
    setCurrStep: (step: number) => void;
    formik: QuestFormikProps;
    loading?: boolean;
}
const FinalizeStep = (props: Props) => {
    const { formik, setCurrStep, loading = false } = props;

    const isContinueDisabled = formik.values.name === "" || loading;

    const handleNameQuest = (name: string) => {
        formik.setFieldValue('name', name);
    };

    const { data } = useTags();

    function getSelectedBeacons(beacons: Array<FilterBarOption & BeaconNode>, selectedBeaconIds: string[]): BeaconNode[] {
        const beaconSelectedObject = convertArrayToObject(selectedBeaconIds);
        return beacons.filter((beacon) => beaconSelectedObject[beacon.id]);
    }

    const beaconsSelected = getSelectedBeacons(data.beacons, formik.values.beacons);


    return (
        <div className="flex flex-col gap-6">
            <h2 className="text-xl font-semibold text-gray-900">Confirm quest details</h2>
            <div className="flex flex-col gap-3">
                <Heading size="sm" >Beacons ({formik.values.beacons.length})</Heading>
                <div className="flex flex-col gap-2 max-h-80 overflow-scroll px-4">
                    {beaconsSelected.map((beacon) => {
                        return <BeaconTile key={`beaconTile_${beacon.id}`} beacon={beacon} />
                    })}
                </div>
            </div>
            <div className="flex flex-col gap-3">
                <Heading size="sm" >Tome</Heading>
                <div className="flex flex-col gap-1">
                    {formik.values.tome && (
                        <TomeAccordion tome={formik.values.tome} params={formik.values.params} />
                    )}
                </div>
            </div>
            <FormTextField
                htmlFor="questName"
                label="Quest name"
                placeholder={"Provide a recognizable name to this quest"}
                value={formik.values.name}
                onChange={(event) => handleNameQuest(event.target.value)}
            />
            {formik.errors.name && formik.touched.name && (
                <p className="text-sm text-red-600 mt-1">{formik.errors.name}</p>
            )}
            <div className="flex flex-row gap-2">
                <Button
                    buttonVariant="ghost"
                    onClick={() => setCurrStep(1)}
                    disabled={loading}
                    aria-label="back button"
                >
                    Back
                </Button>
                <Button
                    onClick={(event) => {
                        event.preventDefault();
                        formik.handleSubmit();
                    }}
                    disabled={isContinueDisabled}
                    type="submit"
                    aria-label="submit quest"
                >
                    {loading ? "Creating quest..." : "Submit"}
                </Button>
            </div>
        </div>
    );
}
export default FinalizeStep;
