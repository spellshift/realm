import { TomeSelectionStepProps } from "./types";
import { FieldInputParams } from "../../../utils/interfacesUI";
import Button from "../../tavern-base-ui/button/Button";

export const StepControls = ({
    formik,
    setCurrStep,
    onBack,
    onNext,
    backAriaLabel = "back to beacons",
    continueAriaLabel = "continue tome step",
}: TomeSelectionStepProps) => {
    const params = formik.values.params;
    const selectedTomeId = formik.values.tomeId;

    const validateAndContinue = async () => {
        if (!selectedTomeId) return;

        const errors = await formik.validateForm();
        const hasParamErrors = errors.params !== undefined;

        if (hasParamErrors) {
            params.forEach((_, index) => {
                formik.setFieldTouched(`params.${index}.value`, true, false);
            });
        } else if (onNext) {
            onNext();
        } else {
            setCurrStep(2);
        }
    };

    const isContinueDisabled = () => {
        if (!selectedTomeId) return true;
        if (params.length === 0) return false;

        return params.some(
            (param: FieldInputParams) => !param.value || param.value.trim() === ""
        ) ?? false;
    };

    return (
        <div className="flex flex-row gap-2">
            <Button
                onClick={onBack ?? (() => setCurrStep(0))}
                buttonVariant="ghost"
                {...(backAriaLabel ? { "aria-label": backAriaLabel } : {})}
            >
                Back
            </Button>
            <Button
                buttonStyle={{ color: "purple", size: "md" }}
                onClick={validateAndContinue}
                disabled={isContinueDisabled()}
                {...(continueAriaLabel ? { "aria-label": continueAriaLabel } : {})}
            >
                Continue
            </Button>
        </div>
    );
};
