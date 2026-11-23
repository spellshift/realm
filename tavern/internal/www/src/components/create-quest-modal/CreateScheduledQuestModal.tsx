import { gql, useMutation } from "@apollo/client";
import { useToast } from "@chakra-ui/react";
import { useFormik } from "formik";
import { useState } from "react";
import { FormStep, FormSteps } from "../form-steps/FormSteps";
import Modal from "../tavern-base-ui/Modal";
import { TomeNode } from "../../utils/interfacesQuery";
import { getRandomQuestName } from "../../utils/questNames";
import { safelyJsonParse } from "../../utils/utils";
import { ModalQuestFormValues } from "./types";
import { modalQuestSchema } from "./validation";
import {
    ScheduledQuestConfirmStep,
    ScheduledQuestTomeStep,
    ScheduledQuestTrigger,
    ScheduledQuestTriggerStep,
} from "./ScheduledQuestSteps";

const CREATE_SCHEDULED_TASK = gql`
    mutation CreateScheduledTask($input: CreateScheduledTaskInput!) {
        createScheduledTask(input: $input) {
            id
            name
        }
    }
`;

const STEPS: FormStep[] = [
    { name: "Trigger", description: "Step 1", href: "#", step: 0 },
    { name: "Tome", description: "Step 2", href: "#", step: 1 },
    { name: "Confirm", description: "Step 3", href: "#", step: 2 },
];

const CRON_FIELD_BOUNDS: [number, number][] = [[0, 59], [0, 23], [1, 31], [1, 12], [0, 7]];

const isValidCronField = (field: string, min: number, max: number): boolean =>
    field.split(",").every((part) => {
        if (part === "*") return true;
        if (/^\*\/[1-9]\d*$/.test(part)) return Number(part.slice(2)) <= max - min + 1;
        if (/^\d+$/.test(part)) return Number(part) >= min && Number(part) <= max;
        if (!/^\d+-\d+$/.test(part)) return false;

        const [start, end] = part.split("-").map(Number);
        return start >= min && start <= end && end <= max;
    });

const isValidCronSchedule = (schedule: string): boolean => {
    const fields = schedule.trim().split(/\s+/);
    return fields.length === CRON_FIELD_BOUNDS.length && fields.every((field, index) =>
        isValidCronField(field, ...CRON_FIELD_BOUNDS[index]));
};

interface CreateScheduledQuestModalProps {
    isOpen: boolean;
    setOpen: (open: boolean) => void;
}

export const CreateScheduledQuestModal = ({ isOpen, setOpen }: CreateScheduledQuestModalProps) => {
    const [generatedName] = useState(() => getRandomQuestName());
    const [currStep, setCurrStep] = useState(0);
    const [trigger, setTrigger] = useState<ScheduledQuestTrigger>("schedule");
    const [schedule, setSchedule] = useState("0 * * * *");
    const [scheduleError, setScheduleError] = useState("");
    const toast = useToast();
    const [createScheduledTask, { loading }] = useMutation(CREATE_SCHEDULED_TASK);

    const formik = useFormik<ModalQuestFormValues>({
        initialValues: { name: generatedName, tomeId: null, params: [], beacons: [] },
        validationSchema: modalQuestSchema.omit(["beacons"]),
        validate: (values) => values.name.trim() ? {} : { name: "Quest name is required" },
        onSubmit: async (values) => {
            const parameters = Object.fromEntries(values.params.map(({ name, value }) => [name, value]));
            try {
                await createScheduledTask({
                    variables: {
                        input: {
                            name: values.name.trim(),
                            description: "",
                            tomeID: values.tomeId,
                            parameters: JSON.stringify(parameters),
                            runOnSchedule: trigger === "schedule" ? schedule.trim() : "",
                            runOnFirstHostCallback: trigger === "newHost",
                            runOnNewBeaconCallback: trigger === "newBeacon",
                        },
                    },
                });
                toast({ title: "Scheduled quest created", status: "success", duration: 5000, isClosable: true, position: "bottom-right" });
                setOpen(false);
            } catch (error) {
                toast({
                    title: "Unable to create scheduled quest",
                    description: error instanceof Error ? error.message : "Please try again.",
                    status: "error",
                    duration: 5000,
                    isClosable: true,
                    position: "bottom-right",
                });
            }
        },
    });

    const advanceFromTrigger = () => {
        if (trigger === "schedule" && !isValidCronSchedule(schedule)) {
            setScheduleError("Enter a valid five-field cron schedule, such as 0 * * * *.");
            return;
        }
        setScheduleError("");
        setCurrStep(1);
    };

    const selectTome = (tome: TomeNode) => {
        const { params } = safelyJsonParse(tome.paramDefs || "");
        formik.setFieldValue("tomeId", tome.id);
        formik.setFieldValue("params", params || []);
    };

    const advanceFromTome = async () => {
        const errors = await formik.validateForm();
        await formik.setFieldTouched("tomeId", true, false);
        formik.values.params.forEach((_, index) => {
            formik.setFieldTouched(`params.${index}.value`, true, false);
        });
        if (!formik.values.tomeId || errors.tomeId || errors.params) return;
        setCurrStep(2);
    };

    return (
        <Modal isOpen={isOpen} setOpen={setOpen} size="xl">
            <form
                className="flex flex-col gap-4"
                onSubmit={(event) => {
                    event.preventDefault();
                    if (currStep === 2) void formik.submitForm();
                }}
            >
                <div className="flex flex-row gap-6">
                    <div className="flex-shrink-0 mt-2 lg:block hidden">
                        <FormSteps steps={STEPS} currStep={currStep} />
                    </div>
                    <div className="flex-1 min-w-0">
                        {currStep === 0 && (
                            <ScheduledQuestTriggerStep
                                trigger={trigger}
                                setTrigger={setTrigger}
                                schedule={schedule}
                                setSchedule={setSchedule}
                                scheduleError={scheduleError}
                                clearScheduleError={() => setScheduleError("")}
                                onCancel={() => setOpen(false)}
                                onNext={advanceFromTrigger}
                            />
                        )}
                        {currStep === 1 && (
                            <ScheduledQuestTomeStep
                                formik={formik}
                                onSelectTome={selectTome}
                                onBack={() => setCurrStep(0)}
                                onNext={advanceFromTome}
                            />
                        )}
                        {currStep === 2 && (
                            <ScheduledQuestConfirmStep
                                formik={formik}
                                trigger={trigger}
                                schedule={schedule}
                                loading={loading}
                                onBack={() => setCurrStep(1)}
                            />
                        )}
                    </div>
                </div>
            </form>
        </Modal>
    );
};
