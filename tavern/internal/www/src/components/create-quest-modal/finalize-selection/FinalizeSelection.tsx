import { Heading } from "@chakra-ui/react";

import Button from "../../tavern-base-ui/button/Button";
import FormTextField from "../../tavern-base-ui/FormTextField";
import { ModalQuestFormikProps } from "../types";
import { BeaconTable } from "../beacon-selection/BeaconTable";
import { TomeTableWrapper } from "../tome-selection/TomeTableWrapper";
import { CopyableKeyValues } from "../../copyable-key-values";

interface FinalizeSelectionProps {
    setCurrStep: (step: number) => void;
    formik: ModalQuestFormikProps;
    loading?: boolean;
}

export const FinalizeSelection = ({
    formik,
    setCurrStep,
    loading = false,
}: FinalizeSelectionProps) => {
    const selectedBeaconIds = formik.values.beacons;
    const selectedTomeId = formik.values.tomeId;

    const isContinueDisabled = formik.values.name === "" || loading;

    const handleNameQuest = (name: string) => {
        formik.setFieldValue("name", name);
    };

    const tomeIds = selectedTomeId ? [selectedTomeId] : [];

    return (
        <div className="flex flex-col gap-6">
            <h2 className="text-xl font-semibold text-gray-900">
                Confirm quest details
            </h2>

            {/* Beacons Section */}
            <div className="flex flex-col gap-3">
                <Heading size="sm">Beacons ({selectedBeaconIds.length})</Heading>
                <BeaconTable beaconIds={selectedBeaconIds} emptyMessage="Unable to find beacons" />
            </div>

            {/* Tome Section */}
            <div className="flex flex-col gap-3">
                <Heading size="sm">Tome</Heading>
                <TomeTableWrapper
                    tomeIds={tomeIds}
                    showFilters={false}
                    emptyMessage="No tome selected"
                />
            </div>

            {/* Parameters Section */}
            <CopyableKeyValues params={formik.values.params} heading="Tome parameters"/>

            {/* Quest Name Input */}
            <FormTextField
                htmlFor="questName"
                label="Quest name"
                placeholder="Provide a recognizable name to this quest"
                value={formik.values.name}
                onChange={(event) => handleNameQuest(event.target.value)}
            />
            {formik.errors.name && formik.touched.name && (
                <p className="text-sm text-red-600 mt-1">{formik.errors.name}</p>
            )}

            {/* Navigation Buttons */}
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
};

export default FinalizeSelection;
