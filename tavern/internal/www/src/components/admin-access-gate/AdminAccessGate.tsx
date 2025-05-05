import { EmptyState, EmptyStateType } from "../tavern-base-ui/EmptyState";
import { AuthorizationContext } from "../../context/AuthorizationContext";
import { ReactNode, useContext } from "react";

type Props = {
    children: ReactNode;
}
export const AdminAccessGate = ({ children }: Props) => {
    const { data } = useContext(AuthorizationContext);
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
