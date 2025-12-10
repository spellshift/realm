import { EmptyState, EmptyStateType } from "../tavern-base-ui/EmptyState";
import { useAuthorization } from "../../context/AuthorizationContext";
import { ReactNode } from "react";

type Props = {
    children: ReactNode;
}
export const AccessGate = ({ children }: Props) => {
    const { data, isLoading, error } = useAuthorization();
    const userData = data?.me || null;


    if (isLoading) {
        return (
            <div className="flex flex-row w-sceen h-screen justify-center items-center">
                <EmptyState label="Loading authroization state" type={EmptyStateType.loading} />
            </div>
        );
    }

    if (error) {
        return (
            <div className="flex flex-row w-sceen h-screen justify-center items-center">
                <EmptyState label="Error fetching authroization state" type={EmptyStateType.error} details="Please contact your admin to diagnose the issue." />
            </div>
        );
    }

    if (!userData?.isActivated) {
        return (
            <div className="flex flex-row w-sceen h-screen justify-center items-center">
                <EmptyState label="Account not approved" details={`Gain approval by providing your id (${userData?.id}) to an admin.`} type={EmptyStateType.noData} />
            </div>
        );
    }

    return <>{children}</>
}
