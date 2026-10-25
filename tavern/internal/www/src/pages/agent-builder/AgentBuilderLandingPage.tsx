import { useCallback, useState } from "react";
import { useMutation } from "@apollo/client";
import { useToast } from "@chakra-ui/react";
import { TabGroup, TabPanels, TabPanel } from "@headlessui/react";
import { useSearchParams, useNavigate } from "react-router-dom";
import { FileTerminal } from "lucide-react";
import Breadcrumbs from "../../components/Breadcrumbs";
import Button from "../../components/tavern-base-ui/button/Button";
import AgentBuilderTabs from "./components/AgentBuilderTabs";
import { AgentsTab } from "./agents-tab";
import { ProfilesTab } from "./profiles-tab";
import { BuildersTab } from "./builders-tab";
import NewBuildProfileModal from "./components/NewBuildProfileModal";
import { DELETE_BUILD_PROFILE, GET_BUILD_PROFILES, useBuilders, useBuildProfiles, useBuildTasks } from "./queries";
import { BuildProfile } from "./types";
import CreateLinkModal from "../assets/components/CreateLinkModal/CreateLinkModal";

// Built agents / Build profiles / Builders --
// defaults to Built agents (index 0) when no `tab` param is present.
const TAB_NAMES = ["agents", "profiles", "builders"] as const;

/**
 * Tabbed landing page using the same tab idiom as the host detail page
 * (HostDetails.tsx/HostTabs.tsx) -- Headless UI TabGroup/TabList/Tab with
 * folder-style tabs, each tab's content broken out into its own
 * `*-tab/` directory (agents-tab, profiles-tab, builders-tab), with shared
 * pieces like the enroll builder modal under `components/`. This page is
 * just the coordinator: it owns the tab/search-param wiring, fetches the
 * data each tab needs (so tab header counts and tab content share one
 * query each, rather than refetching per tab), and renders "Generate
 * agent" / the cross-tab "Create link" modal.
 *
 * Built agents' table shows one row per BuildTask (see buildTaskRows in
 * queries.ts) -- queued/claimed/building tasks render with a live status
 * badge and no artifact yet, so there's no separate "build in progress"
 * banner; the table itself is the live status view. `?task=<id>` (set by
 * the wizard's "Start build" redirect) surfaces the just-created task by
 * sorting it to the top of that table and tagging it "New".
 */
export const AgentBuilderLandingPage = () => {
    const navigate = useNavigate();
    const toast = useToast();
    const [selectedAsset, setSelectedAsset] = useState<{ id: string; name: string } | null>(null);
    const [searchParams, setSearchParams] = useSearchParams();
    const tabParam = searchParams.get("tab");
    const highlightTaskId = searchParams.get("task");
    const selectedIndex = Math.max(0, TAB_NAMES.indexOf(tabParam as typeof TAB_NAMES[number]));

    const { profiles, tomes, loading: profilesLoading, error: profilesError, refetch: refetchProfiles } = useBuildProfiles();
    const { builders, loading: buildersLoading, error: buildersError } = useBuilders();
    const { rows: taskRows, loading: tasksLoading, error: tasksError } = useBuildTasks();
    // Bring the just-started task (if any) to the top, rather than leaving
    // the operator to hunt for it in createdAt order.
    const rows = highlightTaskId
        ? [...taskRows].sort((a, b) => (a.id === highlightTaskId ? -1 : b.id === highlightTaskId ? 1 : 0))
        : taskRows;

    const handleTabChange = (index: number) => {
        const next: Record<string, string> = { tab: TAB_NAMES[index] };
        if (highlightTaskId) next.task = highlightTaskId;
        setSearchParams(next, { replace: true });
    };

    const handleCreateLink = useCallback((id: string, name: string) => {
        setSelectedAsset({ id, name });
    }, []);

    const [newProfileModalOpen, setNewProfileModalOpen] = useState(false);
    const [editingProfile, setEditingProfile] = useState<BuildProfile | null>(null);
    const [deleteProfile] = useMutation(DELETE_BUILD_PROFILE, {
        refetchQueries: [{ query: GET_BUILD_PROFILES }],
        onCompleted: () => {
            toast({ title: "Build profile deleted", status: "success", duration: 2000, isClosable: true });
        },
        onError: (err) => {
            toast({
                title: "Failed to delete build profile",
                description: err.message,
                status: "error",
                duration: 4000,
                isClosable: true,
            });
        },
    });

    const goToWizard = () => navigate("/builder/new");
    const goToNewProfile = () => {
        setEditingProfile(null);
        setNewProfileModalOpen(true);
    };
    const goToEditProfile = (profile: BuildProfile) => {
        setEditingProfile(profile);
        setNewProfileModalOpen(true);
    };
    const handleDeleteProfile = (profile: BuildProfile) => {
        void deleteProfile({ variables: { profileID: profile.id } });
    };

    return (
        <div className="flex flex-col gap-4 w-full min-w-0">
            <div className="flex flex-col sm:flex-row gap-3 justify-between w-full min-w-0 items-start sm:items-center">
                <Breadcrumbs pages={[{ label: "Agent Builder", link: "/builder" }]} />
                <Button
                    leftIcon={<FileTerminal className="w-5 h-5" />}
                    buttonStyle={{ color: "purple", size: "md" }}
                    onClick={goToWizard}
                >
                    Generate agent
                </Button>
            </div>

            <TabGroup selectedIndex={selectedIndex} onChange={handleTabChange}>
                <AgentBuilderTabs
                    agentCount={rows.length}
                    profileCount={profiles.length}
                    builderCount={builders.length}
                />
                <TabPanels className="min-w-0">
                    <TabPanel>
                        <AgentsTab
                            rows={rows}
                            loading={tasksLoading}
                            error={tasksError}
                            highlightTaskId={highlightTaskId}
                            onCreateLink={handleCreateLink}
                            onGenerateAgent={goToWizard}
                        />
                    </TabPanel>
                    <TabPanel>
                        <ProfilesTab
                            profiles={profiles}
                            loading={profilesLoading}
                            error={profilesError}
                            onAddProfile={goToNewProfile}
                            onEditProfile={goToEditProfile}
                            onDeleteProfile={handleDeleteProfile}
                        />
                    </TabPanel>
                    <TabPanel>
                        <BuildersTab
                            builders={builders}
                            loading={buildersLoading}
                            error={buildersError}
                        />
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
            <NewBuildProfileModal
                isOpen={newProfileModalOpen}
                setOpen={setNewProfileModalOpen}
                catalog={tomes}
                existingProfile={editingProfile ?? undefined}
                onSave={async () => {
                    await refetchProfiles();
                }}
            />
        </div>
    );
};

export default AgentBuilderLandingPage;
