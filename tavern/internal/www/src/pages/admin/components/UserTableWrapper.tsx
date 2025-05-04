import { useContext } from "react";
import { EmptyState, EmptyStateType } from "../../../components/tavern-base-ui/EmptyState";
import { useUserTable } from "../hooks/useUserTable";
import UserTable from "./UserTable";
import { AuthorizationContext } from "../../../context/AuthorizationContext";
import { UserType } from "../../../utils/consts";

export const UserTableWrapper = () => {
    const { loading: tableLoading, users, error: tableError } = useUserTable();
    const { data: authData } = useContext(AuthorizationContext);

    const currentUser: UserType = authData!.me!;

    return (
        <div className="flex flex-col justify-center items-center gap-6">
            {(tableLoading) ? (
                <EmptyState type={EmptyStateType.loading} label="Loading users..." />
            ) : (tableError) ? (
                <EmptyState type={EmptyStateType.error} label="Error users..." />
            ) : (users.length > 0) ? (
                <UserTable currentUser={currentUser} data={users} />
            ) : (
                <EmptyState type={EmptyStateType.noData} label="No user data found" />
            )}
        </div>
    );
}
