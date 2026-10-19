import { useMemo, useState } from "react";
import { useNavigate, useSearchParams } from "react-router-dom";
import { Heading, Tooltip } from "@chakra-ui/react";
import { RadioGroup } from "@headlessui/react";
import { ExclamationTriangleIcon, PlusIcon } from "@heroicons/react/24/outline";
import Select, { createFilter, MultiValue } from "react-select";
import Breadcrumbs from "../../components/Breadcrumbs";
import Button from "../../components/tavern-base-ui/button/Button";
import FormTextField from "../../components/tavern-base-ui/FormTextField";
import FreeTextSearch from "../../components/tavern-base-ui/FreeTextSearch";
import { FormRadioGroup } from "../../components/tavern-base-ui/FormRadioGroup";
import { EmptyState, EmptyStateType } from "../../components/tavern-base-ui/EmptyState";
import Badge from "../../components/tavern-base-ui/badge/Badge";
import CodeBlock from "../../components/tavern-base-ui/CodeBlock";
import { FormSteps } from "../../components/form-steps";
import { TomeTactic } from "../../utils/enums";
import { MOCK_PROFILES, MOCK_COMPILE_TARGETS, MOCK_TRANSPORT_TYPES, MOCK_TOME_CATALOG, MockCompileTarget, MockBuildProfile, NO_INSTALL_WARNING } from "./mockData";
import NewBuildProfile from "./NewBuildProfile";

function classNames(...classes: string[]) {
    return classes.filter(Boolean).join(" ");
}

const STEPS = [
    { name: "Choose profile", description: "Step 1", href: "#", step: 0 },
    { name: "Target", description: "Step 2", href: "#", step: 1 },
    { name: "Confirm", description: "Step 3", href: "#", step: 2 },
];

type ProfileFilter = { label: string; value: string; kind: "os" | "format" | "tomeType" };

const OS_OPTIONS: ProfileFilter[] = [
    ...Array.from(new Map(MOCK_COMPILE_TARGETS.map((target) => [target.platform, {
        label: target.platformLabel,
        value: target.platform,
        kind: "os" as const,
    }])).values()),
];
const FORMAT_OPTIONS: ProfileFilter[] = [
    ...Array.from(new Map(MOCK_COMPILE_TARGETS.map((target) => [target.format, {
        label: target.formatLabel,
        value: target.format,
        kind: "format" as const,
    }])).values()),
];
const TOME_TYPE_OPTIONS: ProfileFilter[] = [
    ...Array.from(new Map(MOCK_TOME_CATALOG.map((tome) => [tome.tactic, {
        label: TomeTactic[tome.tactic],
        value: tome.tactic,
        kind: "tomeType" as const,
    }])).values()),
];
const PROFILE_FILTER_OPTIONS = [
    { label: "OS", options: OS_OPTIONS },
    { label: "Target format", options: FORMAT_OPTIONS },
    { label: "Embedded tome type", options: TOME_TYPE_OPTIONS },
];

/**
 * Standalone agent builder wizard, reached from the landing page's
 * "Generate agent" action.
 */
