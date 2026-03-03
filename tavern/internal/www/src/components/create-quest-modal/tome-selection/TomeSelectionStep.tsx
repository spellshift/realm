import { useCallback } from "react";

import { TomeNode } from "../../../utils/interfacesQuery";
import { safelyJsonParse } from "../../../utils/utils";
import { TomeSelectionStepProps } from "./types";
import { ConfigureParams } from "./ConfigureParams";
import { StepControls } from "./StepControls";
import { TomeTable } from "./TomeTable";

export const TomeSelectionStep = ({ setCurrStep, formik }: TomeSelectionStepProps) => {
    const selectedTomeId = formik.values.tomeId;

    const handleSelectTome = useCallback(
        (tome: TomeNode) => {
            const { params: tomeParams } = safelyJsonParse(tome.paramDefs || "");
            formik.setFieldValue("tomeId", tome.id);
            formik.setFieldValue("params", tomeParams || []);
        },
        [formik]
    );

    return (
        <div className="flex flex-col gap-6">
            <div className="flex flex-col gap-1">
                <h2 className="text-xl font-semibold text-gray-900">Select a tome</h2>
                <p className="text-sm text-gray-700 italic">
                    Choose a tome to execute on selected beacons
                </p>
            </div>

            <TomeTable
                selectable={true}
                selectedTomeId={selectedTomeId}
                onSelectTome={handleSelectTome}
            />

            {selectedTomeId && <ConfigureParams formik={formik} />}

            <StepControls formik={formik} setCurrStep={setCurrStep} />
        </div>
    );
};

export default TomeSelectionStep;
