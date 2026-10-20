import { useMemo, useState } from "react";
import { useNavigate, useSearchParams } from "react-router-dom";
import { Heading, Tooltip } from "@chakra-ui/react";
import { RadioGroup } from "@headlessui/react";
import { ExclamationTriangleIcon, PlusIcon } from "@heroicons/react/24/outline";
import Select, { createFilter, MultiValue } from "react-select";
import Breadcrumbs from "../../../components/Breadcrumbs";
import Button from "../../../components/tavern-base-ui/button/Button";
import AlertError from "../../../components/tavern-base-ui/AlertError";
import { useMutation } from "@apollo/client";
import { CREATE_BUILD_TASK, useBuildProfiles, useBuilders } from "../queries";
import FreeTextSearch from "../../../components/tavern-base-ui/FreeTextSearch";
import { FormRadioGroup } from "../../../components/tavern-base-ui/FormRadioGroup";
import { EmptyState, EmptyStateType } from "../../../components/tavern-base-ui/EmptyState";
import Badge from "../../../components/tavern-base-ui/badge/Badge";
import CodeBlock from "../../../components/tavern-base-ui/CodeBlock";
import { FormSteps } from "../../../components/form-steps";
import { TomeTactic } from "../../../utils/enums";
import { classNames } from "../../../utils/utils";
import { COMPILE_TARGETS, TRANSPORT_TYPES, CompileTarget, BuildProfile, NO_INSTALL_WARNING } from "../types";
import NewBuildProfile from "../NewBuildProfile";

const STEPS = [
    { name: "Choose profile", description: "Step 1", href: "#", step: 0 },
    { name: "Target", description: "Step 2", href: "#", step: 1 },
    { name: "Confirm", description: "Step 3", href: "#", step: 2 },
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
    initialAuthoring?: boolean;
};

