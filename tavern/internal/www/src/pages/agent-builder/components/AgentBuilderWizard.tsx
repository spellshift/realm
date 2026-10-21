import { useMemo, useState } from "react";
import { useNavigate } from "react-router-dom";
import { Heading, useToast } from "@chakra-ui/react";
import { RadioGroup } from "@headlessui/react";
import Select, { createFilter, MultiValue } from "react-select";
import Breadcrumbs from "../../../components/Breadcrumbs";
import Button from "../../../components/tavern-base-ui/button/Button";
import AlertError from "../../../components/tavern-base-ui/AlertError";
import { useMutation } from "@apollo/client";
import { CREATE_BUILD_TASK, useBuildProfiles, useBuilders } from "../queries";
import FreeTextSearch from "../../../components/tavern-base-ui/FreeTextSearch";
import { ProfileSelectionTable } from "./ProfileSelectionTable";
import { EmptyState, EmptyStateType } from "../../../components/tavern-base-ui/EmptyState";
import { ChevronDownIcon, ChevronRightIcon } from "@heroicons/react/24/outline";
import CodeBlock from "../../../components/tavern-base-ui/CodeBlock";
import { FormSteps } from "../../../components/form-steps";
import { TomeTactic } from "../../../utils/enums";
import { classNames } from "../../../utils/utils";
import { COMPILE_TARGETS, TRANSPORT_TYPES, CompileTarget, BuildProfile } from "../types";

const STEPS = [
    { name: "Select profile", description: "Step 1", href: "#", step: 0 },
    { name: "Select target", description: "Step 2", href: "#", step: 1 },
    { name: "Confirm details", description: "Step 3", href: "#", step: 2 },
];

type ProfileFilter = { label: string; value: string; kind: "os" | "format" | "tomeType" };

const OS_OPTIONS: ProfileFilter[] = [
    ...Array.from(new Map(COMPILE_TARGETS.map((target) => [target.platform, {
        label: target.platformLabel,
        value: target.platform,
        kind: "os" as const,
    }])).values()),
];
const FORMAT_OPTIONS: ProfileFilter[] = [
    ...Array.from(new Map(COMPILE_TARGETS.map((target) => [target.format, {
        label: target.formatLabel,
        value: target.format,
        kind: "format" as const,
    }])).values()),
];
const TOME_TYPE_OPTIONS: ProfileFilter[] = Object.entries(TomeTactic).map(([value, label]) => ({
    label, value, kind: "tomeType",
}));
const PROFILE_FILTER_OPTIONS = [
    { label: "OS", options: OS_OPTIONS },
    { label: "Target format", options: FORMAT_OPTIONS },
    { label: "Embedded tome type", options: TOME_TYPE_OPTIONS },
];

export type AgentBuilderWizardProps = {
    showHeader?: boolean;
    onComplete?: (taskId: string) => void;
    onCancel?: () => void;
};

