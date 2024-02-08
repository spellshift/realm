import React from "react";
import { PageWrapper } from "../../components/page-wrapper";
import { EmptyState, EmptyStateType } from "../../components/tavern-base-ui/EmptyState";
import { PageNavItem } from "../../utils/enums";
import HostFilter from "./components/HostFilter";
import UserTable from "./components/UserTable";
import { useHostsFilter } from "./hooks/useHostsFilter";
import { useUserTable } from "./hooks/useUserTable";

export const UserList = () => {
    const { loading, users, error } = useUserTable();

    return (
        <PageWrapper currNavItem={PageNavItem.users}>
            <div className="border-b border-gray-200 pb-5 sm:flex sm:items-center sm:justify-between">
                <h3 className="text-xl font-semibold leading-6 text-gray-900">User Management</h3>
            </div>
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
        </PageWrapper>
    );
}
