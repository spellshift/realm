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
import { CreateQuestInitialData } from "../../context/CreateQuestModalContext";

interface CreateQuestModalProps {
    isOpen: boolean;
    setOpen: (arg: boolean) => void;
    initialFormData?: CreateQuestInitialData;
    onComplete?: (questId: string) => void;
}

function getInitialStep(initialData?: CreateQuestInitialData): number {
    if (initialData?.beacons && initialData.beacons.length > 0) {
        if (initialData?.tomeId) {
            return 2;
        }
        return 1;
    }
    return 0;
}

function getInitialFormValues(
    initialData: CreateQuestInitialData | undefined,
    placeholderTitle: string
): ModalQuestFormValues {
    return {
        name: initialData?.name || placeholderTitle,
        tomeId: initialData?.tomeId ?? null,
        params: initialData?.params || [],
        beacons: initialData?.beacons || [],
    };
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

const CreateQuestModal = ({ isOpen, setOpen, initialFormData, onComplete }: CreateQuestModalProps) => {
    const [placeholderTitle] = useState(() => getRandomQuestName());
    const [currStep, setCurrStep] = useState(() => getInitialStep(initialFormData));
    const { submitQuest, loading } = useModalSubmitQuest();

    const formik = useFormik<ModalQuestFormValues>({
        initialValues: getInitialFormValues(initialFormData, placeholderTitle),
        validationSchema: modalQuestSchema,
        validateOnChange: true,
        validateOnBlur: true,
        onSubmit: async (values: ModalQuestFormValues) => {
            const result = await submitQuest(values);
            const questId = result?.data?.createQuest?.id;
            if (questId && onComplete) {
                onComplete(questId);
            }
        },
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
