import { useCallback, useMemo } from "react";
import { useNavigate } from "react-router-dom";
import { formatDistance } from "date-fns";
import Badge from "../../../components/tavern-base-ui/badge/Badge";
import Button from "../../../components/tavern-base-ui/button/Button";
import { VirtualizedTable } from "../../../components/tavern-base-ui/virtualized-table/VirtualizedTable";
import { VirtualizedTableColumn } from "../../../components/tavern-base-ui/virtualized-table/types";
import { GET_BEACON_DETAIL_QUERY } from "./queries";
import { BeaconDetailQueryResponse } from "./types";
import { PrincipalAdminTypes, SupportedTransports } from "../../../utils/enums";
import { BeaconNode } from "../../../utils/interfacesQuery";
import { checkIfBeaconOffline, getEnumKey } from "../../../utils/utils";

interface BeaconsTableProps {
    beaconIds: string[];
    hasMore?: boolean;
    onLoadMore?: () => void;
}

export const BeaconsTable = ({ beaconIds, hasMore = false, onLoadMore }: BeaconsTableProps) => {
    const navigate = useNavigate();
    const currentDate = useMemo(() => new Date(), []);
    const principalColors = Object.values(PrincipalAdminTypes);

    const getVariables = useCallback((id: string) => ({ id }), []);

    const extractData = useCallback((response: BeaconDetailQueryResponse): BeaconNode | null => {
        return response?.beacons?.edges?.[0]?.node || null;
    }, []);

    const columns: VirtualizedTableColumn<BeaconNode>[] = useMemo(() => [
        {
            key: 'name',
            label: 'Beacon',
            width: 'minmax(150px,2fr)',
            render: (beacon) => (
                <div className="flex items-center min-w-0">
                    <span className="truncate">{beacon.name}</span>
                </div>
            ),
            renderSkeleton: () => (
                <div className="flex items-center min-w-0">
                    <div className="h-4 bg-gray-200 rounded animate-pulse w-32"></div>
                </div>
            ),
        },
        {
            key: 'principal',
            label: 'Principal',
            width: 'minmax(100px,1fr)',
            render: (beacon) => {
                const principal = beacon.principal;
                const color = principalColors.indexOf(principal as PrincipalAdminTypes) === -1 ? 'gray' : 'purple';
                return (
                    <div className="flex items-center">
                        <Badge badgeStyle={{ color: color }}>
                            {principal}
                        </Badge>
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
            key: 'transport',
            label: 'Transport',
            width: 'minmax(80px,1fr)',
            render: (beacon) => {
                const transport = beacon.transport;
                if (!transport) {
                    return null;
                }
                return (
                    <div className="flex items-center">
                        <Badge>
                            {getEnumKey(SupportedTransports, transport)}
                        </Badge>
                    </div>
                );
            },
            renderSkeleton: () => (
                <div className="flex items-center">
                    <div className="h-6 bg-gray-200 rounded animate-pulse w-16"></div>
                </div>
            ),
        },
        {
            key: 'status',
            label: 'Status',
            width: 'minmax(80px,1fr)',
            render: (beacon) => {
                const beaconOffline = checkIfBeaconOffline(beacon);
                const color = beaconOffline ? "red" : "gray";
                const text = beaconOffline ? "Offline" : "Online";
                return (
                    <div className="flex items-center">
                        <Badge badgeStyle={{ color: color }}>
                            {text}
                        </Badge>
                    </div>
                );
            },
            renderSkeleton: () => (
                <div className="flex items-center">
                    <div className="h-6 bg-gray-200 rounded animate-pulse w-16"></div>
                </div>
            ),
        },
        {
            key: 'lastSeenAt',
            label: 'Last callback',
            width: 'minmax(120px,1fr)',
            render: (beacon) => {
                let formattedLastSeen = 'N/A';
                if (beacon.lastSeenAt) {
                    try {
                        formattedLastSeen = formatDistance(new Date(beacon.lastSeenAt), currentDate);
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
            render: (beacon) => {
                const isOffline = checkIfBeaconOffline(beacon);
                const id = beacon.id;
                return (
                    <div className="flex flex-row justify-end">
                        {!isOffline &&
                            <Button
                                buttonStyle={{ color: "gray", size: 'md' }}
                                buttonVariant="ghost"
                                onClick={(e: any) => {
                                    e.stopPropagation();
                                    navigate("/createQuest", {
                                        state: {
                                            step: 1,
                                            beacons: [id]
                                        }
                                    });
                                }}>
                                New quest
                            </Button>
                        }
                    </div>
                );
            },
            renderSkeleton: () => (
                <div className="flex items-center min-w-0">
                    <div className="h-8 bg-gray-200 rounded animate-pulse w-24"></div>
                </div>
            ),
        },
    ], [currentDate, principalColors, navigate]);

    return (
        <VirtualizedTable<BeaconNode, BeaconDetailQueryResponse>
            items={beaconIds}
            columns={columns}
            query={GET_BEACON_DETAIL_QUERY}
            getVariables={getVariables}
            extractData={extractData}
            hasMore={hasMore}
            onLoadMore={onLoadMore}
            estimateRowSize={73}
            overscan={5}
            height= "60vh"
        />
    );
};
