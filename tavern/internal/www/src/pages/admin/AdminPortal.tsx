import { AdminAccessGate } from "../../components/admin-access-gate";
import Breadcrumbs from "../../components/Breadcrumbs";
import PageHeader from "../../components/tavern-base-ui/PageHeader";
import { PageWrapper } from "../../features/page-wrapper";
import { PageNavItem } from "../../utils/enums";
import { UserTableWrapper } from "./components/UserTableWrapper";

export const AdminPortal = () => {
    // Admin access gate is nested in PageWrapper to allow users who somehow get to an unauthorized page the ability to view the navigation to traverse back to acceptable pages

    return (
        <PageWrapper currNavItem={PageNavItem.admin}>
            <AdminAccessGate>
                <Breadcrumbs pages={[{
                    label: "Admin",
                    link: "/admin"
                }]} />
                <PageHeader title="Admin" description="This portal is only accessible to Realm Admin. You can  Activate/Deactivate users to grant or remove access to Realm. You can Promote/Demote users to grant or remove Admin privileges." />
                <UserTableWrapper />
            </AdminAccessGate>
        </PageWrapper>
    );
}
