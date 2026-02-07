import { useMemo } from "react";
import { formatDistance } from "date-fns";
import Badge from "../../components/tavern-base-ui/badge/Badge";
import HostTile from "../../components/HostTile";
import { VirtualizedTableRow } from "../../components/tavern-base-ui/virtualized-table/VirtualizedTableRow";
import { VirtualizedTableColumn } from "../../components/tavern-base-ui/virtualized-table/types";
import { GET_HOST_DETAIL_QUERY } from "./queries";
import { HostDetailQueryResponse } from "./types";
import { getOfflineOnlineStatus, getFormatForPrincipal } from "../../utils/utils";
import { PrincipalAdminTypes } from "../../utils/enums";
import { HostNode } from "../../utils/interfacesQuery";

interface HostRowVirtualizedProps {
    hostId: string;
    onRowClick: (hostId: string) => void;
    isVisible: boolean;
}

export const HostRowVirtualized = ({ hostId, onRowClick, isVisible }: HostRowVirtualizedProps) => {
    const currentDate = useMemo(() => new Date(), []);
    const principalColors = Object.values(PrincipalAdminTypes);

    const columns: VirtualizedTableColumn<HostNode>[] = useMemo(() => [
        {
            key: 'host-details',
            gridWidth: 'minmax(250px,3fr)',
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
            gridWidth: 'minmax(120px,1fr)',
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
            gridWidth: 'minmax(150px,2fr)',
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
            gridWidth: 'minmax(120px,1fr)',
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
    ], [currentDate, principalColors]);

    return (
        <VirtualizedTableRow<HostNode, HostDetailQueryResponse>
            itemId={hostId}
            query={GET_HOST_DETAIL_QUERY}
            getVariables={(id) => ({ id })}
            columns={columns}
            extractData={(response) => response?.hosts?.edges?.[0]?.node || null}
            onRowClick={onRowClick}
            isVisible={isVisible}
        />
    );
};
