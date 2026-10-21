import { useState } from "react";
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
import { useBuilders, useBuildProfiles } from "../agent-builder/queries";

const TAB_NAMES = ["users", "profiles", "builders"] as const;

export const AdminPortal = () => {
    const [searchParams, setSearchParams] = useSearchParams();
    const tabParam = searchParams.get("tab");
    const tabIndex = TAB_NAMES.indexOf(tabParam as typeof TAB_NAMES[number]);
    const selectedIndex = tabIndex >= 0 ? tabIndex : 0;

    const [newProfileModalOpen, setNewProfileModalOpen] = useState(false);

    const {
        data,
        userIds,
        initialLoading,
        error,
        hasMore,
        loadMore,
    } = useUserIds();

    const { profiles, tomes, loading: profilesLoading, error: profilesError, refetch: refetchProfiles } = useBuildProfiles();
    const { builders, loading: buildersLoading, error: buildersError, refetch: refetchBuilders } = useBuilders();

    const { data: authData } = useAuthorization();
    const currentUser = authData?.me as UserNode | undefined;

    const handleTabChange = (index: number) => {
        setSearchParams({ tab: TAB_NAMES[index] }, { replace: true });
    };

    const goToNewProfile = () => setNewProfileModalOpen(true);

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
                            />
                        </TabPanel>
                        <TabPanel>
                            <BuildersTab
                                builders={builders}
                                loading={buildersLoading}
                                error={buildersError}
                                onEnrolled={() => void refetchBuilders()}
                            />
                        </TabPanel>
                    </TabPanels>
                </TabGroup>
                <NewBuildProfileModal
                    isOpen={newProfileModalOpen}
                    setOpen={setNewProfileModalOpen}
                    catalog={tomes}
                    onSave={async () => {
                        await refetchProfiles();
                    }}
                />
            </div>
        </AdminAccessGate>
    );
};

export default AdminPortal;
