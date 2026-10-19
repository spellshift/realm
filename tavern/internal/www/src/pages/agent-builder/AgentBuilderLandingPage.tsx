import { useCallback, useState } from "react";
import { TabGroup, TabList, Tab, TabPanels, TabPanel } from "@headlessui/react";
import { useSearchParams, useNavigate } from "react-router-dom";
import { Hammer, Settings2, PackageCheck, FileTerminal } from "lucide-react";
import Breadcrumbs from "../../components/Breadcrumbs";
import Table from "../../components/tavern-base-ui/table/Table";
import { EmptyState, EmptyStateType } from "../../components/tavern-base-ui/EmptyState";
import Button from "../../components/tavern-base-ui/button/Button";
import { PlusIcon } from "@heroicons/react/24/outline";
import { builderColumns, profileColumns, builtAgentColumns } from "./landingShared";
import { MOCK_BUILDERS, MOCK_PROFILES, MOCK_BUILT_AGENTS } from "./mockData";
import CreateLinkModal from "../assets/components/CreateLinkModal/CreateLinkModal";

// Built agents / Build profiles / Builders --
// defaults to Built agents (index 0) when no `tab` param is present.
const TAB_NAMES = ["agents", "profiles", "builders"] as const;

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
 * `?emptydemo=profiles|builders` forces that tab's mock list empty, purely
 * to screenshot the empty-state CTA -- not real app behavior.
 */
export const AgentBuilderLandingPage = () => {
    const navigate = useNavigate();
    const [selectedAsset, setSelectedAsset] = useState<{ id: string; name: string } | null>(null);
    const [searchParams, setSearchParams] = useSearchParams();
    const tabParam = searchParams.get("tab");
    const emptyDemo = searchParams.get("emptydemo");
    const selectedIndex = Math.max(0, TAB_NAMES.indexOf(tabParam as typeof TAB_NAMES[number]));

    const handleTabChange = (index: number) => {
        setSearchParams({ tab: TAB_NAMES[index] }, { replace: true });
    };

    const profiles = emptyDemo === "profiles" ? [] : MOCK_PROFILES;
    const builders = emptyDemo === "builders" ? [] : MOCK_BUILDERS;
    const handleCreateLink = useCallback((id: string, name: string) => {
        setSelectedAsset({ id, name });
    }, []);

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
                        <div>({MOCK_BUILT_AGENTS.length})</div>
                    </Tab>
                    <Tab className={tabClass}>
                        <Settings2 className="w-4 h-4" />
                        <div>Build profiles</div>
                        <div>({profiles.length})</div>
                    </Tab>
                    <Tab className={tabClass}>
                        <Hammer className="w-4 h-4" />
                        <div>Builders</div>
                        <div>({builders.length})</div>
                    </Tab>
                </TabList>
                <TabPanels className="min-w-0">
                    <TabPanel>
                        <div className="flex flex-col gap-2">
                            <div className="rounded-md border border-gray-200 border-t-0 overflow-x-auto max-w-full">
                                <div className="min-w-[720px] w-full">
                                    <Table data={MOCK_BUILT_AGENTS} columns={builtAgentColumns(handleCreateLink)} />
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
                        </div>
                    </TabPanel>

                    <TabPanel>
                        {profiles.length === 0 ? (
                            <div className="rounded-md border border-gray-200 border-t-0 overflow-hidden">
                                <EmptyState
                                    type={EmptyStateType.noData}
                                    label="No build profiles available"
                                    details="Add one to get started."
                                >
                                    <Button
                                        leftIcon={<PlusIcon className="w-4 h-4" />}
                                        buttonStyle={{ color: "purple", size: "md" }}
                                        onClick={() => {}}
                                    >
                                        Add build profile
                                    </Button>
                                </EmptyState>
                            </div>
                        ) : (
                            <div className="flex flex-col gap-2">
                                <div className="rounded-md border border-gray-200 border-t-0 overflow-x-auto max-w-full">
                                    <div className="min-w-[720px] w-full">
                                        <Table data={profiles} columns={profileColumns} />
                                    </div>
                                </div>
                                <Button
                                    buttonVariant="ghost"
                                    buttonStyle={{ color: "purple", size: "sm" }}
                                    leftIcon={<PlusIcon className="w-4 h-4" />}
                                    onClick={() => {}}
                                    className="w-fit"
                                >
                                    Add build profile
                                </Button>
                            </div>
                        )}
                    </TabPanel>

                    <TabPanel>
                        {builders.length === 0 ? (
                            <div className="rounded-md border border-gray-200 border-t-0 overflow-hidden">
                                <EmptyState
                                    type={EmptyStateType.noData}
                                    label="No builders available"
                                    details="Add one to get started."
                                >
                                    <Button
                                        leftIcon={<PlusIcon className="w-4 h-4" />}
                                        buttonStyle={{ color: "purple", size: "md" }}
                                        onClick={() => {}}
                                    >
                                        Enroll builder
                                    </Button>
                                </EmptyState>
                            </div>
                        ) : (
                            <div className="flex flex-col gap-2">
                                <div className="rounded-md border border-gray-200 border-t-0 overflow-x-auto max-w-full">
                                    <div className="min-w-[720px] w-full">
                                        <Table data={builders} columns={builderColumns} />
                                    </div>
                                </div>
                                <Button
                                    buttonVariant="ghost"
                                    buttonStyle={{ color: "purple", size: "sm" }}
                                    leftIcon={<PlusIcon className="w-4 h-4" />}
                                    onClick={() => {}}
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
        </div>
    );
};

export default AgentBuilderLandingPage;
