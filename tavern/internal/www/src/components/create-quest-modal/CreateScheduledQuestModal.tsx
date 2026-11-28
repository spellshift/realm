import { gql, useMutation } from "@apollo/client";
import { useToast } from "@chakra-ui/react";
import { useFormik } from "formik";
import { useState } from "react";
import { FormStep, FormSteps } from "../form-steps/FormSteps";
import Modal from "../tavern-base-ui/Modal";
import { ScheduledQuestHostScope } from "../../context/CreateQuestModalContext/types";
import { getRandomQuestName } from "../../utils/questNames";
import { TomeSelectionStep } from "./tome-selection/TomeSelectionStep";
import { ModalQuestFormValues } from "./types";
import { modalQuestSchema } from "./validation";
import {
    ScheduledQuestConfirmStep,
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

type CronFieldSpec = {
    min: number;
    max: number;
    names?: Record<string, number>;
};

const MONTH_NAMES: Record<string, number> = {
    JAN: 1, FEB: 2, MAR: 3, APR: 4, MAY: 5, JUN: 6,
    JUL: 7, AUG: 8, SEP: 9, OCT: 10, NOV: 11, DEC: 12,
};

const DOW_NAMES: Record<string, number> = {
    SUN: 0, MON: 1, TUE: 2, WED: 3, THU: 4, FRI: 5, SAT: 6,
};

// Matches robfig/cron/v3 (Minute | Hour | Dom | Month | Dow), where Dow is 0–6 (Sunday = 0).
const CRON_FIELD_SPECS: CronFieldSpec[] = [
    { min: 0, max: 59 },
    { min: 0, max: 23 },
    { min: 1, max: 31 },
    { min: 1, max: 12, names: MONTH_NAMES },
    { min: 0, max: 6, names: DOW_NAMES },
];

const parseCronAtom = (token: string, spec: CronFieldSpec): number | null => {
    if (/^\d+$/.test(token)) {
        const value = Number(token);
        return value >= spec.min && value <= spec.max ? value : null;
    }
    const named = spec.names?.[token.toUpperCase()];
    return named !== undefined ? named : null;
};

const isValidCronField = (field: string, spec: CronFieldSpec): boolean =>
    field.split(",").every((part) => {
        const stepParts = part.split("/");
        if (stepParts.length > 2) return false;

        const [rangePart, stepStr] = stepParts;
        if (stepStr !== undefined) {
            if (!/^[1-9]\d*$/.test(stepStr)) return false;
            if (Number(stepStr) > spec.max - spec.min + 1) return false;
        }

        if (rangePart === "*") return true;

        const bounds = rangePart.split("-");
        if (bounds.length === 1) {
            return parseCronAtom(bounds[0], spec) !== null;
        }
        if (bounds.length === 2) {
            const start = parseCronAtom(bounds[0], spec);
            const end = parseCronAtom(bounds[1], spec);
            return start !== null && end !== null && start <= end;
        }
        return false;
    });

export const isValidCronSchedule = (schedule: string): boolean => {
    const fields = schedule.trim().split(/\s+/);
    return fields.length === CRON_FIELD_SPECS.length && fields.every((field, index) =>
        isValidCronField(field, CRON_FIELD_SPECS[index]));
};

interface CreateScheduledQuestModalProps {
    isOpen: boolean;
    setOpen: (open: boolean) => void;
    scheduledHost?: ScheduledQuestHostScope;
}

export const CreateScheduledQuestModal = ({ isOpen, setOpen, scheduledHost }: CreateScheduledQuestModalProps) => {
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
                            ...(scheduledHost ? { scheduledHostIDs: [scheduledHost.id] } : {}),
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

    return (
        <Modal isOpen={isOpen} setOpen={setOpen} size="xl">
            <form
                noValidate
                className="flex flex-col gap-4"
                onSubmit={(event) => {
                    event.preventDefault();
                    if (currStep === 2 && !loading) void formik.submitForm();
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
                            <TomeSelectionStep
                                formik={formik}
                                setCurrStep={setCurrStep}
                                subtitle="Choose a tome to execute on configured triggers"
                                controlsPosition="top"
                                backAriaLabel=""
                                continueAriaLabel=""
                            />
                        )}
                        {currStep === 2 && (
                            <ScheduledQuestConfirmStep
                                formik={formik}
                                trigger={trigger}
                                schedule={schedule}
                                scheduledHost={scheduledHost}
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
