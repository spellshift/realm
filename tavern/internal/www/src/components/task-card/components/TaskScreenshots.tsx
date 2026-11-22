import { FC, useCallback, useMemo } from "react";
import { gql, useQuery } from "@apollo/client";
import { Link } from "react-router-dom";
import { ArrowDownToLine, ArrowRight } from "lucide-react";
import { EmptyState, EmptyStateType } from "../../tavern-base-ui/EmptyState";
import Tooltip from "../../tavern-base-ui/Tooltip";
import { VirtualizedTable } from "../../tavern-base-ui/virtualized-table/VirtualizedTable";
import { VirtualizedTableColumn } from "../../tavern-base-ui/virtualized-table/types";
import { capturedAt, imageURL, thumbnailURL } from "../../../pages/host-details/screenshots-tab/utils";
import { ScreenshotNode } from "../../../pages/host-details/screenshots-tab/types";
import { formatBytes } from "../../../utils/utils";

const GET_TASK_SCREENSHOT_IDS = gql`
    query GetTaskScreenshotIds($taskId: ID!) {
        tasks(where: { id: $taskId }) {
            edges {
                node {
                    screenshots(orderBy: { field: CREATED_AT, direction: DESC }) {
                        edges { node { id } }
                    }
                }
            }
        }
    }
`;

const GET_TASK_SCREENSHOT_DETAIL = gql`
    query GetTaskScreenshotDetail($taskId: ID!, $screenshotId: ID!) {
        tasks(where: { id: $taskId }) {
            edges {
                node {
                    screenshots(where: { id: $screenshotId }, first: 1) {
                        edges {
                            node { id name size createdAt }
                        }
                    }
                }
            }
        }
    }
`;

interface TaskScreenshotsResponse {
    tasks: { edges: Array<{ node: { screenshots: { edges: Array<{ node: ScreenshotNode }> } } }> };
}

interface TaskScreenshotsProps {
    taskId: string;
    hostId: string;
}

const TaskScreenshots: FC<TaskScreenshotsProps> = ({ taskId, hostId }) => {
    const { data, loading, error } = useQuery<TaskScreenshotsResponse>(GET_TASK_SCREENSHOT_IDS, {
        variables: { taskId },
        fetchPolicy: "cache-and-network",
    });

    const screenshotIds = useMemo(
        () => data?.tasks?.edges?.[0]?.node?.screenshots?.edges?.map(edge => edge.node.id) || [],
        [data]
    );
    const getVariables = useCallback((id: string) => ({ taskId, screenshotId: id }), [taskId]);
    const extractData = useCallback((response: TaskScreenshotsResponse): ScreenshotNode | null =>
        response?.tasks?.edges?.[0]?.node?.screenshots?.edges?.[0]?.node || null, []);

    const columns: VirtualizedTableColumn<ScreenshotNode>[] = useMemo(() => [
        {
            key: "name",
            label: "Screenshot",
            width: "minmax(180px,2fr)",
            render: screenshot => (
                <div className="flex items-center gap-3 min-w-0">
                    <img src={thumbnailURL(screenshot.id)} alt="" loading="lazy" className="w-16 h-11 object-contain bg-gray-100 rounded-sm shrink-0" />
                    <Tooltip label={screenshot.name}>
                        <span className="truncate text-sm">{screenshot.name}</span>
                    </Tooltip>
                </div>
            ),
        },
        {
            key: "createdAt",
            label: "Captured",
            width: "minmax(120px,1fr)",
            render: screenshot => capturedAt(screenshot.createdAt),
        },
        {
            key: "size",
            label: "Size",
            width: "minmax(60px,0.5fr)",
            render: screenshot => formatBytes(screenshot.size),
        },
        {
            key: "download",
            label: "",
            width: "40px",
            render: screenshot => (
                <Tooltip label={`Download ${screenshot.name}`}>
                    <a href={imageURL(screenshot.id)} download={screenshot.name} aria-label={`Download ${screenshot.name}`} className="p-1 rounded text-gray-600 hover:bg-gray-100 hover:text-purple-700">
                        <ArrowDownToLine className="w-4 h-4" />
                    </a>
                </Tooltip>
            ),
        },
    ], []);

    if (loading && screenshotIds.length === 0) {
        return <EmptyState type={EmptyStateType.loading} label="Loading screenshots..." />;
    }

    if (error) {
        return <EmptyState type={EmptyStateType.error} label="Error loading screenshots" details={error.message} />;
    }

    return (
        <div className="flex flex-col gap-2">
            <VirtualizedTable<ScreenshotNode, TaskScreenshotsResponse>
                items={screenshotIds}
                columns={columns}
                query={GET_TASK_SCREENSHOT_DETAIL}
                getVariables={getVariables}
                extractData={extractData}
                estimateRowSize={73}
                overscan={2}
                height="300px"
                minHeight="200px"
                minWidth="500px"
            />
            {hostId && (
                <div className="flex justify-end py-2">
                    <Link to={`/hosts/${hostId}?tab=screenshots`} className="inline-flex items-center gap-1 text-sm semi-bold text-gray-800 hover:text-purple-800">
                        View host screenshots
                        <ArrowRight className="w-4 h-4" />
                    </Link>
                </div>
            )}
        </div>
    );
};

export default TaskScreenshots;
