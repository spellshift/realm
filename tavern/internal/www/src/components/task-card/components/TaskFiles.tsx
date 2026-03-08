import { FC, useCallback, useMemo } from "react";
import { gql, useQuery } from "@apollo/client";
import { Link } from "react-router-dom";
import { ArrowDownToLine, ArrowRight } from "lucide-react";
import Tooltip from "../../tavern-base-ui/Tooltip";
import Button from "../../tavern-base-ui/button/Button";
import { VirtualizedTable } from "../../tavern-base-ui/virtualized-table/VirtualizedTable";
import { VirtualizedTableColumn } from "../../tavern-base-ui/virtualized-table/types";
import { EmptyState, EmptyStateType } from "../../tavern-base-ui/EmptyState";
import { formatBytes } from "../../../utils/utils";

const GET_TASK_FILE_IDS_QUERY = gql`
    query GetTaskFileIds($taskId: ID!) {
        tasks(where: { id: $taskId }) {
            edges {
                node {
                    reportedFiles(orderBy: { field: NAME, direction: ASC }) {
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

const GET_TASK_FILE_DETAIL_QUERY = gql`
    query GetTaskFileDetail($taskId: ID!, $fileId: ID!) {
        tasks(where: { id: $taskId }) {
            edges {
                node {
                    reportedFiles(where: { id: $fileId }, first: 1) {
                        edges {
                            node {
                                id
                                path
                                owner
                                group
                                permissions
                                size
                                hash
                            }
                        }
                    }
                }
            }
        }
    }
`;

interface FileNode {
    id: string;
    path: string;
    owner: string | null;
    group: string | null;
    permissions: string | null;
    size: number;
    hash: string | null;
}

interface TaskFileIdsQueryResponse {
    tasks: {
        edges: {
            node: {
                reportedFiles: {
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

interface TaskFileDetailQueryResponse {
    tasks: {
        edges: {
            node: {
                reportedFiles: {
                    edges: {
                        node: FileNode;
                    }[];
                };
            };
        }[];
    };
}

interface TaskFilesProps {
    taskId: string;
    hostId: string;
}

const TaskFiles: FC<TaskFilesProps> = ({ taskId, hostId }) => {
    const { data, loading, error } = useQuery<TaskFileIdsQueryResponse>(
        GET_TASK_FILE_IDS_QUERY,
        {
            variables: { taskId },
            fetchPolicy: 'cache-and-network',
        }
    );

    const fileIds = useMemo(
        () => data?.tasks?.edges?.[0]?.node?.reportedFiles?.edges?.map(edge => edge.node.id) || [],
        [data]
    );

    const getVariables = useCallback((id: string) => ({ taskId, fileId: id }), [taskId]);

    const extractData = useCallback((response: TaskFileDetailQueryResponse): FileNode | null => {
        return response?.tasks?.edges?.[0]?.node?.reportedFiles?.edges?.[0]?.node || null;
    }, []);

    const columns: VirtualizedTableColumn<FileNode>[] = useMemo(() => [
        {
            key: 'path',
            label: 'Path',
            width: 'minmax(80px,2fr)',
            render: (file: FileNode) => (
                <Tooltip label={file.path} isDisabled={!file.path}>
                    <div className="truncate text-sm">
                        {file.path}
                    </div>
                </Tooltip>
            ),
        },
        {
            key: 'owner',
            label: 'Owner',
            width: 'minmax(80px,0.75fr)',
            render: (file: FileNode) => file.owner || "-",
        },
        {
            key: 'permissions',
            label: 'Permissions',
            width: 'minmax(80px,0.75fr)',
            render: (file: FileNode) => (
                <code className="text-sm">{file.permissions || "-"}</code>
            ),
        },
        {
            key: 'size',
            label: 'Size',
            width: 'minmax(60px,0.5fr)',
            render: (file: FileNode) => (
                <Tooltip label={`${file.size} bytes`}>
                    <span>{formatBytes(file.size)}</span>
                </Tooltip>
            ),
        },
        {
            key: 'actions',
            label: '',
            width: 'minmax(30px,0.3fr)',
            render: (file: FileNode) => (
                <Tooltip label="Download">
                    <a href={`/cdn/hostfiles/${file.id}`} download onClick={(e) => e.stopPropagation()}>
                        <Button
                            buttonVariant="ghost"
                            buttonStyle={{ color: "gray", size: "xs" }}
                            leftIcon={<ArrowDownToLine className="w-4 h-4" />}
                            aria-label="Download"
                        />
                    </a>
                </Tooltip>
            ),
        },
    ], []);

    if (loading && fileIds.length === 0) {
        return (
            <EmptyState
                type={EmptyStateType.loading}
                label="Loading files..."
            />
        );
    }

    if (error) {
        return (
            <EmptyState
                type={EmptyStateType.error}
                label="Error loading files"
                details={error.message}
            />
        );
    }

    return (
        <div className="flex flex-col gap-2">
            <VirtualizedTable<FileNode, TaskFileDetailQueryResponse>
                items={fileIds}
                columns={columns}
                query={GET_TASK_FILE_DETAIL_QUERY}
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
                    to={`/hosts/${hostId}?tab=files`}
                    className="inline-flex items-center gap-1 text-sm semi-bold text-gray-800 hover:text-purple-800"
                >
                    View host files
                    <ArrowRight className="w-4 h-4" />
                </Link>
            </div>
        </div>
    );
};

export default TaskFiles;
