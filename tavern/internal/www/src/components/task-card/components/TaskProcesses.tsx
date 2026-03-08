import { FC, useCallback, useMemo } from "react";
import { gql, useQuery } from "@apollo/client";
import { Link } from "react-router-dom";
import Badge from "../../tavern-base-ui/badge/Badge";
import Tooltip from "../../tavern-base-ui/Tooltip";
import { VirtualizedTable } from "../../tavern-base-ui/virtualized-table/VirtualizedTable";
import { VirtualizedTableColumn } from "../../tavern-base-ui/virtualized-table/types";
import { PrincipalAdminTypes } from "../../../utils/enums";
import { ProcessNode } from "../../../utils/interfacesQuery";
import { EmptyState, EmptyStateType } from "../../tavern-base-ui/EmptyState";
import { ArrowRight } from "lucide-react";

const GET_TASK_PROCESS_IDS_QUERY = gql`
    query GetTaskProcessIds($taskId: ID!) {
        tasks(where: { id: $taskId }) {
            edges {
                node {
                    reportedProcesses(orderBy: { field: NAME, direction: DESC }) {
                        edges {
                            node {
                                id
                            }
                        }
                    }
                }
            }
        }
    }
`;

const GET_TASK_PROCESS_DETAIL_QUERY = gql`
    query GetTaskProcessDetail($taskId: ID!, $processId: ID!) {
        tasks(where: { id: $taskId }) {
            edges {
                node {
                    reportedProcesses(where: { id: $processId }, first: 1) {
                        edges {
                            node {
                                id
                                lastModifiedAt
                                principal
                                pid
                                ppid
                                name
                                path
                                status
                            }
                        }
                    }
                }
            }
        }
    }
`;

interface TaskProcessIdsQueryResponse {
    tasks: {
        edges: {
            node: {
                reportedProcesses: {
                    edges: {
                        node: {
                            id: string;
                        };
                    }[];
                };
            };
        }[];
    };
}

interface TaskProcessDetailQueryResponse {
    tasks: {
        edges: {
            node: {
                reportedProcesses: {
                    edges: {
                        node: ProcessNode;
                    }[];
                };
            };
        }[];
    };
}

interface TaskProcessesProps {
    taskId: string;
    hostId: string;
}

const TaskProcesses: FC<TaskProcessesProps> = ({ taskId, hostId }) => {
    const principalColors = Object.values(PrincipalAdminTypes);

    const { data, loading, error } = useQuery<TaskProcessIdsQueryResponse>(
        GET_TASK_PROCESS_IDS_QUERY,
        {
            variables: { taskId },
            fetchPolicy: 'cache-and-network',
        }
    );

    const processIds = useMemo(
        () => data?.tasks?.edges?.[0]?.node?.reportedProcesses?.edges?.map(edge => edge.node.id) || [],
        [data]
    );

    const getVariables = useCallback((id: string) => ({ taskId, processId: id }), [taskId]);

    const extractData = useCallback((response: TaskProcessDetailQueryResponse): ProcessNode | null => {
        return response?.tasks?.edges?.[0]?.node?.reportedProcesses?.edges?.[0]?.node || null;
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
            width: 'minmax(100px,1fr)',
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
            width: 'minmax(50px,0.5fr)',
            render: (process: ProcessNode) => process.pid,
        },
        {
            key: 'ppid',
            label: 'PPID',
            width: 'minmax(50px,0.5fr)',
            render: (process: ProcessNode) => process.ppid,
        },
    ], [principalColors]);

    if (loading && processIds.length === 0) {
        return (
            <EmptyState
                type={EmptyStateType.loading}
                label="Loading processes..."
            />
        );
    }

    if (error) {
        return (
            <EmptyState
                type={EmptyStateType.error}
                label="Error loading processes"
                details={error.message}
            />
        );
    }

    return (
        <div className="flex flex-col gap-2">
            <VirtualizedTable<ProcessNode, TaskProcessDetailQueryResponse>
                items={processIds}
                columns={columns}
                query={GET_TASK_PROCESS_DETAIL_QUERY}
                getVariables={getVariables}
                extractData={extractData}
                estimateRowSize={73}
                overscan={2}
                height="300px"
                minHeight="200px"
                minWidth="600px"
            />
            <div className="flex justify-end py-1">
                <Link
                    to={`/hosts/${hostId}?tab=processes`}
                    className="inline-flex items-center gap-1 text-sm semi-bold text-gray-800 hover:text-purple-800"
                >
                    View host processes
                    <ArrowRight className="w-4 h-4" />
                </Link>
            </div>
        </div>
    );
};

export default TaskProcesses;
