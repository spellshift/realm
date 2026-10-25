import { useEffect, useState } from "react";
import { useMutation } from "@apollo/client";
import Select, { MultiValue } from "react-select";
import { useNavigate } from "react-router-dom";
import Breadcrumbs from "../../components/Breadcrumbs";
import PageHeader from "../../components/tavern-base-ui/PageHeader";
import Button from "../../components/tavern-base-ui/button/Button";
import AlertError from "../../components/tavern-base-ui/AlertError";
import FormTextField from "../../components/tavern-base-ui/FormTextField";
import CodeBlock from "../../components/tavern-base-ui/CodeBlock";
import { FormSteps } from "../../components/form-steps";
import Table from "../../components/tavern-base-ui/table/Table";
import { builderColumns } from "./builders-tab";
import { REGISTER_BUILDER, downloadBuilderConfig, useEnrolledBuilder } from "./queries";
import { PlatformOption, PLATFORM_OPTIONS } from "./types";

const DEFAULT_UPSTREAM = "http://127.0.0.1:8000";
const STEPS = [
    { name: "Configure builder", description: "Step 1", href: "#", step: 0 },
    { name: "Copy config", description: "Step 2", href: "#", step: 1 },
    { name: "Start builder", description: "Step 3", href: "#", step: 2 },
];

export const EnrollBuilderPage = () => {
    const navigate = useNavigate();
    const [step, setStep] = useState(0);
    const [targets, setTargets] = useState<PlatformOption[]>(PLATFORM_OPTIONS);
    const [upstream, setUpstream] = useState(DEFAULT_UPSTREAM);
    const [pollInterval, setPollInterval] = useState(5);
    const [executor, setExecutor] = useState("docker");
    const [error, setError] = useState<string | null>(null);
    const [result, setResult] = useState<{ identifier: string; config: string } | null>(null);
    const [registerBuilder, { loading }] = useMutation(REGISTER_BUILDER);
    const { builder, online: registered, refetch, error: statusError } = useEnrolledBuilder(result?.identifier ?? null, step !== 2);
    const pollIntervalValid = Number.isInteger(pollInterval) && pollInterval >= 1;

    useEffect(() => {
        if (step !== 2 || !result || registered) return;
        const timer = setInterval(() => { void refetch().catch(() => undefined); }, 5000);
        return () => clearInterval(timer);
    }, [step, result, registered, refetch]);

    const submit = async () => {
        if (targets.length === 0 || !pollIntervalValid || loading) return;
        setError(null);
        try {
            const { data } = await registerBuilder({ variables: { input: {
                supportedTargets: targets.map((target) => target.value),
                upstream: upstream.trim() || undefined,
                pollInterval,
                executor,
            } } });
            if (!data?.registerBuilder) throw new Error("The server did not return a builder.");
            setResult({ identifier: data.registerBuilder.builder.identifier, config: data.registerBuilder.config });
            setStep(1);
        } catch (caught) {
            setError(caught instanceof Error ? caught.message : String(caught));
        }
    };

    return (
        <div className="max-w-4xl mx-auto w-full flex flex-col gap-6">
            <Breadcrumbs pages={[
                { label: "Admin", link: "/admin?tab=builders" },
                { label: "Enroll builder", link: "/admin/builders/enroll" },
            ]} />
            <PageHeader title="Enroll builder" description="Register a builder and start it with its one-time configuration." />
            {error && <AlertError label="Unable to enroll builder" details={error} />}
            <div className="flex flex-row gap-6">
                <div className="flex-shrink-0 mt-2 lg:block hidden"><FormSteps steps={STEPS} currStep={step} /></div>
                <div className="flex-1 min-w-0 flex flex-col gap-4">
                    {step === 0 && (
                        <form className="flex flex-col gap-4" onSubmit={(event) => { event.preventDefault(); void submit(); }}>
                            <h2 className="text-xl font-semibold">Configure builder</h2>
                            <div className="flex flex-col gap-1">
                                <label className="text-sm font-medium text-gray-900">Supported platforms</label>
                                <Select<PlatformOption, true> isMulti options={PLATFORM_OPTIONS} value={targets}
                                    onChange={(selected: MultiValue<PlatformOption>) => setTargets([...selected])} />
                            </div>
                            <FormTextField htmlFor="builderUpstream" label="Upstream" placeholder={DEFAULT_UPSTREAM}
                                value={upstream} onChange={(event) => setUpstream(event.target.value)} />
                            <FormTextField htmlFor="builderPollInterval" label="Poll interval (s)" type="number" min={1}
                                value={pollInterval} onChange={(event) => setPollInterval(Number(event.target.value))} />
                            <div className="flex flex-col gap-1">
                                <label htmlFor="builderExecutor" className="text-sm font-medium text-gray-900">Executor</label>
                                <select id="builderExecutor" className="rounded-md border border-gray-300 p-2" value={executor}
                                    onChange={(event) => setExecutor(event.target.value)}>
                                    <option value="docker">Docker</option>
                                    <option value="local">Local</option>
                                </select>
                                {executor === "local" && <p className="text-xs text-gray-600">Local execution is available on Linux for testing.</p>}
                            </div>
                            <div className="flex gap-2">
                                <Button buttonVariant="ghost" onClick={() => navigate("/admin?tab=builders")}>Cancel</Button>
                                <Button type="submit" buttonStyle={{ color: "purple", size: "md" }} isLoading={loading}
                                    disabled={targets.length === 0 || !pollIntervalValid}>Configure</Button>
                            </div>
                        </form>
                    )}
                    {step === 1 && result && (
                        <div className="flex flex-col gap-4">
                            <h2 className="text-xl font-semibold">Copy config</h2>
                            <p className="text-sm text-amber-700">Save this configuration now. Its mTLS credentials are shown only once.</p>
                            <CodeBlock code={result.config} language="yaml" showCopyButton />
                            <p className="text-sm text-gray-700">Save as <code>builder-{result.identifier}.yaml</code>.</p>
                            <div className="flex gap-2">
                                <Button onClick={() => downloadBuilderConfig(result.config, result.identifier)}>Download config</Button>
                                <Button buttonStyle={{ color: "purple", size: "md" }} onClick={() => setStep(2)}>Next</Button>
                            </div>
                        </div>
                    )}
                    {step === 2 && result && (
                        <div className="flex flex-col gap-4">
                            <h2 className="text-xl font-semibold">Start builder</h2>
                            <p className="text-sm text-gray-700">From the Realm project root, run the builder with the saved config:</p>
                            <CodeBlock code={`go run ./tavern builder --config builder-${result.identifier}.yaml`} language="bash" showCopyButton />
                            {registered && builder ? (
                                <div className="rounded-md border border-gray-200 overflow-x-auto max-w-full">
                                    <div className="min-w-[720px] w-full">
                                        <Table data={[builder]} columns={builderColumns} />
                                    </div>
                                </div>
                            ) : (
                                <p role="status" className="text-sm text-gray-700">Waiting for the builder to connect to Tavern…</p>
                            )}
                            {statusError && <AlertError label="Unable to check builder connection" details={statusError.message} />}
                            <div className="flex gap-2">
                                <Button buttonVariant="ghost" onClick={() => setStep(1)}>Previous</Button>
                                <Button buttonStyle={{ color: "purple", size: "md" }} disabled={!registered}
                                    onClick={() => navigate("/admin?tab=builders")}>Done</Button>
                            </div>
                        </div>
                    )}
                </div>
            </div>
        </div>
    );
};

export default EnrollBuilderPage;
