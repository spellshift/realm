import { useFormik } from "formik";
import { useState } from "react";

import { FormSteps } from "../../../components/form-steps";
import TomeStepWrapper from "./TomeStepWrapper";
import FinalizeStep from "./FinalizeStep";
import BeaconStepWrapper from "./BeaconStepWrapper";
import { useSubmitQuest } from "../hooks/useSubmitQuest";
import { getRandomQuestName } from "../../../utils/questNames";
import { useLocation } from "react-router-dom";
import { QuestFormValues, LocationStateData } from "../types";
import AlertError from "../../../components/tavern-base-ui/AlertError";
import { createQuestSchema } from "../validation";

const QuestForm = () => {
    const location = useLocation();
    const data = location.state as LocationStateData | undefined;
    const [currStep, setCurrStep] = useState<number>(data?.step || 0);
    const { submitQuest, loading, error, reset } = useSubmitQuest();
    const placeholderTitle = getRandomQuestName();


    const steps = [
        { name: 'Select agent beacons', description: 'Step 1', href: '#', step: 0 },
        { name: 'Select a tome', description: 'Step 2', href: '#', step: 1 },
        { name: 'Confirm quest details', description: 'Step 3', href: '#', step: 2 },
    ];

    const formik = useFormik<QuestFormValues>({
        initialValues: {
            name: data?.name || placeholderTitle,
            tome: data?.tome || null,
            params: data?.params || [],
            beacons: data?.beacons || [],
        },
        validationSchema: createQuestSchema,
        validateOnChange: false,
        validateOnBlur: false,
        onSubmit: (values: QuestFormValues) => submitQuest(values),
    });

    function getStepView(step: number) {
        switch (step) {
            case 0:
                return <BeaconStepWrapper setCurrStep={setCurrStep} formik={formik} />
            case 1:
                return <TomeStepWrapper setCurrStep={setCurrStep} formik={formik} />
            case 2:
                return <FinalizeStep setCurrStep={setCurrStep} formik={formik} loading={loading} />
            default:
                return <div>An error has occurred</div>;
        }
    }

    return (
        <form
            id='create-quest-form'
            className="py-6"
            onSubmit={formik.handleSubmit}
        >
            {error && (
                <div className="mb-4">
                    <AlertError
                        label="Failed to create quest"
                        details="There was an error submitting your quest. Please try again or contact support if the issue persists."
                    />
                    <button
                        type="button"
                        onClick={reset}
                        className="mt-2 text-sm text-red-600 hover:text-red-500 underline"
                    >
                        Dismiss error
                    </button>
                </div>
            )}
            {Object.keys(formik.errors).length > 0 && formik.submitCount > 0 && (
                <div className="mb-4">
                    <AlertError
                        label="Validation error"
                        details={Object.values(formik.errors).filter(err => typeof err === 'string').join(', ')}
                    />
                </div>
            )}
            <div className="grid grid-cols-12">
                <div className="hidden md:flex col-span-3">
                    <FormSteps currStep={currStep} steps={steps} />
                </div>
                <div className="col-span-12 md:col-span-9">
                    {getStepView(currStep)}
                </div>
            </div>
        </form>
    );
}
export default QuestForm;
