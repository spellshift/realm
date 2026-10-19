import { useCallback, useState } from "react";
import { TabGroup, TabList, Tab, TabPanels, TabPanel } from "@headlessui/react";
import { useSearchParams, useNavigate } from "react-router-dom";
import { Hammer, Settings2, PackageCheck, FileTerminal } from "lucide-react";
import { useMutation } from "@apollo/client";
import Select, { MultiValue } from "react-select";
import Breadcrumbs from "../../components/Breadcrumbs";
import Table from "../../components/tavern-base-ui/table/Table";
import { EmptyState, EmptyStateType } from "../../components/tavern-base-ui/EmptyState";
import Button from "../../components/tavern-base-ui/button/Button";
import AlertError from "../../components/tavern-base-ui/AlertError";
import Modal from "../../components/tavern-base-ui/Modal";
import FormTextField from "../../components/tavern-base-ui/FormTextField";
import CodeBlock from "../../components/tavern-base-ui/CodeBlock";
import { PlusIcon } from "@heroicons/react/24/outline";
import { builderColumns, profileColumns, buildTaskColumns } from "./landingShared";
import { useBuilders, useBuildProfiles, useBuildTasks, REGISTER_BUILDER, downloadBuilderConfig } from "./api";
import CreateLinkModal from "../assets/components/CreateLinkModal/CreateLinkModal";
import { Platform } from "./types";

// Built agents / Build profiles / Builders --
// defaults to Built agents (index 0) when no `tab` param is present.
const TAB_NAMES = ["agents", "profiles", "builders"] as const;

// Enroll builder modal's platform multi-select options -- module-level data,
// not a component, same as COMPILE_TARGETS/TRANSPORT_TYPES in types.ts.
type PlatformOption = { label: string; value: Platform };
const PLATFORM_OPTIONS: PlatformOption[] = [
    { label: "Linux", value: "LINUX" },
    { label: "macOS", value: "MACOS" },
    { label: "Windows", value: "WINDOWS" },
];
const DEFAULT_UPSTREAM = "http://127.0.0.1:8000";
const DEFAULT_POLL_INTERVAL = 5;

const tabClass = ({ selected }: { selected: boolean }) =>
    `p-3 sm:p-4 flex flex-row gap-1 items-center flex-shrink-0 whitespace-nowrap border-t-2 border-l-2 border-r-2 rounded-t-lg ${
        selected
            ? "border-t-purple-600 bg-white text-purple-800 hover:bg-gray-100"
            : "border-transparent hover:bg-white hover:border-t-purple-600"
    }`;

/**
 * Tabbed landing page using the same tab idiom as the host detail page
 * (HostTabs.tsx) -- Headless UI TabGroup/TabList/Tab with folder-style
 * tabs. "Add build profile" / "Enroll builder" now live at the bottom of
 * their own tab's list (ghost link below the table, same pattern as the
 * wizard's profile list) rather than in the shared header, which now only
 * carries "Generate agent". Empty states reuse the FilesTab/ProcessTab
 * "no data" CTA pattern.
 *
 * `?emptydemo=profiles|builders` forces that tab's list empty, purely to
 * screenshot the empty-state CTA -- not real app behavior.
 *
 * Built agents' table shows one row per BuildTask (see buildTaskRows in
 * api.ts) -- queued/claimed/building tasks render with a live status badge
 * and no artifact yet, so there's no separate "build in progress" banner;
 * the table itself is the live status view.
 */
