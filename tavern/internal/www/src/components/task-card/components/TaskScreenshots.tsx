import { FC, useCallback, useMemo, useState } from "react";
import { gql, useQuery } from "@apollo/client";
import { Link } from "react-router-dom";
import { ArrowDownToLine, ArrowRight, ChevronLeft, ChevronRight } from "lucide-react";
import Button from "../../tavern-base-ui/button/Button";
import { EmptyState, EmptyStateType } from "../../tavern-base-ui/EmptyState";
import Tooltip from "../../tavern-base-ui/Tooltip";
import { VirtualizedCardList } from "../../tavern-base-ui/virtualized-card-list";
import { capturedAt, imageURL, thumbnailURL } from "../../../pages/host-details/screenshots-tab/utils";
import { ScreenshotNode } from "../../../pages/host-details/screenshots-tab/types";
import { formatBytes } from "../../../utils/utils";

type TaskScreenshot = Pick<ScreenshotNode, "id" | "name" | "size" | "createdAt">;

export const GET_TASK_SCREENSHOTS = gql`
    query GetTaskScreenshots($taskId: ID!) {
        tasks(where: { id: $taskId }) {
            edges {
                node {
                    screenshots(orderBy: { field: CREATED_AT, direction: DESC }) {
                        edges { node { id name size createdAt } }
                    }
                }
            }
        }
    }
`;

interface TaskScreenshotsResponse {
    tasks: { edges: Array<{ node: { screenshots: { edges: Array<{ node: TaskScreenshot }> } } }> };
}

interface TaskScreenshotsProps {
    taskId: string;
    hostId: string;
}

