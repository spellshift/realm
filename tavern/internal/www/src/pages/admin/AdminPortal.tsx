import { useState } from "react";
import { useMutation } from "@apollo/client";
import { TabGroup, TabPanels, TabPanel } from "@headlessui/react";
import { useSearchParams } from "react-router-dom";
import { AdminAccessGate } from "../../components/admin-access-gate";
import Breadcrumbs from "../../components/Breadcrumbs";
import PageHeader from "../../components/tavern-base-ui/PageHeader";
import { VirtualizedTableWrapper } from "../../components/tavern-base-ui/virtualized-table";
import { useAuthorization } from "../../context/AuthorizationContext";
import { UserNode } from "../../utils/interfacesQuery";
import { EmptyState, EmptyStateType } from "../../components/tavern-base-ui/EmptyState";
import UserTable from "./UserTable";
import { useUserIds } from "./hooks/useUserIds";
import AdminTabs from "./components/AdminTabs";
import { ProfilesTab } from "../agent-builder/profiles-tab";
import { BuildersTab } from "../agent-builder/builders-tab";
import NewBuildProfileModal from "../agent-builder/components/NewBuildProfileModal";
import { DELETE_BUILD_PROFILE, GET_BUILD_PROFILES, useBuilders, useBuildProfiles } from "../agent-builder/queries";
import { BuildProfile } from "../agent-builder/types";

const TAB_NAMES = ["users", "profiles", "builders"] as const;

export const AdminPortal = () => {
    const [searchParams, setSearchParams] = useSearchParams();
    const tabParam = searchParams.get("tab");
    const tabIndex = TAB_NAMES.indexOf(tabParam as typeof TAB_NAMES[number]);
    const selectedIndex = tabIndex >= 0 ? tabIndex : 0;

    const [newProfileModalOpen, setNewProfileModalOpen] = useState(false);
    const [editingProfile, setEditingProfile] = useState<BuildProfile | null>(null);
    const [deleteProfile] = useMutation(DELETE_BUILD_PROFILE, {
        refetchQueries: [{ query: GET_BUILD_PROFILES }],
    });

    const {
        data,
        userIds,
        initialLoading,
        error,
        hasMore,
        loadMore,
    } = useUserIds();

    const { profiles, tomes, loading: profilesLoading, error: profilesError, refetch: refetchProfiles } = useBuildProfiles();
    const { builders, loading: buildersLoading, error: buildersError } = useBuilders();

    const { data: authData } = useAuthorization();
    const currentUser = authData?.me as UserNode | undefined;

    const handleTabChange = (index: number) => {
        setSearchParams({ tab: TAB_NAMES[index] }, { replace: true });
    };

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

    if (!currentUser) {
        return (
            <AdminAccessGate>
                <EmptyState type={EmptyStateType.error} label="Failed to load user information" />
            </AdminAccessGate>
        );
    }

    return (
        <AdminAccessGate>
            <div className="flex flex-col gap-4 w-full min-w-0">
                <Breadcrumbs pages={[{
                    label: "Admin",
                    link: "/admin"
                }]} />
                <PageHeader
                    title="Admin"
                    description="This portal is only accessible to Realm Admin. Manage users, build profiles, and builders."
                />
                <TabGroup selectedIndex={selectedIndex} onChange={handleTabChange}>
                    <AdminTabs
                        userCount={data?.users?.totalCount}
                        profileCount={profiles.length}
                        builderCount={builders.length}
                    />
                    <TabPanels className="min-w-0">
                        <TabPanel>
                            <VirtualizedTableWrapper
                                title="Users"
                                totalItems={data?.users?.totalCount}
                                loading={initialLoading}
                                error={error}
                                showFiltering={false}
                                table={
                                    <UserTable
                                        userIds={userIds}
                                        currentUser={currentUser}
                                        hasMore={hasMore}
                                        onLoadMore={loadMore}
                                    />
                                }
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
        </AdminAccessGate>
    );
};

export default AdminPortal;
