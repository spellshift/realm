import { useCallback, useMemo } from "react";
import Badge from "../../../components/tavern-base-ui/badge/Badge";
import Tooltip from "../../../components/tavern-base-ui/Tooltip";
import { VirtualizedTable } from "../../../components/tavern-base-ui/virtualized-table/VirtualizedTable";
import { VirtualizedTableColumn } from "../../../components/tavern-base-ui/virtualized-table/types";
import { GET_PROCESS_DETAIL_QUERY } from "./queries";
import { ProcessDetailQueryResponse } from "./types";
import { PrincipalAdminTypes } from "../../../utils/enums";
import { ProcessNode } from "../../../utils/interfacesQuery";
import { format } from "date-fns";

interface ProcessesTableProps {
    hostId: string;
    processIds: string[];
    hasMore?: boolean;
    onLoadMore?: () => void;
}


const formatStatus = (status: string): string => {
    return status.replace('STATUS_', '').toLowerCase().replace(/_/g, ' ');
};

export const ProcessesTable = ({ hostId, processIds, hasMore = false, onLoadMore }: ProcessesTableProps) => {
    const principalColors = Object.values(PrincipalAdminTypes);

    const getVariables = useCallback((id: string) => ({ hostId, processId: id }), [hostId]);

    const extractData = useCallback((response: ProcessDetailQueryResponse): ProcessNode | null => {
        return response?.hosts?.edges?.[0]?.node?.processes?.edges?.[0]?.node || null;
    }, []);

    const columns: VirtualizedTableColumn<ProcessNode>[] = useMemo(() => [
        {
            key: 'name',
            label: 'Process name',
            width: 'minmax(80px,1fr)',
            render: (process: ProcessNode) => process.name
        },
        {
            key: 'path',
            label: 'Path',
            width: 'minmax(100px,1fr)',
            render: (process: ProcessNode) => (
                <Tooltip label={process.path || ''} isDisabled={!process.path}>
                    <div className="truncate text-sm text-gray-600">
                        {process.path || '-'}
                    </div>
                </Tooltip>
            ),
        },
        {
            key: 'principal',
            label: 'User',
            width: 'minmax(120px,1fr)',
            render: (process: ProcessNode) => {
                const principal = process.principal;
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
            key: 'pid',
            label: 'PID',
            width: 'minmax(80px,0.5fr)',
            render: (process: ProcessNode) => process.pid,
        },
        {
            key: 'ppid',
            label: 'PPID',
            width: 'minmax(80px,0.5fr)',
            render: (process: ProcessNode) => process.ppid,
        },
        {
            key: 'status',
            label: 'Status',
            width: 'minmax(80px,0.5fr)',
            render: (process: ProcessNode) => formatStatus(process.status),
        },
        {
            key: 'lastModifiedAt',
            label: 'Last Reported',
            width: 'minmax(80px,1fr)',
            render: (process: ProcessNode) => format(new Date(process.lastModifiedAt), "yyyy-MM-dd HH:mm"),
        },
    ], [principalColors]);

    return (
        <VirtualizedTable<ProcessNode, ProcessDetailQueryResponse>
            items={processIds}
            columns={columns}
            query={GET_PROCESS_DETAIL_QUERY}
            getVariables={getVariables}
            extractData={extractData}
            hasMore={hasMore}
            onLoadMore={onLoadMore}
            estimateRowSize={73}
            overscan={5}
            height="60vh"
        />
    );
};
