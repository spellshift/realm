import { AdminAccessGate } from "../../components/admin-access-gate";
import { PageWrapper } from "../../features/page-wrapper";
import { PageNavItem } from "../../utils/enums";
import { UserTableWrapper } from "./components/UserTableWrapper";

export const AdminPortal = () => {

    // Admin access gate is nested in PageWrapper to allow users who somehow get to an unauthorized page the ability to view the navigation to traverse back to acceptable pages

    return (
        <PageWrapper currNavItem={PageNavItem.admin}>
            <AdminAccessGate>
                <div className="border-b border-gray-200 pb-5 flex flex-col sm:flex-row  sm:items-center sm:justify-between gap-4">
                    <div className="flex-1 flex flex-col gap-2">
                        <h3 className="text-xl font-semibold leading-6 text-gray-900">Admin Portal</h3>
                        <div className="max-w-2xl text-sm">
                            <span>This portal is only accessible to Realm Admin. You can  Activate/Deactivate users to grant or remove access to Realm. You can Promote/Demote users to grant or remove Admin privileges.</span>
                        </div>
                    </div>
                </div>
                <div>
                    <UserTableWrapper />
                </div>
            </AdminAccessGate>
        </PageWrapper>
    );
}
