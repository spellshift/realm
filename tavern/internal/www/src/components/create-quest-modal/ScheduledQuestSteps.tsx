import { Heading } from "@chakra-ui/react";
import { CopyableKeyValues } from "../copyable-key-values";
import Button from "../tavern-base-ui/button/Button";
import FormTextField from "../tavern-base-ui/FormTextField";
import { SegmentedControl } from "../tavern-base-ui/SegmentedControl";
import { ConfigureParams } from "./tome-selection/ConfigureParams";
import { TomeTableWrapper } from "./tome-selection/TomeTableWrapper";
import { ModalQuestFormikProps } from "./types";
import { TomeNode } from "../../utils/interfacesQuery";

export type ScheduledQuestTrigger = "schedule" | "newHost" | "newBeacon";

export const TRIGGER_OPTIONS: { label: string; value: ScheduledQuestTrigger }[] = [
    { label: "Schedule", value: "schedule" },
    { label: "New host", value: "newHost" },
    { label: "New beacon", value: "newBeacon" },
];

const CRON_EXAMPLES = [
    { label: "Every minute", expression: "* * * * *" },
    { label: "Once an hour", expression: "0 * * * *" },
    { label: "Every other hour", expression: "0 */2 * * *" },
    { label: "Every minute from 9 AM until 5 PM", expression: "* 9-16 * * *" },
];

interface TomeStepProps {
    formik: ModalQuestFormikProps;
    onSelectTome: (tome: TomeNode) => void;
    onBack: () => void;
    onNext: () => void;
}

export const ScheduledQuestTomeStep = ({ formik, onSelectTome, onBack, onNext }: TomeStepProps) => (
    <div className="flex flex-col gap-6">
        <div className="flex flex-col gap-1">
            <h2 className="text-xl font-semibold text-gray-900">Select a tome</h2>
            <p className="text-sm text-gray-700 italic">Choose a tome to execute on configured triggers</p>
        </div>
        <TomeTableWrapper selectable selectedTomeId={formik.values.tomeId} onSelectTome={onSelectTome} />
        {formik.touched.tomeId && formik.errors.tomeId && <p className="text-sm text-red-600">{formik.errors.tomeId}</p>}
        {formik.values.tomeId && <ConfigureParams formik={formik} />}
        <div className="flex gap-2">
            <Button buttonVariant="ghost" onClick={onBack}>Back</Button>
            <Button onClick={onNext}>Continue</Button>
        </div>
    </div>
);

interface TriggerStepProps {
    trigger: ScheduledQuestTrigger;
    setTrigger: (trigger: ScheduledQuestTrigger) => void;
    schedule: string;
    setSchedule: (schedule: string) => void;
    scheduleError: string;
    clearScheduleError: () => void;
    onCancel: () => void;
    onNext: () => void;
}

export const ScheduledQuestTriggerStep = ({ trigger, setTrigger, schedule, setSchedule, scheduleError, clearScheduleError, onCancel, onNext }: TriggerStepProps) => (
    <div className="flex flex-col gap-6">
        <div className="flex flex-col gap-1">
            <h2 className="text-xl font-semibold text-gray-900">Choose a trigger</h2>
            <p className="text-sm text-gray-700 italic">Choose when this quest should run</p>
        </div>
        <SegmentedControl label="Quest trigger" options={TRIGGER_OPTIONS} value={trigger} onChange={setTrigger} />
        {trigger === "schedule" && (
            <div className="flex flex-col gap-4">
                <FormTextField
                    htmlFor="scheduledQuestCron"
                    label="Schedule (cron)"
                    value={schedule}
                    onChange={(event) => { setSchedule(event.target.value); clearScheduleError(); }}
                    placeholder="0 * * * *"
                />
                <div className="text-sm text-gray-600">
                    <p>Minute, hour, day of month, month, day of week.</p>
                    <p className="mt-2 font-medium text-gray-700">Examples</p>
                    <ul className="mt-1 space-y-1">
                        {CRON_EXAMPLES.map(({ label, expression }) => (
                            <li key={expression} className="flex flex-wrap gap-x-3">
                                <code className="w-32 text-gray-900">{expression}</code>
                                <span>{label}</span>
                            </li>
                        ))}
                    </ul>
                </div>
                {scheduleError && <p className="text-sm text-red-600">{scheduleError}</p>}
            </div>
        )}
        {trigger === "newHost" && (
            <p className="text-sm text-gray-700">This quest runs only when a new host checks in for the first time.</p>
        )}
        {trigger === "newBeacon" && (
            <p className="text-sm text-gray-700">This quest runs only when a new beacon checks in for the first time.</p>
        )}
        <div className="flex gap-2">
            <Button buttonVariant="ghost" onClick={onCancel}>Cancel</Button>
            <Button onClick={onNext}>Continue</Button>
        </div>
    </div>
);

interface ConfirmStepProps {
    formik: ModalQuestFormikProps;
    trigger: ScheduledQuestTrigger;
    schedule: string;
    loading: boolean;
    onBack: () => void;
}

export const ScheduledQuestConfirmStep = ({ formik, trigger, schedule, loading, onBack }: ConfirmStepProps) => {
    const triggerLabel = TRIGGER_OPTIONS.find((option) => option.value === trigger)?.label;

    return (
        <div className="flex flex-col gap-6">
            <h2 className="text-xl font-semibold text-gray-900">Confirm scheduled quest</h2>
            <div className="flex flex-col gap-3">
                <Heading size="sm">Trigger</Heading>
                <dl className="grid grid-cols-[max-content_1fr] gap-x-6 gap-y-3 text-sm">
                    <dt className="font-medium text-gray-600">Type</dt>
                    <dd className="text-gray-900">{triggerLabel}</dd>
                    {trigger === "schedule" && (
                        <>
                            <dt className="font-medium text-gray-600">Schedule</dt>
                            <dd className="font-mono text-gray-900">{schedule.trim()}</dd>
                        </>
                    )}
                    <dt className="font-medium text-gray-600">Hosts</dt>
                    <dd className="text-gray-900">All hosts</dd>
                </dl>
            </div>
            <div className="flex flex-col gap-3">
                <Heading size="sm">Tome</Heading>
                <TomeTableWrapper tomeIds={formik.values.tomeId ? [formik.values.tomeId] : []} showFilters={false} />
            </div>
            <CopyableKeyValues params={formik.values.params} heading="Tome parameters" />
            <div>
                <FormTextField
                    htmlFor="scheduledQuestName"
                    label="Quest Name"
                    placeholder="Provide a recognizable name to this quest"
                    value={formik.values.name}
                    onChange={(event) => formik.setFieldValue("name", event.target.value)}
                    onBlur={() => formik.setFieldTouched("name", true)}
                />
                {formik.touched.name && formik.errors.name && <p className="text-sm text-red-600">{formik.errors.name}</p>}
            </div>
            <div className="flex gap-2">
                <Button buttonVariant="ghost" onClick={onBack} disabled={loading}>Back</Button>
                <Button type="submit" disabled={loading}>
                    {loading ? "Creating..." : "Create scheduled quest"}
                </Button>
            </div>
        </div>
    );
};