export const AgentBuilderWizardPage = () => {
    const navigate = useNavigate();
    const [searchParams] = useSearchParams();
    const [currStep, setCurrStep] = useState(0);
    const [profiles, setProfiles] = useState<MockBuildProfile[]>(MOCK_PROFILES);
    const [selectedProfile, setSelectedProfile] = useState(MOCK_PROFILES[0]);
    const [selectedTarget, setSelectedTarget] = useState<MockCompileTarget>(MOCK_COMPILE_TARGETS[0]);
    const [showAdvanced, setShowAdvanced] = useState(false);
    const [authoring, setAuthoring] = useState(searchParams.get("authoring") === "1");
    const [profileSearch, setProfileSearch] = useState("");
    const [profileFields, setProfileFields] = useState<ProfileFilter[]>([]);
    const [searchKey, setSearchKey] = useState(0);
    const selectedOS = profileFields.filter((field) => field.kind === "os").map((field) => field.value);
    const selectedFormats = profileFields.filter((field) => field.kind === "format").map((field) => field.value);
    const selectedTomeTypes = profileFields.filter((field) => field.kind === "tomeType").map((field) => field.value);
    const targetMatchesFilters = (target: MockCompileTarget) =>
        (selectedOS.length === 0 || selectedOS.includes(target.platform)) &&
        (selectedFormats.length === 0 || selectedFormats.includes(target.format));

    const clearProfileFilters = () => {
        setProfileSearch("");
        setProfileFields([]);
        setSearchKey((key) => key + 1);
    };

    const handleSaveProfile = (profile: MockBuildProfile) => {
        setProfiles((p) => [...p, profile]);
        setSelectedProfile(profile);
        clearProfileFilters();
        setAuthoring(false);
    };

    const query = profileSearch.trim().toLocaleLowerCase();
    const filteredProfiles = profiles.filter((profile) => {
        if (query && !profile.name.toLocaleLowerCase().includes(query) && !profile.description.toLocaleLowerCase().includes(query)) return false;
        if (!profile.supportedTargets.some((targetId) => {
            const target = MOCK_COMPILE_TARGETS.find((item) => item.id === targetId);
            return target && targetMatchesFilters(target);
        })) return false;
        if (selectedTomeTypes.length > 0 && !profile.tomes.some((tome) => selectedTomeTypes.includes(tome.tactic))) return false;
        return true;
    });
    const visibleSelectedProfile = filteredProfiles.find((profile) => profile.id === selectedProfile.id) ?? filteredProfiles[0] ?? null;
    const hasProfileFilters = !!profileSearch.trim() || profileFields.length > 0;

    const targetsByPlatform = useMemo(() => {
        const groups: Array<{ platformLabel: string; targets: MockCompileTarget[] }> = [];
        MOCK_COMPILE_TARGETS.filter((target) => selectedProfile.supportedTargets.includes(target.id)).forEach((target) => {
            let group = groups.find((g) => g.platformLabel === target.platformLabel);
            if (!group) {
                group = { platformLabel: target.platformLabel, targets: [] };
                groups.push(group);
            }
            group.targets.push(target);
        });
        return groups;
    }, [selectedProfile]);

    return (
        <div className="max-w-3xl mx-auto w-full flex flex-col gap-4">
            <Breadcrumbs
                pages={[
                    { label: "Agent Builder", link: "/scratch-agent-builder" },
                    { label: "Generate agent", link: "/scratch-agent-builder/new" },
                ]}
            />

            <form className="flex flex-col gap-4">
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
                                        <EmptyState type={EmptyStateType.noMatches} label="No matching profiles" details="Try a different search or clear the filters." />
                                    </div>
                                ) : (
                                    <FormRadioGroup
                                        label="Build profile"
                                        data={filteredProfiles}
                                        selected={visibleSelectedProfile}
                                        setSelected={setSelectedProfile}
                                        renderNameAddon={(profile: MockBuildProfile) => !profile.hasPersistenceTome && (
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
                                    <Button disabled={!visibleSelectedProfile} onClick={() => {
                                        if (!visibleSelectedProfile) return;
                                        const matchingTarget = MOCK_COMPILE_TARGETS.find((target) =>
                                            visibleSelectedProfile.supportedTargets.includes(target.id) &&
                                            targetMatchesFilters(target)
                                        );
                                        if (matchingTarget && (selectedOS.length > 0 || selectedFormats.length > 0 || !visibleSelectedProfile.supportedTargets.includes(selectedTarget.id))) {
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
                                onCancel={() => setAuthoring(false)}
                                onSave={handleSaveProfile}
                            />
                        )}

                        {currStep === 1 && (
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
                                <div className="flex flex-row gap-2">
                                    <Button buttonVariant="ghost" onClick={() => setCurrStep(0)}>Back</Button>
                                    <Button onClick={() => setCurrStep(2)}>Continue</Button>
                                </div>
                            </div>
                        )}

                        {currStep === 2 && (
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
                                                                {MOCK_TRANSPORT_TYPES.find((type) => type.value === transport.type)?.label ?? transport.type}
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

                                <FormTextField
                                    htmlFor="buildName"
                                    label="Build name"
                                    placeholder="Provide a recognizable name for this build"
                                    defaultValue="agile-falcon-build"
                                />

                                <div className="flex flex-row gap-2">
                                    <Button buttonVariant="ghost" onClick={() => setCurrStep(1)}>Back</Button>
                                    <Button type="submit" onClick={(e) => { e.preventDefault(); navigate("/scratch-agent-builder"); }}>
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

export default AgentBuilderWizardPage;
