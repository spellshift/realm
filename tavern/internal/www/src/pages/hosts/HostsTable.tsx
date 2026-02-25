import { useCallback, useMemo } from "react";
import { useNavigate } from "react-router-dom";
import { formatDistance } from "date-fns";
import Badge from "../../components/tavern-base-ui/badge/Badge";
import HostTile from "../../components/HostTile";
import { VirtualizedTable } from "../../components/tavern-base-ui/virtualized-table/VirtualizedTable";
import { VirtualizedTableColumn } from "../../components/tavern-base-ui/virtualized-table/types";
import { GET_HOST_DETAIL_QUERY } from "./queries";
import { HostDetailQueryResponse } from "./types";
import { getOfflineOnlineStatus, getFormatForPrincipal } from "../../utils/utils";
import { PrincipalAdminTypes } from "../../utils/enums";
import { HostNode } from "../../utils/interfacesQuery";
import { CreateShellButton } from "../../components/create-shell-button/CreateShellButton";

interface HostsTableProps {
    hostIds: string[];
    hasMore?: boolean;
    onLoadMore?: () => void;
}

export const HostsTable = ({ hostIds, hasMore = false, onLoadMore }: HostsTableProps) => {
    const navigate = useNavigate();
    const currentDate = useMemo(() => new Date(), []);
    const principalColors = Object.values(PrincipalAdminTypes);

    const handleRowClick = useCallback((hostId: string) => {
        navigate(`/hosts/${hostId}`);
    }, [navigate]);

    const getVariables = useCallback((id: string) => ({ id }), []);

    const extractData = useCallback((response: HostDetailQueryResponse): HostNode | null => {
        return response?.hosts?.edges?.[0]?.node || null;
    }, []);

    const columns: VirtualizedTableColumn<HostNode>[] = useMemo(() => [
        {
            key: 'host-details',
            label: 'Host details',
            width: 'minmax(250px,3fr)',
            render: (host) => (
                <div className="flex flex-col min-w-0">
                    <HostTile data={host} />
                </div>
            ),
            renderSkeleton: () => (
                <div className="flex flex-col min-w-0 space-y-2">
                    <div className="h-4 bg-gray-200 rounded animate-pulse w-3/4"></div>
                    <div className="h-3 bg-gray-200 rounded animate-pulse w-1/2"></div>
                </div>
            ),
        },
        {
            key: 'online-beacons',
            label: 'Online beacons',
            width: 'minmax(120px,1fr)',
            render: (host) => {
                const beacons = host.beacons?.edges || [];
                const { online, offline } = getOfflineOnlineStatus(beacons);
                const beaconStatusColor = online === 0 ? "red" : "gray";

                return (
                    <div className="flex items-center">
                        <Badge badgeStyle={{ color: beaconStatusColor }}>
                            {online}/{offline + online}
                        </Badge>
                    </div>
                );
            },
            renderSkeleton: () => (
                <div className="flex items-center">
                    <div className="h-6 bg-gray-200 rounded animate-pulse w-12"></div>
                </div>
            ),
        },
        {
            key: 'beacon-principals',
            label: 'Beacon principals',
            width: 'minmax(150px,2fr)',
            render: (host) => {
                const beacons = host.beacons?.edges || [];
                const beaconPrincipals = getFormatForPrincipal(beacons);

                return (
                    <div className="flex flex-row flex-wrap gap-1 items-center min-w-0">
                        {beaconPrincipals.map((principal: string) => {
                            const color = principalColors.indexOf(principal as PrincipalAdminTypes) === -1 ? 'gray' : 'purple';
                            return (
                                <Badge key={principal} badgeStyle={{ color: color }}>
                                    {principal}
                                </Badge>
                            );
                        })}
                    </div>
                );
            },
            renderSkeleton: () => (
                <div className="flex items-center">
                    <div className="h-6 bg-gray-200 rounded animate-pulse w-20"></div>
                </div>
            ),
        },
        {
            key: 'last-callback',
            label: 'Last callback',
            width: 'minmax(120px,1fr)',
            render: (host) => {
                let formattedLastSeen = 'N/A';
                if (host.lastSeenAt) {
                    try {
                        formattedLastSeen = formatDistance(new Date(host.lastSeenAt), currentDate);
                    } catch {
                        formattedLastSeen = 'Invalid date';
                    }
                }

                return (
                    <div className="flex items-center min-w-0">
                        <span className="truncate">{formattedLastSeen}</span>
                    </div>
                );
            },
            renderSkeleton: () => (
                <div className="flex items-center min-w-0">
                    <div className="h-4 bg-gray-200 rounded animate-pulse w-20"></div>
                </div>
            ),
        },
        {
            key: 'actions',
            label: '',
            width: 'minmax(100px,1fr)',
            render: (host) => (
                <div className="flex justify-end">
                    <CreateShellButton hostId={host.id} />
                </div>
            ),
            renderSkeleton: () => (
                <div className="flex justify-end">
                    <div className="h-8 bg-gray-200 rounded animate-pulse w-20"></div>
                </div>
            ),
        },
    ], [currentDate, principalColors]);

    return (
        <VirtualizedTable<HostNode, HostDetailQueryResponse>
            items={hostIds}
            columns={columns}
            query={GET_HOST_DETAIL_QUERY}
            getVariables={getVariables}
            extractData={extractData}
            onItemClick={handleRowClick}
            hasMore={hasMore}
            onLoadMore={onLoadMore}
            estimateRowSize={73}
            overscan={5}
        />
    );
};