const TaskScreenshots: FC<TaskScreenshotsProps> = ({ taskId, hostId }) => {
    const [selectedId, setSelectedId] = useState<string | null>(null);
    const { data, loading, error } = useQuery<TaskScreenshotsResponse>(GET_TASK_SCREENSHOTS, {
        variables: { taskId },
        fetchPolicy: "cache-and-network",
    });

    const connection = data?.tasks?.edges?.[0]?.node?.screenshots;
    const screenshots = useMemo(() => connection?.edges?.map(edge => edge.node) || [], [connection]);
    const screenshotIds = useMemo(() => screenshots.map(screenshot => screenshot.id), [screenshots]);
    const screenshotsById = useMemo(() => new Map(screenshots.map(screenshot => [screenshot.id, screenshot])), [screenshots]);
    const selectedIndex = Math.max(0, screenshots.findIndex(screenshot => screenshot.id === selectedId));
    const selected = screenshots[selectedIndex];

    const renderThumbnail = useCallback(({ itemId }: { itemId: string }) => {
        const screenshot = screenshotsById.get(itemId);
        if (!screenshot) return null;
        return (
            <div className={`flex h-[61px] items-center gap-2 px-2 py-2 border-b border-gray-100 ${screenshot.id === selected?.id ? "bg-purple-50 border-l-2 border-l-purple-600" : "hover:bg-gray-50"}`}>
                <button
                    type="button"
                    onClick={() => setSelectedId(screenshot.id)}
                    aria-label={`Preview ${screenshot.name}`}
                    aria-current={screenshot.id === selected?.id ? "true" : undefined}
                    className="flex items-center gap-2 min-w-0 flex-1 text-left"
                >
                    <img src={thumbnailURL(screenshot.id)} alt="" loading="lazy" className="w-16 h-11 object-contain bg-gray-100 rounded-sm shrink-0" />
                    <span className="min-w-0 flex-1">
                        <span className="block truncate text-xs text-gray-900" title={screenshot.name}>{screenshot.name}</span>
                        <span className="block truncate text-xs text-gray-600">{capturedAt(screenshot.createdAt)} · {formatBytes(screenshot.size)}</span>
                    </span>
                </button>
                <Tooltip label={`Download ${screenshot.name}`}>
                    <a href={imageURL(screenshot.id)} download={screenshot.name} aria-label={`Download ${screenshot.name}`} className="p-1 rounded text-gray-600 hover:bg-gray-100 hover:text-purple-700">
                        <ArrowDownToLine className="w-4 h-4" />
                    </a>
                </Tooltip>
            </div>
        );
    }, [screenshotsById, selected?.id]);

    if (loading && screenshots.length === 0) {
        return <EmptyState type={EmptyStateType.loading} label="Loading screenshots..." />;
    }
    if (error) {
        return <EmptyState type={EmptyStateType.error} label="Error loading screenshots" details={error.message} />;
    }
    if (!selected) {
        return <EmptyState type={EmptyStateType.noData} label="No screenshots reported" />;
    }

    return (
        <div className="flex flex-col gap-2 mt-2">
            <div className="grid grid-cols-1 lg:grid-cols-[minmax(260px,300px)_minmax(0,1fr)] gap-4 lg:min-h-[60vh] [--screenshot-list-height:min(25dvh,220px)] lg:[--screenshot-list-height:60vh] [--screenshot-list-min-height:0px] lg:[--screenshot-list-min-height:320px]">
                <div className="order-2 lg:order-1 border border-gray-200 rounded-md overflow-hidden flex flex-col min-h-0">
                    <VirtualizedCardList
                        items={screenshotIds}
                        renderCard={renderThumbnail}
                        estimateCardSize={61}
                        dynamicSizing={false}
                        gap={0}
                        padding="0"
                        height="var(--screenshot-list-height)"
                        minHeight="var(--screenshot-list-min-height)"
                    />
                </div>
                <div className="order-1 lg:order-2 border border-gray-200 rounded-md overflow-hidden flex flex-col min-w-0 h-[30dvh] min-h-[220px] lg:h-[60vh] lg:min-h-[320px]">
                    <div className="flex items-center justify-between gap-2 px-4 py-2 border-b border-gray-200 text-sm">
                        <span className="truncate font-medium text-gray-900" title={selected.name}>{selected.name}</span>
                        <div className="flex items-center gap-1 shrink-0">
                            <span className="text-xs text-gray-600 mr-2">{selectedIndex + 1} of {screenshots.length}</span>
                            <Button buttonVariant="ghost" buttonStyle={{ color: "gray", size: "square_sm" }} leftIcon={<ChevronLeft className="w-4 h-4" />} aria-label="Previous screenshot" disabled={selectedIndex === 0} onClick={() => setSelectedId(screenshots[selectedIndex - 1].id)} />
                            <Button buttonVariant="ghost" buttonStyle={{ color: "gray", size: "square_sm" }} leftIcon={<ChevronRight className="w-4 h-4" />} aria-label="Next screenshot" disabled={selectedIndex === screenshots.length - 1} onClick={() => setSelectedId(screenshots[selectedIndex + 1].id)} />
                            <Tooltip label={`Download ${selected.name}`}>
                                <a href={imageURL(selected.id)} download={selected.name} aria-label={`Download ${selected.name}`} className="p-1 rounded text-gray-600 hover:bg-gray-100 hover:text-purple-700">
                                    <ArrowDownToLine className="w-4 h-4" />
                                </a>
                            </Tooltip>
                        </div>
                    </div>
                    <div className="flex-1 min-h-0 overflow-hidden bg-gray-100 flex items-center justify-center p-4">
                        <img src={imageURL(selected.id)} alt={selected.name} className="max-w-full max-h-full object-contain shadow-sm" />
                    </div>
                    <div className="flex flex-wrap gap-x-6 gap-y-1 px-4 py-3 border-t border-gray-200 text-xs text-gray-600">
                        <span>Captured <b className="text-gray-900 font-medium">{capturedAt(selected.createdAt)}</b></span>
                        <span>Size <b className="text-gray-900 font-medium">{formatBytes(selected.size)}</b></span>
                    </div>
                </div>
            </div>
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
