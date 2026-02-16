import { useCallback, useMemo } from "react";
import { VirtualizedTable } from "../../components/tavern-base-ui/virtualized-table/VirtualizedTable";
import { VirtualizedTableColumn } from "../../components/tavern-base-ui/virtualized-table/types";
import UserImageAndName from "../../components/UserImageAndName";
import Button from "../../components/tavern-base-ui/button/Button";
import Badge from "../../components/tavern-base-ui/badge/Badge";
import { UserNode } from "../../utils/interfacesQuery";
import { useUpdateUser } from "./hooks/useUpdateUser";
import { GET_USER_DETAIL_QUERY } from "./queries";
import { UserDetailQueryResponse } from "./types";

interface UserTableProps {
    userIds: string[];
    currentUser: UserNode;
    hasMore?: boolean;
    onLoadMore?: () => void;
}

const UserTable = ({ userIds, currentUser, hasMore = false, onLoadMore }: UserTableProps) => {
    const { submitUpdateUser } = useUpdateUser();

    const getVariables = useCallback((id: string) => ({ id }), []);

    const extractData = useCallback((response: UserDetailQueryResponse): UserNode | null => {
        return response?.users?.edges?.[0]?.node || null;
    }, []);

    const columns: VirtualizedTableColumn<UserNode>[] = useMemo(() => [
        {
            key: 'name',
            label: 'Name',
            width: 'minmax(200px, 2fr)',
            render: (user) => (
                <UserImageAndName userData={user} />
            ),
            renderSkeleton: () => (
                <div className="flex items-center gap-2">
                    <div className="h-8 w-8 bg-gray-200 rounded-full animate-pulse"></div>
                    <div className="h-4 bg-gray-200 rounded animate-pulse w-24"></div>
                </div>
            ),
        },
        {
            key: 'status',
            label: 'Status',
            width: 'minmax(150px, 1fr)',
            render: (user) => (
                <div className="flex flex-row flex-wrap gap-1">
                    {user?.isActivated ? (
                        <Badge badgeStyle={{ color: "green" }}>
                            <div>Activated</div>
                        </Badge>
                    ) : (
                        <Badge>
                            <div>Pending</div>
                        </Badge>
                    )}
                    {user?.isAdmin && (
                        <Badge badgeStyle={{ color: "purple" }}>
                            <div>Administrator</div>
                        </Badge>
                    )}
                </div>
            ),
            renderSkeleton: () => (
                <div className="flex items-center gap-1">
                    <div className="h-6 bg-gray-200 rounded animate-pulse w-16"></div>
                </div>
            ),
        },
        {
            key: 'actions',
            label: 'Actions',
            width: 'minmax(250px, 2fr)',
            render: (user) => {
                const isDisabled = user.id === currentUser.id;
                return (
                    <div className="flex flex-row flex-wrap gap-2">
                        {!user?.isActivated && (
                            <Button
                                disabled={isDisabled}
                                buttonVariant="outline"
                                buttonStyle={{ color: "gray", size: "sm" }}
                                onClick={(e) => {
                                    e.stopPropagation();
                                    submitUpdateUser({ id: parseInt(user.id), activated: true, admin: false });
                                }}
                            >
                                Activate
                            </Button>
                        )}
                        {user?.isActivated && (
                            <Button
                                disabled={isDisabled}
                                buttonVariant="outline"
                                buttonStyle={{ color: "red", size: "sm" }}
                                onClick={(e) => {
                                    e.stopPropagation();
                                    submitUpdateUser({ id: parseInt(user.id), activated: false, admin: false });
                                }}
                            >
                                Deactivate
                            </Button>
                        )}
                        {user?.isActivated && !user?.isAdmin && (
                            <Button
                                disabled={isDisabled}
                                buttonVariant="outline"
                                buttonStyle={{ color: "purple", size: "sm" }}
                                onClick={(e) => {
                                    e.stopPropagation();
                                    submitUpdateUser({ id: parseInt(user.id), activated: true, admin: true });
                                }}
                            >
                                Promote
                            </Button>
                        )}
                        {user?.isActivated && user?.isAdmin && (
                            <Button
                                disabled={isDisabled}
                                buttonVariant="outline"
                                buttonStyle={{ color: "red", size: "sm" }}
                                onClick={(e) => {
                                    e.stopPropagation();
                                    submitUpdateUser({ id: parseInt(user.id), activated: true, admin: false });
                                }}
                            >
                                Demote
                            </Button>
                        )}
                    </div>
                );
            },
            renderSkeleton: () => (
                <div className="flex items-center gap-2">
                    <div className="h-8 bg-gray-200 rounded animate-pulse w-20"></div>
                    <div className="h-8 bg-gray-200 rounded animate-pulse w-20"></div>
                </div>
            ),
        },
    ], [currentUser.id, submitUpdateUser]);

    return (
        <VirtualizedTable<UserNode, UserDetailQueryResponse>
            items={userIds}
            columns={columns}
            query={GET_USER_DETAIL_QUERY}
            getVariables={getVariables}
            extractData={extractData}
            hasMore={hasMore}
            onLoadMore={onLoadMore}
        />
    );
};

export default UserTable;