export const AgentBuilderWizard = ({
    showHeader = true,
    onComplete,
    onCancel,
    initialAuthoring = false,
}: AgentBuilderWizardProps) => {
    const navigate = useNavigate();
    const [searchParams] = useSearchParams();
    const [currStep, setCurrStep] = useState(0);
    const { profiles, tomes, loading, error: profilesError, refetch } = useBuildProfiles();
    const { builders, error: buildersError } = useBuilders();
    const [selectedProfileId, setSelectedProfileId] = useState<string | null>(null);
    const selectedProfile = profiles.find((profile) => profile.id === selectedProfileId) ?? profiles[0] ?? null;
    const setSelectedProfile = (profile: BuildProfile) => setSelectedProfileId(profile.id);
    const [createTask, { loading: creating }] = useMutation(CREATE_BUILD_TASK);
    const [error, setError] = useState<string | null>(null);
    const [selectedTarget, setSelectedTarget] = useState<CompileTarget>(COMPILE_TARGETS[0]);
    const targetAvailable = builders.some((builder) => builder.online && builder.supportedTargets.includes(selectedTarget.platform));
    const [showAdvanced, setShowAdvanced] = useState(false);
    const [authoring, setAuthoring] = useState(initialAuthoring || searchParams.get("authoring") === "1");
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

    const handleSaveProfile = async (profile: BuildProfile) => {
        await refetch();
        setSelectedProfile(profile);
        clearProfileFilters();
        setAuthoring(false);
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
            if (onComplete) {
                onComplete(data.createBuildTask.id);
            } else {
                navigate(`/builder?tab=agents&task=${data.createBuildTask.id}`);
            }
        } catch (error) {
            setError(error instanceof Error ? error.message : String(error));
        }
    };

    return (
        <div className="max-w-3xl mx-auto w-full flex flex-col gap-4">
            {showHeader && (
                <Breadcrumbs
                    pages={[
                        { label: "Agent Builder", link: "/builder" },
                        { label: "Generate agent", link: "/builder/new" },
                    ]}
                />
            )}

            {profilesError && <AlertError label="Unable to load build profiles" details={profilesError.message} />}
            {buildersError && <AlertError label="Unable to load builders" details={buildersError.message} />}
            {error && <AlertError label="Unable to start build" details={error} />}
            {loading && profiles.length === 0 && <p role="status">Loading build profiles…</p>}
            <form className="flex flex-col gap-4" onSubmit={(event) => { event.preventDefault(); if (currStep === 2) void startBuild(); }}>
                <h2 className="text-xl font-semibold text-gray-900">Generate agent</h2>
                <div className="flex flex-row gap-6">
                    <div className="flex-shrink-0 mt-2 lg:block hidden">
                        <FormSteps steps={STEPS} currStep={currStep} />
                    </div>
                    <div className="flex-1 min-w-0">
                        {currStep === 0 && !authoring && (
                            <div className="flex flex-col gap-4">
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
                                        <label className="font-medium text-gray-700">Profile fields</label>
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
                                <div className="flex items-center justify-between gap-2 text-sm text-gray-500">
                                    <span>{filteredProfiles.length} of {profiles.length} profiles</span>
                                    {hasProfileFilters && (
                                        <Button buttonVariant="ghost" buttonStyle={{ color: "purple", size: "sm" }} onClick={clearProfileFilters}>
                                            Clear filters
                                        </Button>
                                    )}
                                </div>
                                {filteredProfiles.length === 0 ? (
                                    <div className="rounded-md border border-gray-200 py-6">
                                        <EmptyState type={EmptyStateType.noMatches} label={profiles.length ? "No matching profiles" : "No build profiles available"} details={profiles.length ? "Try a different search or clear the filters." : "Create a profile to get started."} />
                                    </div>
                                ) : (
                                    <FormRadioGroup
                                        label="Build profile"
                                        data={filteredProfiles}
                                        selected={visibleSelectedProfile}
                                        setSelected={setSelectedProfile}
                                        renderNameAddon={(profile: BuildProfile) => !profile.hasPersistenceTome && (
                                            <Tooltip label={NO_INSTALL_WARNING} hasArrow placement="right" maxW="280px">
                                                <span tabIndex={0} aria-label="No install warning" className="inline-flex text-amber-500">
                                                    <ExclamationTriangleIcon className="h-4 w-4" />
                                                </span>
                                            </Tooltip>
                                        )}
                                    />
                                )}
                                <Button
                                    buttonVariant="ghost"
                                    buttonStyle={{ color: "purple", size: "sm" }}
                                    leftIcon={<PlusIcon className="w-4 h-4" />}
                                    onClick={() => setAuthoring(true)}
                                    className="w-fit"
                                >
                                    New profile
                                </Button>
                                <div className="flex flex-row gap-2">
                                    {onCancel && (
                                        <Button buttonVariant="outline" buttonStyle={{ color: "gray", size: "md" }} onClick={onCancel}>
                                            Cancel
                                        </Button>
                                    )}
                                    <Button disabled={!visibleSelectedProfile} onClick={() => {
                                        if (!visibleSelectedProfile) return;
                                        const matchingTarget = COMPILE_TARGETS.find(targetMatchesFilters);
                                        if (matchingTarget && (selectedOS.length > 0 || selectedFormats.length > 0)) {
                                            setSelectedTarget(matchingTarget);
                                        }
                                        setSelectedProfile(visibleSelectedProfile);
                                        setCurrStep(1);
                                    }}>Continue</Button>
                                </div>
                            </div>
                        )}

                        {currStep === 0 && authoring && (
                            <NewBuildProfile
                                catalog={tomes}
                                onCancel={() => setAuthoring(false)}
                                onSave={handleSaveProfile}
                            />
                        )}

                        {currStep === 1 && selectedProfile && (
                            <div className="flex flex-col gap-6">
                                <RadioGroup value={selectedTarget} onChange={setSelectedTarget}>
                                    <RadioGroup.Label>
                                        <Heading size="sm">Compilation target</Heading>
                                    </RadioGroup.Label>
                                    <div className="mt-4 flex flex-col gap-5 max-w-md">
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
                                {!targetAvailable && <p className="text-sm text-amber-700">Enroll a builder that supports {selectedTarget.platformLabel} before starting this build.</p>}
                                <div className="flex flex-row gap-2">
                                    <Button buttonVariant="ghost" onClick={() => setCurrStep(0)}>Back</Button>
                                    <Button onClick={() => setCurrStep(2)} disabled={!targetAvailable}>Continue</Button>
                                </div>
                            </div>
                        )}

                        {currStep === 2 && selectedProfile && (
                            <div className="flex flex-col gap-6">
                                <div className="flex flex-col gap-3">
                                    <Heading size="sm">Summary</Heading>
                                    <div className="rounded-md border border-gray-200 p-4 flex flex-col gap-2">
                                        <div className="flex flex-row flex-wrap gap-2 items-center">
                                            <Badge badgeStyle={{ color: "purple" }}>{selectedProfile.name}</Badge>
                                            <Badge badgeStyle={{ color: "gray" }}>{selectedTarget.platformLabel}</Badge>
                                            <Badge badgeStyle={{ color: "gray" }}>{selectedTarget.formatLabel}</Badge>
                                        </div>
                                        <p className="text-sm text-gray-500">{selectedProfile.description}</p>
                                        <div className="pt-2">
                                            <h3 className="text-sm font-medium text-gray-900 mb-2">Transports</h3>
                                            <div className="rounded-md border border-gray-200 divide-y divide-gray-200">
                                                {selectedProfile.transports.map((transport) => (
                                                    <div key={transport.id} className="flex flex-row items-start justify-between gap-2 p-3">
                                                        <div className="min-w-0 flex flex-col gap-1">
                                                            <span className="text-sm font-medium text-gray-900">
                                                                {TRANSPORT_TYPES.find((type) => type.value === transport.type)?.label ?? transport.type}
                                                            </span>
                                                            <code className="text-xs text-gray-500 break-all">{transport.uri || "—"}</code>
                                                        </div>
                                                        <Badge badgeStyle={{ color: "gray" }}>
                                                            <span className="sr-only">Callback interval </span>{transport.interval} s
                                                        </Badge>
                                                    </div>
                                                ))}
                                            </div>
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

                                <p className="text-sm text-gray-500">The server generates a unique artifact name for this build.</p>

                                <div className="flex flex-row gap-2">
                                    <Button buttonVariant="ghost" onClick={() => setCurrStep(1)}>Back</Button>
                                    <Button type="submit" isLoading={creating} disabled={!targetAvailable}>
                                        Start build
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
