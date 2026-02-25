import { useCallback, useMemo } from "react";
import { useNavigate } from "react-router-dom";
import { TerminalIcon } from "lucide-react";
import { Image, Tooltip } from "@chakra-ui/react";
import Badge from "../../../components/tavern-base-ui/badge/Badge";
import Button from "../../../components/tavern-base-ui/button/Button";
import { VirtualizedTable } from "../../../components/tavern-base-ui/virtualized-table/VirtualizedTable";
import { VirtualizedTableColumn } from "../../../components/tavern-base-ui/virtualized-table/types";
import { GET_SHELL_DETAIL_QUERY } from "./queries";
import { ShellsQueryResponse, ShellNode } from "./types";
import { checkIfBeaconOffline } from "../../../utils/utils";
import UserImageAndName from "../../../components/UserImageAndName";
import PlaceholderUser from "../../../assets/PlaceholderUser.png";

interface ShellsTableProps {
    shellIds: string[];
    hasMore?: boolean;
    onLoadMore?: () => void;
}

export const ShellsTable = ({ shellIds, hasMore = false, onLoadMore }: ShellsTableProps) => {
    const navigate = useNavigate();

    const getVariables = useCallback((id: string) => ({ id }), []);

    const extractData = useCallback((response: ShellsQueryResponse): ShellNode | null => {
        return response?.edges?.[0]?.node || null;
    }, []);

    const columns: VirtualizedTableColumn<ShellNode>[] = useMemo(() => [
        {
            key: 'beacon',
            label: 'Beacon',
            width: 'minmax(150px, 1.5fr)',
            render: (shell) => (
                <div className="flex items-center min-w-0">
                    <span className="truncate">{shell.beacon.name}</span>
                </div>
            ),
            renderSkeleton: () => (
                <div className="flex items-center min-w-0">
                    <div className="h-4 bg-gray-200 rounded animate-pulse w-32"></div>
                </div>
            ),
        },
        {
            key: 'status',
            label: 'Beacon Status',
            width: 'minmax(120px, 1fr)',
            render: (shell) => {
                const isOffline = checkIfBeaconOffline(shell.beacon);
                const color = isOffline ? "red" : "green";
                const text = isOffline ? "Offline" : "Online";
                return (
                    <Badge badgeStyle={{ color }}>
                        {text}
                    </Badge>
                );
            },
            renderSkeleton: () => (
                <div className="h-6 bg-gray-200 rounded animate-pulse w-20"></div>
            ),
        },
        {
            key: 'usage',
            label: 'Usage',
            width: 'minmax(100px, 1fr)',
            render: (shell) => (
                <span className="text-sm text-gray-600">
                    {shell.shellTasks.totalCount} Tasks
                </span>
            ),
            renderSkeleton: () => (
                <div className="h-4 bg-gray-200 rounded animate-pulse w-16"></div>
            ),
        },
        {
            key: 'creator',
            label: 'Creator',
            width: 'minmax(200px, 2fr)',
            render: (shell) => (
                <UserImageAndName userData={shell.owner} />
            ),
            renderSkeleton: () => (
                <div className="flex items-center gap-2">
                    <div className="h-8 w-8 bg-gray-200 rounded-full animate-pulse"></div>
                    <div className="h-4 bg-gray-200 rounded animate-pulse w-24"></div>
                </div>
            ),
        },
        {
            key: 'activeUsers',
            label: 'Active Users',
            width: 'minmax(150px, 1.5fr)',
            render: (shell) => {
                const users = shell.activeUsers?.edges?.map(edge => edge.node) || [];
                if (users.length === 0) return <span className="text-gray-400 text-sm">-</span>;

                return (
                    <div className="flex -space-x-2 overflow-hidden">
                        {users.map((user) => (
                            <Tooltip label={user.name} key={user.id}>
                                <Image
                                    borderRadius='full'
                                    boxSize='28px'
                                    src={user.photoURL || PlaceholderUser}
                                    alt={user.name}
                                    className="border-2 border-white inline-block h-8 w-8 rounded-full ring-2 ring-white"
                                />
                            </Tooltip>
                        ))}
                    </div>
                );
            },
            renderSkeleton: () => (
                <div className="flex -space-x-2">
                    <div className="h-8 w-8 bg-gray-200 rounded-full animate-pulse ring-2 ring-white"></div>
                    <div className="h-8 w-8 bg-gray-200 rounded-full animate-pulse ring-2 ring-white"></div>
                </div>
            ),
        },
        {
            key: 'actions',
            label: '',
            width: 'minmax(80px, 0.5fr)',
            render: (shell) => {
                const isBeaconOffline = checkIfBeaconOffline(shell.beacon);
                const isTerminated = shell.closedAt || isBeaconOffline;

                if (isTerminated) {
                    return (
                        <div className="flex justify-end">
                            <Badge badgeStyle={{ color: "gray" }}>Terminated</Badge>
                        </div>
                    );
                }

                return (
                    <div className="flex justify-end">
                        <Button
                            buttonStyle={{ color: "purple", size: 'sm' }}
                            buttonVariant="ghost"
                            onClick={(e) => {
                                e.stopPropagation();
                                navigate(`/shellv2/${shell.id}`);
                            }}
                        >
                            <TerminalIcon className="w-4 h-4" />
                        </Button>
                    </div>
                );
            },
            renderSkeleton: () => (
                <div className="flex justify-end">
                    <div className="h-8 w-8 bg-gray-200 rounded animate-pulse"></div>
                </div>
            ),
        },
    ], [navigate]);

    return (
        <VirtualizedTable<ShellNode, ShellsQueryResponse>
            items={shellIds}
            columns={columns}
            query={GET_SHELL_DETAIL_QUERY}
            getVariables={getVariables}
            extractData={extractData}
            hasMore={hasMore}
            onLoadMore={onLoadMore}
            estimateRowSize={60}
            overscan={5}
            height="60vh"
        />
    );
};
