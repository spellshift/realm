import { useFormik } from "formik";
import { useMemo, useState } from "react";

import Modal from "../tavern-base-ui/Modal";
import { FormSteps } from "../form-steps";
import { getRandomQuestName } from "../../utils/questNames";
import { ModalQuestFormValues, ModalQuestFormikProps } from "./types";
import { modalQuestSchema } from "./validation";
import { useModalSubmitQuest } from "./useModalSubmitQuest";
import { BeaconSelectionStep } from "./beacon-selection";
import { TomeSelectionStep } from "./tome-selection";
import { FinalizeSelection } from "./finalize-selection";

interface CreateQuestModalProps {
    isOpen: boolean;
    setOpen: (arg: any) => any;
    initialBeacons?: string[];
}

interface StepConfig {
    meta: any;
    component: React.ComponentType<{
        setCurrStep: (step: number) => void;
        formik: ModalQuestFormikProps;
        loading?: boolean;
        setOpen: (arg: any) => any;
    }>;
}

const STEPS: StepConfig[] = [
    {
        meta: { name: 'Select beacons', description: 'Step 1', href: '#', step: 0 },
        component: BeaconSelectionStep,
    },
    {
        meta: { name: 'Select tome', description: 'Step 2', href: '#', step: 1 },
        component: TomeSelectionStep,
    },
    {
        meta: { name: 'Confirm details', description: 'Step 3', href: '#', step: 2 },
        component: FinalizeSelection,
    },
];

const CreateQuestModal = ({ isOpen, setOpen, initialBeacons = [] }: CreateQuestModalProps) => {
    const [currStep, setCurrStep] = useState(0);
    const { submitQuest, loading } = useModalSubmitQuest(setOpen);
    const [placeholderTitle] = useState(() => getRandomQuestName());

    const formik = useFormik<ModalQuestFormValues>({
        initialValues: {
            name: placeholderTitle,
            tomeId: null,
            params: [],
            beacons: initialBeacons,
        },
        validationSchema: modalQuestSchema,
        validateOnChange: false,
        validateOnBlur: false,
        onSubmit: (values: ModalQuestFormValues) => submitQuest(values),
    });

    const stepsMeta = useMemo(() => STEPS.map(s => s.meta), []);
    const currentStep = STEPS[currStep];
    const StepComponent = currentStep?.component;

    return (
        <Modal isOpen={isOpen} setOpen={setOpen} size="xl">
            <form
                id="create-quest-modal-form"
                className="flex flex-col gap-4"
                onSubmit={formik.handleSubmit}
            >
                <div className="flex flex-row gap-6">
                    <div className="flex-shrink-0 mt-2 lg:block hidden">
                        <FormSteps steps={stepsMeta} currStep={currStep} />
                    </div>
                    <div className="flex-1 min-w-0">
                        {StepComponent && (
                            <StepComponent
                                setCurrStep={setCurrStep}
                                formik={formik}
                                loading={loading}
                                setOpen={setOpen}
                            />
                        )}
                    </div>
                </div>
            </form>
        </Modal>
    );
};

export default CreateQuestModal;
