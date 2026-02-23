import { AdminAccessGate } from "../../components/admin-access-gate";
import Breadcrumbs from "../../components/Breadcrumbs";
import PageHeader from "../../components/tavern-base-ui/PageHeader";
import { VirtualizedTableWrapper } from "../../components/tavern-base-ui/virtualized-table";
import { useAuthorization } from "../../context/AuthorizationContext";
import { UserNode } from "../../utils/interfacesQuery";
import { EmptyState, EmptyStateType } from "../../components/tavern-base-ui/EmptyState";
import UserTable from "./UserTable";
import { useUserIds } from "./hooks/useUserIds";

export const AdminPortal = () => {
    const {
        data,
        userIds,
        initialLoading,
        error,
        hasMore,
        loadMore,
    } = useUserIds();

    const { data: authData } = useAuthorization();

    const currentUser = authData?.me as UserNode | undefined;

    if (!currentUser) {
        return (
            <AdminAccessGate>
                <EmptyState type={EmptyStateType.error} label="Failed to load user information" />
            </AdminAccessGate>
        );
    }

    return (
        <AdminAccessGate>
            <Breadcrumbs pages={[{
                label: "Admin",
                link: "/admin"
            }]} />
            <PageHeader title="Admin" description="This portal is only accessible to Realm Admin. You can Activate/Deactivate users to grant or remove access to Realm. You can Promote/Demote users to grant or remove Admin privileges." />
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
        </AdminAccessGate>
    );
}