export const AgentBuilderLandingPage = () => {
    const navigate = useNavigate();
    const [selectedAsset, setSelectedAsset] = useState<{ id: string; name: string } | null>(null);
    const [searchParams, setSearchParams] = useSearchParams();
    const tabParam = searchParams.get("tab");
    const emptyDemo = searchParams.get("emptydemo");
    const selectedIndex = Math.max(0, TAB_NAMES.indexOf(tabParam as typeof TAB_NAMES[number]));

    const { profiles, loading: profilesLoading, error: profilesError } = useBuildProfiles();
    const { builders, loading: buildersLoading, error: buildersError, refetch: refetchBuilders } = useBuilders();
    const { rows, loading: tasksLoading, error: tasksError } = useBuildTasks();

    const handleTabChange = (index: number) => {
        setSearchParams({ tab: TAB_NAMES[index] }, { replace: true });
    };

    const visibleProfiles = emptyDemo === "profiles" ? [] : profiles;
    const visibleBuilders = emptyDemo === "builders" ? [] : builders;
    const handleCreateLink = useCallback((id: string, name: string) => {
        setSelectedAsset({ id, name });
    }, []);

    // Enroll builder modal state -- inlined here (rather than a separate
    // component) per the "no new Tavern UI components" rule; it's just
    // Modal + FormTextField + CodeBlock, the same primitives CreateLinkModal
    // composes, wired directly into this page. registerBuilder returns a
    // one-time mTLS cert/config bundle that can't be re-fetched later, so
    // the result view stays up until the operator explicitly closes it.
    const [enrolling, setEnrolling] = useState(false);
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
    const closeEnrollModal = () => {
        setEnrolling(false);
        resetEnrollForm();
    };
    const submitEnroll = async () => {
        if (enrollTargets.length === 0 || enrollSubmitting) return;
        setEnrollError(null);
        try {
            const { data } = await registerBuilder({ variables: { input: {
                supportedTargets: enrollTargets.map((target) => `PLATFORM_${target.value}`),
                upstream: enrollUpstream.trim() || undefined,
                pollInterval: enrollPollInterval,
            } } });
            if (!data?.registerBuilder) throw new Error("The server did not return a builder.");
            setEnrollResult({ identifier: data.registerBuilder.builder.identifier, config: data.registerBuilder.config });
            void refetchBuilders();
        } catch (error) {
            setEnrollError(error instanceof Error ? error.message : String(error));
        }
    };

    return (
        <div className="flex flex-col gap-4 w-full min-w-0">
            <div className="flex flex-col sm:flex-row gap-3 justify-between w-full min-w-0 items-start sm:items-center">
                <Breadcrumbs pages={[{ label: "Agent Builder", link: "/scratch-agent-builder" }]} />
                <Button
                    leftIcon={<FileTerminal className="w-5 h-5" />}
                    buttonStyle={{ color: "purple", size: "md" }}
                    onClick={() => navigate("/scratch-agent-builder/new")}
                >
                    Generate agent
                </Button>
            </div>

            <TabGroup selectedIndex={selectedIndex} onChange={handleTabChange}>
                <TabList className="flex flex-row gap-2 border-gray-200 w-full min-w-0 overflow-x-auto bg-gray-100">
                    <Tab className={tabClass}>
                        <PackageCheck className="w-4 h-4" />
                        <div>Built agents</div>
                        <div>({rows.length})</div>
                    </Tab>
                    <Tab className={tabClass}>
                        <Settings2 className="w-4 h-4" />
                        <div>Build profiles</div>
                        <div>({visibleProfiles.length})</div>
                    </Tab>
                    <Tab className={tabClass}>
                        <Hammer className="w-4 h-4" />
                        <div>Builders</div>
                        <div>({visibleBuilders.length})</div>
                    </Tab>
                </TabList>
                <TabPanels className="min-w-0">
                    <TabPanel>
                        <div className="flex flex-col gap-2">
                            {tasksError && <AlertError label="Unable to load build tasks" details={tasksError.message} />}
                            {tasksLoading && rows.length === 0 ? (
                                <p role="status">Loading built agents…</p>
                            ) : rows.length === 0 ? (
                                <div className="rounded-md border border-gray-200 border-t-0 overflow-hidden">
                                    <EmptyState
                                        type={EmptyStateType.noData}
                                        label="No built agents yet"
                                        details="Generate an agent to get started."
                                    >
                                        <Button
                                            leftIcon={<PlusIcon className="w-4 h-4" />}
                                            buttonStyle={{ color: "purple", size: "md" }}
                                            onClick={() => navigate("/scratch-agent-builder/new")}
                                        >
                                            Generate agent
                                        </Button>
                                    </EmptyState>
                                </div>
                            ) : (
                                <>
                                    <div className="rounded-md border border-gray-200 border-t-0 overflow-x-auto max-w-full">
                                        <div className="min-w-[720px] w-full">
                                            <Table data={rows} columns={buildTaskColumns(handleCreateLink)} />
                                        </div>
                                    </div>
                                    <Button
                                        buttonVariant="ghost"
                                        buttonStyle={{ color: "purple", size: "sm" }}
                                        leftIcon={<PlusIcon className="w-4 h-4" />}
                                        onClick={() => navigate("/scratch-agent-builder/new")}
                                        className="w-fit"
                                    >
                                        Generate agent
                                    </Button>
                                </>
                            )}
                        </div>
                    </TabPanel>

                    <TabPanel>
                        {profilesError && <AlertError label="Unable to load build profiles" details={profilesError.message} />}
                        {profilesLoading && profiles.length === 0 ? (
                            <p role="status">Loading build profiles…</p>
                        ) : visibleProfiles.length === 0 ? (
                            <div className="rounded-md border border-gray-200 border-t-0 overflow-hidden">
                                <EmptyState
                                    type={EmptyStateType.noData}
                                    label="No build profiles available"
                                    details="Add one to get started."
                                >
                                    <Button
                                        leftIcon={<PlusIcon className="w-4 h-4" />}
                                        buttonStyle={{ color: "purple", size: "md" }}
                                        onClick={() => navigate("/scratch-agent-builder/new?authoring=1")}
                                    >
                                        Add build profile
                                    </Button>
                                </EmptyState>
                            </div>
                        ) : (
                            <div className="flex flex-col gap-2">
                                <div className="rounded-md border border-gray-200 border-t-0 overflow-x-auto max-w-full">
                                    <div className="min-w-[720px] w-full">
                                        <Table data={visibleProfiles} columns={profileColumns} />
                                    </div>
                                </div>
                                <Button
                                    buttonVariant="ghost"
                                    buttonStyle={{ color: "purple", size: "sm" }}
                                    leftIcon={<PlusIcon className="w-4 h-4" />}
                                    onClick={() => navigate("/scratch-agent-builder/new?authoring=1")}
                                    className="w-fit"
                                >
                                    Add build profile
                                </Button>
                            </div>
                        )}
                    </TabPanel>

                    <TabPanel>
                        {buildersError && <AlertError label="Unable to load builders" details={buildersError.message} />}
                        {buildersLoading && builders.length === 0 ? (
                            <p role="status">Loading builders…</p>
                        ) : visibleBuilders.length === 0 ? (
                            <div className="rounded-md border border-gray-200 border-t-0 overflow-hidden">
                                <EmptyState
                                    type={EmptyStateType.noData}
                                    label="No builders available"
                                    details="Add one to get started."
                                >
                                    <Button
                                        leftIcon={<PlusIcon className="w-4 h-4" />}
                                        buttonStyle={{ color: "purple", size: "md" }}
                                        onClick={() => setEnrolling(true)}
                                    >
                                        Enroll builder
                                    </Button>
                                </EmptyState>
                            </div>
                        ) : (
                            <div className="flex flex-col gap-2">
                                <div className="rounded-md border border-gray-200 border-t-0 overflow-x-auto max-w-full">
                                    <div className="min-w-[720px] w-full">
                                        <Table data={visibleBuilders} columns={builderColumns} />
                                    </div>
                                </div>
                                <Button
                                    buttonVariant="ghost"
                                    buttonStyle={{ color: "purple", size: "sm" }}
                                    leftIcon={<PlusIcon className="w-4 h-4" />}
                                    onClick={() => setEnrolling(true)}
                                    className="w-fit"
                                >
                                    Enroll builder
                                </Button>
                            </div>
                        )}
                    </TabPanel>
                </TabPanels>
            </TabGroup>
            {selectedAsset && (
                <CreateLinkModal
                    isOpen
                    setOpen={(open) => { if (!open) setSelectedAsset(null); }}
                    assetId={selectedAsset.id}
                    assetName={selectedAsset.name}
                />
            )}
            <Modal isOpen={enrolling} setOpen={closeEnrollModal} size="sm">
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
                                <Button buttonVariant="ghost" onClick={closeEnrollModal}>Done</Button>
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
                                <Button buttonVariant="ghost" onClick={closeEnrollModal}>Cancel</Button>
                                <Button onClick={submitEnroll} isLoading={enrollSubmitting} disabled={enrollTargets.length === 0}>
                                    Enroll
                                </Button>
                            </div>
                        </div>
                    )}
                </div>
            </Modal>
        </div>
    );
};

export default AgentBuilderLandingPage;
