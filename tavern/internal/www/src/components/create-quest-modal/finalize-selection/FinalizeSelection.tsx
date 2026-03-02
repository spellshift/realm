import { Heading } from "@chakra-ui/react";

import Button from "../../tavern-base-ui/button/Button";
import { EmptyState, EmptyStateType } from "../../tavern-base-ui/EmptyState";
import FormTextField from "../../tavern-base-ui/FormTextField";
import TomeAccordion from "../../TomeAccordion";
import { useTomeById } from "./useTomeById";
import { ModalQuestFormikProps } from "../types";
import { BeaconTable } from "./BeaconTable";

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
    const params = formik.values.params;

    const { tome: displayTome, loading: tomeLoading } = useTomeById(selectedTomeId);

    const isContinueDisabled = formik.values.name === "" || loading;

    const handleNameQuest = (name: string) => {
        formik.setFieldValue("name", name);
    };

    const isLoading = tomeLoading;

    if (isLoading) {
        return (
            <div className="flex flex-col gap-6">
                <h2 className="text-xl font-semibold text-gray-900">
                    Confirm quest details
                </h2>
                <EmptyState type={EmptyStateType.loading} label="Loading quest details..." />
            </div>
        );
    }

    const hasNoTome = !displayTome;

    return (
        <div className="flex flex-col gap-6">
            <h2 className="text-xl font-semibold text-gray-900">
                Confirm quest details
            </h2>

            {/* Beacons Section */}
            <div className="flex flex-col gap-3">
                <Heading size="sm">Beacons ({selectedBeaconIds.length})</Heading>
                <BeaconTable beaconIds={selectedBeaconIds} />
            </div>

            {/* Tome Section */}
            <div className="flex flex-col gap-3">
                <Heading size="sm">Tome</Heading>
                {hasNoTome ? (
                    <div className="flex items-center justify-center py-4 text-gray-500 border border-dashed border-gray-300 rounded-md">
                        <p className="text-sm">No tome selected</p>
                    </div>
                ) : (
                    <div className="flex flex-col gap-1">
                        <TomeAccordion tome={displayTome} params={params} />
                    </div>
                )}
            </div>

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
