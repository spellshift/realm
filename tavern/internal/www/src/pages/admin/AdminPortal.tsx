import { AdminAccessGate } from "../../components/admin-access-gate";
import Breadcrumbs from "../../components/Breadcrumbs";
import PageHeader from "../../components/tavern-base-ui/PageHeader";
import { PageWrapper } from "../../components/page-wrapper";
import { PageNavItem } from "../../utils/enums";
import { useQuery } from "@apollo/client";
import { GET_USER_QUERY } from "../../utils/queries";
import { useAuthorization } from "../../context/AuthorizationContext";
import { UserNode, UserQueryTopLevel } from "../../utils/interfacesQuery";
import { EmptyState, EmptyStateType } from "../../components/tavern-base-ui/EmptyState";
import UserTable from "./components/UserTable";

export const AdminPortal = () => {
    const { loading, data, error } = useQuery<UserQueryTopLevel>(GET_USER_QUERY);

    const { data: authData } = useAuthorization();

    const currentUser = authData?.me as UserNode | undefined;

    if (!currentUser) {
        return (
            <PageWrapper currNavItem={PageNavItem.admin}>
                <AdminAccessGate>
                    <EmptyState type={EmptyStateType.error} label="Failed to load user information" />
                </AdminAccessGate>
            </PageWrapper>
        );
    }

    return (
        <PageWrapper currNavItem={PageNavItem.admin}>
            <AdminAccessGate>
                <Breadcrumbs pages={[{
                    label: "Admin",
                    link: "/admin"
                }]} />
                <PageHeader title="Admin" description="This portal is only accessible to Realm Admin. You can Activate/Deactivate users to grant or remove access to Realm. You can Promote/Demote users to grant or remove Admin privileges." />
                <div className="flex flex-col justify-center items-center gap-6">
                    {(loading) ? (
                        <EmptyState type={EmptyStateType.loading} label="Loading users..." />
                    ) : (error) ? (
                        <EmptyState type={EmptyStateType.error} label="Failed to load users" />
                    ) : (data?.users?.totalCount && data.users.totalCount > 0) ? (
                        <UserTable currentUser={currentUser} data={data.users.edges} />
                    ) : (
                        <EmptyState type={EmptyStateType.noData} label="No user data found" />
                    )}
                </div>
            </AdminAccessGate>
        </PageWrapper>
    );
}