export const AgentBuilderWizard = ({
    showHeader = true,
    onComplete,
    onCancel,
}: AgentBuilderWizardProps) => {
    const navigate = useNavigate();
    const toast = useToast();
    const [currStep, setCurrStep] = useState(0);
    const { profiles, loading, error: profilesError } = useBuildProfiles();
    const { builders, error: buildersError } = useBuilders();
    const [selectedProfileId, setSelectedProfileId] = useState<string | null>(null);
    const selectedProfile = profiles.find((profile) => profile.id === selectedProfileId) ?? profiles[0] ?? null;
    const setSelectedProfile = (profile: BuildProfile) => setSelectedProfileId(profile.id);
    const [createTask, { loading: creating }] = useMutation(CREATE_BUILD_TASK);
    const [error, setError] = useState<string | null>(null);
    const [selectedTarget, setSelectedTarget] = useState<CompileTarget>(COMPILE_TARGETS[0]);
    const targetAvailable = builders.some((builder) => builder.online && builder.supportedTargets.includes(selectedTarget.platform));
    const [showAdvanced, setShowAdvanced] = useState(false);
    const [expandedTransports, setExpandedTransports] = useState<Record<number, boolean>>({});
    const toggleTransportExpanded = (index: number) => {
        setExpandedTransports((prev) => ({
            ...prev,
            [index]: !prev[index],
        }));
    };
    const [profileSearch, setProfileSearch] = useState("");
    const [profileFields, setProfileFields] = useState<ProfileFilter[]>([]);
    const [searchKey, setSearchKey] = useState(0);
    const selectedOS = profileFields.filter((field) => field.kind === "os").map((field) => field.value);
    const selectedFormats = profileFields.filter((field) => field.kind === "format").map((field) => field.value);
    const selectedTomeTypes = profileFields.filter((field) => field.kind === "tomeType").map((field) => field.value);
    const targetMatchesFilters = (target: CompileTarget) =>
        (selectedOS.length === 0 || selectedOS.includes(target.platform)) &&
        (selectedFormats.length === 0 || selectedFormats.includes(target.format));

    const clearProfileFilters = () => {
        setProfileSearch("");
        setProfileFields([]);
        setSearchKey((key) => key + 1);
    };

    const query = profileSearch.trim().toLocaleLowerCase();
    const filteredProfiles = profiles.filter((profile) => {
        if (query && !profile.name.toLocaleLowerCase().includes(query) && !profile.description.toLocaleLowerCase().includes(query)) return false;
        if (selectedTomeTypes.length > 0 && !profile.tomes.some((tome) => selectedTomeTypes.includes(tome.tactic))) return false;
        return true;
    });
    const visibleSelectedProfile = filteredProfiles.find((profile) => profile.id === selectedProfile?.id) ?? filteredProfiles[0] ?? null;
    const hasProfileFilters = !!profileSearch.trim() || profileFields.length > 0;

    const targetsByPlatform = useMemo(() => {
        const groups: Array<{ platformLabel: string; targets: CompileTarget[] }> = [];
        COMPILE_TARGETS.forEach((target) => {
            let group = groups.find((g) => g.platformLabel === target.platformLabel);
            if (!group) {
                group = { platformLabel: target.platformLabel, targets: [] };
                groups.push(group);
            }
            group.targets.push(target);
        });
        return groups;
    }, []);

    const startBuild = async () => {
        if (!selectedProfile || creating) return;
        setError(null);
        try {
            const { data } = await createTask({ variables: { input: {
                profileID: selectedProfile.id,
                targetOS: selectedTarget.platform,
                targetFormat: selectedTarget.format,
            } } });
            if (!data?.createBuildTask?.id) throw new Error("The server did not return a build task.");
            toast({
                title: "Agent build started",
                description: `Build task #${data.createBuildTask.id} queued. The generated artifact will appear in Assets once built.`,
                status: "success",
                duration: 5000,
                isClosable: true,
            });
            if (onComplete) {
                onComplete(data.createBuildTask.id);
            } else {
                navigate("/assets");
            }
        } catch (error) {
            setError(error instanceof Error ? error.message : String(error));
        }
    };

    return (
        <div className="max-w-4xl mx-auto w-full flex flex-col gap-6">
            {showHeader && (
                <Breadcrumbs
                    pages={[
                        { label: "Assets", link: "/assets" },
                        { label: "Generate agent", link: "/assets/build" },
                    ]}
                />
            )}

            {profilesError && <AlertError label="Unable to load build profiles" details={profilesError.message} />}
            {buildersError && <AlertError label="Unable to load builders" details={buildersError.message} />}
            {error && <AlertError label="Unable to start build" details={error} />}
            {loading && profiles.length === 0 && <p role="status">Loading build profiles…</p>}
            <form className="flex flex-col gap-4" onSubmit={(event) => { event.preventDefault(); if (currStep === 2) void startBuild(); }}>
                <div className="flex flex-row gap-6">
                    <div className="flex-shrink-0 mt-2 lg:block hidden">
                        <FormSteps steps={STEPS} currStep={currStep} />
                    </div>
                    <div className="flex-1 min-w-0">
                        {currStep === 0 && (
                            <div className="flex flex-col gap-6">
                                <div className="flex flex-col gap-1">
                                    <h2 className="text-xl font-semibold text-gray-900">Select build profile</h2>
                                    <p className="text-sm text-gray-700 italic">
                                        Select a profile to configure your agent build
                                    </p>
                                </div>
                                <div className="flex flex-col gap-2">
                                    <div className="grid grid-cols-2 gap-2">
                                        <div className="min-w-0">
                                            <FreeTextSearch
                                                key={searchKey}
                                                placeholder="Profile name or description"
                                                defaultValue={profileSearch}
                                                setSearch={setProfileSearch}
                                            />
                                        </div>
                                        <div className="relative z-20 flex flex-col gap-1 min-w-0">
                                            <div className="flex items-center justify-between gap-2">
                                                <label className="font-medium text-gray-700">Profile fields</label>
                                                {hasProfileFilters && (
                                                    <Button buttonVariant="ghost" buttonStyle={{ color: "purple", size: "sm" }} onClick={clearProfileFilters}>
                                                        Clear filters
                                                    </Button>
                                                )}
                                            </div>
                                            <Select<ProfileFilter, true>
                                                isSearchable
                                                isMulti
                                                options={PROFILE_FILTER_OPTIONS}
                                                onChange={(fields: MultiValue<ProfileFilter>) => setProfileFields([...fields])}
                                                filterOption={createFilter({ matchFrom: "any", stringify: (option) => option.label })}
                                                value={profileFields}
                                            />
                                        </div>
                                    </div>
                                    {filteredProfiles.length === 0 ? (
                                        <div className="rounded-md border border-gray-200 py-6">
                                            <EmptyState
                                                type={profiles.length ? EmptyStateType.noMatches : EmptyStateType.noData}
                                                label={profiles.length ? "No matching profiles" : "No build profiles available"}
                                                details={profiles.length ? "Try a different search or clear the filters." : "Please ask an admin to add a profile in the admin panel."}
                                            />
                                        </div>
                                    ) : (
                                        <ProfileSelectionTable
                                            profiles={filteredProfiles}
                                            selectedProfileId={visibleSelectedProfile?.id ?? null}
                                            onSelectProfile={setSelectedProfile}
                                        />
                                    )}
                                </div>
                                <div className="flex flex-row gap-2">
                                    <Button
                                        buttonStyle={{ color: "gray", size: "md" }}
                                        onClick={() => {
                                            if (onCancel) {
                                                onCancel();
                                            } else {
                                                navigate("/assets");
                                            }
                                        }}
                                    >
                                        Cancel
                                    </Button>
                                    <Button
                                        buttonStyle={{ color: "purple", size: "md" }}
                                        disabled={!visibleSelectedProfile}
                                        onClick={() => {
                                            if (!visibleSelectedProfile) return;
                                            const matchingTarget = COMPILE_TARGETS.find(targetMatchesFilters);
                                            if (matchingTarget && (selectedOS.length > 0 || selectedFormats.length > 0)) {
                                                setSelectedTarget(matchingTarget);
                                            }
                                            setSelectedProfile(visibleSelectedProfile);
                                            setCurrStep(1);
                                        }}
                                    >
                                        Continue
                                    </Button>
                                </div>
                            </div>
                        )}

                        {currStep === 1 && selectedProfile && (
                            <div className="flex flex-col gap-6">
                                <div className="flex flex-col gap-1">
                                    <h2 className="text-xl font-semibold text-gray-900">Select compilation target</h2>
                                    <p className="text-sm text-gray-700 italic">
                                        Select an operating system and target format
                                    </p>
                                </div>
                                <RadioGroup value={selectedTarget} onChange={setSelectedTarget}>
                                    <div className="flex flex-col gap-5 max-w-md">
                                        {targetsByPlatform.map((group) => (
                                             <div key={group.platformLabel} className="flex flex-col gap-2">
                                                <span className="text-xs font-semibold uppercase tracking-wide text-gray-500">
                                                    {group.platformLabel}
                                                </span>
                                                <div className="-space-y-px rounded-md bg-white flex flex-col">
                                                    {group.targets.map((target, idx) => (
                                                        <RadioGroup.Option
                                                            key={target.id}
                                                            value={target}
                                                            className={({ checked }) =>
                                                                classNames(
                                                                    idx === 0 ? "rounded-tl-md rounded-tr-md" : "",
                                                                    idx === group.targets.length - 1 ? "rounded-bl-md rounded-br-md" : "",
                                                                    checked ? "z-10 border-purple-200 bg-purple-50" : "border-gray-200",
                                                                    "relative flex cursor-pointer border p-3 focus:outline-none"
                                                                )
                                                            }
                                                        >
                                                            {({ checked }) => (
                                                                <>
                                                                    <span
                                                                        className={classNames(
                                                                            checked ? "bg-purple-700 border-transparent" : "bg-white border-gray-300",
                                                                            "mt-0.5 h-4 w-4 shrink-0 cursor-pointer rounded-full border flex items-center justify-center"
                                                                        )}
                                                                        aria-hidden="true"
                                                                    >
                                                                        <span className="rounded-full bg-white w-1.5 h-1.5" />
                                                                    </span>
                                                                    <span className="ml-3 flex flex-col">
                                                                        <RadioGroup.Label
                                                                            as="span"
                                                                            className={classNames(checked ? "text-purple-900" : "text-gray-900", "block text-sm font-medium")}
                                                                        >
                                                                            {target.formatLabel}
                                                                        </RadioGroup.Label>
                                                                        <RadioGroup.Description
                                                                            as="span"
                                                                            className={classNames(checked ? "text-purple-700" : "text-gray-500", "block text-sm")}
                                                                        >
                                                                            {target.description}
                                                                        </RadioGroup.Description>
                                                                    </span>
                                                                </>
                                                            )}
                                                        </RadioGroup.Option>
                                                    ))}
                                                </div>
                                            </div>
                                        ))}
                                    </div>
                                </RadioGroup>

                                <div className="flex flex-row gap-2">
                                    <Button buttonVariant="ghost" onClick={() => setCurrStep(0)}>Back</Button>
                                    <Button buttonStyle={{ color: "purple", size: "md" }} onClick={() => setCurrStep(2)}>
                                        Continue
                                    </Button>
                                </div>
                            </div>
                        )}

                        {currStep === 2 && selectedProfile && (
                            <div className="flex flex-col gap-6">
                                <h2 className="text-xl font-semibold text-gray-900">Confirm build details</h2>

                                <div className="rounded-md border border-gray-200 divide-y divide-gray-200 text-sm">
                                    <div className="p-3 flex justify-between">
                                        <span className="text-gray-500">Profile</span>
                                        <span className="font-medium text-gray-900">{selectedProfile.name}</span>
                                    </div>
                                    <div className="p-3 flex justify-between">
                                        <span className="text-gray-500">Target</span>
                                        <span className="font-medium text-gray-900">
                                            {selectedTarget.platformLabel} ({selectedTarget.formatLabel})
                                        </span>
                                    </div>
                                    <div className="p-3 flex justify-between">
                                        <span className="text-gray-500">Build image</span>
                                        <code className="text-xs font-mono text-gray-900">{selectedProfile.buildImage}</code>
                                    </div>
                                    <div className="p-3 flex flex-col gap-2">
                                        <div className="flex justify-between items-center">
                                            <span className="text-gray-500">Transports</span>
                                            <span className="text-xs text-gray-500">
                                                {selectedProfile.transports.length} configured
                                            </span>
                                        </div>
                                        {selectedProfile.transports.length === 0 ? (
                                            <span className="text-gray-400 text-xs italic">None</span>
                                        ) : (
                                            <div className="max-h-48 overflow-y-auto flex flex-col gap-2 pr-1">
                                                {selectedProfile.transports.map((transport, idx) => {
                                                    const isExpanded = !!expandedTransports[idx];
                                                    const typeLabel =
                                                        TRANSPORT_TYPES.find((t) => t.value === transport.type)?.label ??
                                                        transport.type;
                                                    return (
                                                        <div
                                                            key={transport.id || idx}
                                                            className="rounded border border-gray-200 bg-gray-50/50 text-xs overflow-hidden"
                                                        >
                                                            <button
                                                                type="button"
                                                                className="w-full p-2.5 flex items-center justify-between hover:bg-gray-100 transition-colors text-left"
                                                                onClick={() => toggleTransportExpanded(idx)}
                                                                aria-expanded={isExpanded}
                                                                aria-label={`Toggle details for ${typeLabel} transport`}
                                                            >
                                                                <div className="flex items-center gap-2 min-w-0">
                                                                    {isExpanded ? (
                                                                        <ChevronDownIcon className="w-4 h-4 text-gray-500 flex-shrink-0" />
                                                                    ) : (
                                                                        <ChevronRightIcon className="w-4 h-4 text-gray-500 flex-shrink-0" />
                                                                    )}
                                                                    <span className="font-semibold text-gray-900">{typeLabel}</span>
                                                                    <span className="text-gray-500 font-mono truncate">{transport.uri}</span>
                                                                </div>
                                                                <span className="text-gray-500 flex-shrink-0 ml-2">{transport.interval}s interval</span>
                                                            </button>
                                                            {isExpanded && (
                                                                <div className="p-2.5 pt-2 border-t border-gray-200 bg-white flex flex-col gap-1.5">
                                                                    <div className="flex justify-between items-start gap-2">
                                                                        <span className="text-gray-500 flex-shrink-0">URI:</span>
                                                                        <span className="font-mono text-gray-900 break-all text-right">{transport.uri}</span>
                                                                    </div>
                                                                    <div className="flex justify-between items-center gap-2">
                                                                        <span className="text-gray-500 flex-shrink-0">Beacon interval:</span>
                                                                        <span className="text-gray-900">{transport.interval} seconds</span>
                                                                    </div>
                                                                    {transport.extra && (
                                                                        <div className="flex justify-between items-start gap-2">
                                                                            <span className="text-gray-500 flex-shrink-0">Extra:</span>
                                                                            <span className="font-mono text-gray-900 break-all text-right">{transport.extra}</span>
                                                                        </div>
                                                                    )}
                                                                </div>
                                                            )}
                                                        </div>
                                                    );
                                                })}
                                            </div>
                                        )}
                                    </div>
                                    <div className="p-3 flex justify-between">
                                        <span className="text-gray-500">Embedded tomes</span>
                                        <div className="flex flex-col gap-1 items-end">
                                            {selectedProfile.tomes.length === 0 ? (
                                                <span className="text-gray-400">None</span>
                                            ) : (
                                                selectedProfile.tomes.map((tome) => (
                                                    <span key={tome.tomeId} className="font-medium text-gray-900">
                                                        {tome.tomeName}
                                                    </span>
                                                ))
                                            )}
                                        </div>
                                    </div>
                                </div>

                                <div className="flex flex-col gap-2">
                                    <button
                                        type="button"
                                        className="text-sm font-medium text-purple-700 text-left"
                                        onClick={() => setShowAdvanced((v) => !v)}
                                    >
                                        {showAdvanced ? "Hide" : "Show"} advanced build script options
                                    </button>
                                    {showAdvanced && (
                                        <div className="flex flex-col gap-3">
                                            <Heading size="sm">Build script</Heading>
                                            <CodeBlock code={selectedProfile.buildScript} language="bash" showCopyButton />
                                            <Heading size="sm">Artifact path</Heading>
                                            <CodeBlock code={selectedProfile.artifactPath} language="bash" />
                                        </div>
                                    )}
                                </div>

                                <div className="flex flex-row gap-2">
                                    <Button buttonVariant="ghost" onClick={() => setCurrStep(1)} disabled={creating} aria-label="back button">
                                        Back
                                    </Button>
                                    <Button
                                        type="submit"
                                        buttonStyle={{ color: "purple", size: "md" }}
                                        isLoading={creating}
                                        disabled={!targetAvailable}
                                        aria-label="submit agent build"
                                    >
                                        Submit
                                    </Button>
                                </div>
                            </div>
                        )}
                    </div>
                </div>
            </form>
        </div>
    );
};

export default AgentBuilderWizard;
