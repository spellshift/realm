import { useState } from "react";
import { useMutation } from "@apollo/client";
import Select, { MultiValue } from "react-select";
import Modal from "../../../components/tavern-base-ui/Modal";
import Button from "../../../components/tavern-base-ui/button/Button";
import AlertError from "../../../components/tavern-base-ui/AlertError";
import FormTextField from "../../../components/tavern-base-ui/FormTextField";
import CodeBlock from "../../../components/tavern-base-ui/CodeBlock";
import { REGISTER_BUILDER, downloadBuilderConfig } from "../queries";
import { PlatformOption, PLATFORM_OPTIONS } from "../types";

const DEFAULT_UPSTREAM = "http://127.0.0.1:8000";
const DEFAULT_POLL_INTERVAL = 5;

/**
 * Enroll builder modal -- inlined Modal + FormTextField + CodeBlock (the
 * same primitives CreateLinkModal composes) rather than a new Tavern UI
 * component. registerBuilder returns a one-time mTLS cert/config bundle
 * that can't be re-fetched later, so the result view stays up until the
 * operator explicitly closes it.
 */
export const EnrollBuilderModal = ({
    isOpen,
    onClose,
    onEnrolled,
}: {
    isOpen: boolean;
    onClose: () => void;
    onEnrolled: () => void;
}) => {
    const [enrollTargets, setEnrollTargets] = useState<PlatformOption[]>(PLATFORM_OPTIONS);
    const [enrollUpstream, setEnrollUpstream] = useState(DEFAULT_UPSTREAM);
    const [enrollPollInterval, setEnrollPollInterval] = useState(DEFAULT_POLL_INTERVAL);
    const [enrollError, setEnrollError] = useState<string | null>(null);
    const [enrollResult, setEnrollResult] = useState<{ identifier: string; config: string } | null>(null);
    const [registerBuilder, { loading: enrollSubmitting }] = useMutation(REGISTER_BUILDER);

    const resetEnrollForm = () => {
        setEnrollTargets(PLATFORM_OPTIONS);
        setEnrollUpstream(DEFAULT_UPSTREAM);
        setEnrollPollInterval(DEFAULT_POLL_INTERVAL);
        setEnrollError(null);
        setEnrollResult(null);
    };
    const closeModal = () => {
        onClose();
        resetEnrollForm();
    };
    // Mirrors the Builder ent schema's poll_interval validation (Min(1)) --
    // checked client-side so a bad value fails on the "Enroll" button
    // rather than round-tripping to the server first.
    const pollIntervalValid = Number.isInteger(enrollPollInterval) && enrollPollInterval >= 1;
    const submitEnroll = async () => {
        if (enrollTargets.length === 0 || !pollIntervalValid || enrollSubmitting) return;
        setEnrollError(null);
        try {
            const { data } = await registerBuilder({ variables: { input: {
                supportedTargets: enrollTargets.map((target) => target.value),
                upstream: enrollUpstream.trim() || undefined,
                pollInterval: enrollPollInterval,
            } } });
            if (!data?.registerBuilder) throw new Error("The server did not return a builder.");
            setEnrollResult({ identifier: data.registerBuilder.builder.identifier, config: data.registerBuilder.config });
            onEnrolled();
        } catch (error) {
            setEnrollError(error instanceof Error ? error.message : String(error));
        }
    };

    return (
        <Modal isOpen={isOpen} setOpen={(open) => { if (!open) closeModal(); }} size="sm">
            <div className="flex flex-col gap-6">
                <div>
                    <h3 className="text-xl font-semibold leading-6 text-gray-900">Enroll builder</h3>
                    <p className="mt-1 text-sm text-gray-500">
                        Register a new builder and generate the config it needs to authenticate with Tavern.
                    </p>
                </div>

                {enrollError && <AlertError label="Unable to enroll builder" details={enrollError} />}

                {enrollResult ? (
                    <div className="flex flex-col gap-3">
                        <p className="text-sm text-amber-700">
                            Save this configuration now -- its mTLS credentials are only shown once.
                        </p>
                        <CodeBlock code={enrollResult.config} language="yaml" showCopyButton />
                        <div className="flex flex-row gap-2">
                            <Button onClick={() => downloadBuilderConfig(enrollResult.config, enrollResult.identifier)}>
                                Download config
                            </Button>
                            <Button buttonVariant="ghost" onClick={closeModal}>Done</Button>
                        </div>
                    </div>
                ) : (
                    <div className="flex flex-col gap-4">
                        <div className="flex flex-col gap-1">
                            <label className="block text-sm font-medium text-gray-900">Supported platforms</label>
                            <Select<PlatformOption, true>
                                isMulti
                                options={PLATFORM_OPTIONS}
                                value={enrollTargets}
                                onChange={(selected: MultiValue<PlatformOption>) => setEnrollTargets([...selected])}
                            />
                        </div>
                        <FormTextField
                            htmlFor="builderUpstream"
                            label="Upstream"
                            placeholder={DEFAULT_UPSTREAM}
                            value={enrollUpstream}
                            onChange={(event) => setEnrollUpstream(event.target.value)}
                        />
                        <FormTextField
                            htmlFor="builderPollInterval"
                            label="Poll interval (s)"
                            type="number"
                            min={1}
                            value={enrollPollInterval}
                            onChange={(event) => setEnrollPollInterval(Number(event.target.value))}
                        />
                        <div className="flex flex-row gap-2">
                            <Button buttonVariant="ghost" onClick={closeModal}>Cancel</Button>
                            <Button onClick={submitEnroll} isLoading={enrollSubmitting} disabled={enrollTargets.length === 0 || !pollIntervalValid}>
                                Enroll
                            </Button>
                        </div>
                    </div>
                )}
            </div>
        </Modal>
    );
};

export default EnrollBuilderModal;
