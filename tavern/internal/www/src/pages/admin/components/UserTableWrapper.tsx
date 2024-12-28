import { EmptyState, EmptyStateType } from "../../../components/tavern-base-ui/EmptyState";
import { useUserTable } from "../hooks/useUserTable";
import UserTable from "./UserTable";

export const UserTableWrapper = () => {
    const { loading, users, error } = useUserTable();
    return (
        <div className="flex flex-col justify-center items-center gap-6">
            {(loading) ? (
                <EmptyState type={EmptyStateType.loading} label="Loading users..." />
            ) : error ? (
                <EmptyState type={EmptyStateType.error} label="Error users..." />
            ) : (users.length > 0) ? (
                <div className="py-4 bg-white rounded-lg shadow-lg mt-2 flex flex-col gap-1 w-full">
                    <UserTable data={users} />
                </div>
            ) : (
                <EmptyState type={EmptyStateType.noData} label="No user data found" />
            )}
        </div>
    );
}