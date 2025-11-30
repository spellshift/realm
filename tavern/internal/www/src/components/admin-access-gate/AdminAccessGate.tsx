import { EmptyState, EmptyStateType } from "../tavern-base-ui/EmptyState";
import { useAuthorization } from "../../context/AuthorizationContext";
import { ReactNode } from "react";

type Props = {
    children: ReactNode;
}
export const AdminAccessGate = ({ children }: Props) => {
    const { data } = useAuthorization();
    const userData = data?.me || null;

    if (!userData?.isAdmin) {
        return (
            <div className="flex flex-row w-sceen h-screen justify-center items-center">
                <EmptyState label="Not Authorized" type={EmptyStateType.error} details="You are not authorized to view this page. Please contact your admin if you believe this is a mistake." />
            </div>
        );
    }

    return <>{children}</>
}
