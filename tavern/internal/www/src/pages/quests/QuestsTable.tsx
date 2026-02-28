import { useCallback, useMemo } from "react";
import { useNavigate } from "react-router-dom";
import { formatDistance } from "date-fns";
import UserImageAndName from "../../components/UserImageAndName";
import Badge from "../../components/tavern-base-ui/badge/Badge";
import { VirtualizedTable } from "../../components/tavern-base-ui/virtualized-table/VirtualizedTable";
import { VirtualizedTableColumn } from "../../components/tavern-base-ui/virtualized-table/types";
import { GET_QUEST_DETAIL_QUERY } from "./queries";
import { QuestDetailQueryResponse, QuestDetailNode, GetQuestDetailQueryVariables } from "./types";
import { constructTaskFilterQuery } from "../../utils/constructQueryUtils";
import { useFilters } from "../../context/FilterContext";
import { OrderDirection, TaskOrderField } from "../../utils/enums";

interface QuestsTableProps {
    questIds: string[];
    hasMore?: boolean;
    onLoadMore?: () => void;
}

export const QuestsTable = ({ questIds, hasMore = false, onLoadMore }: QuestsTableProps) => {
    const navigate = useNavigate();
    const { filters } = useFilters();
    const currentDate = useMemo(() => new Date(), []);

    const handleRowClick = useCallback((questId: string) => {
        navigate(`/tasks/${questId}`);
    }, [navigate]);

    const getVariables = useCallback((id: string): GetQuestDetailQueryVariables => {
        const currentTimestamp = new Date();
        const filterQueryTaskFields = constructTaskFilterQuery(filters, currentTimestamp);

        return {
            id,
            whereTotalTask: {
                ...(filterQueryTaskFields?.hasTasksWith || {})
            },
            whereFinishedTask: {
                execFinishedAtNotNil: true,
                ...(filterQueryTaskFields?.hasTasksWith || {})
            },
            whereOutputTask: {
                outputSizeGT: 0,
                ...(filterQueryTaskFields?.hasTasksWith || {})
            },
            whereErrorTask: {
                errorNotNil: true,
                ...(filterQueryTaskFields?.hasTasksWith || {})
            },
            firstTask: 1,
            orderByTask: [{ direction: OrderDirection.Desc, field: TaskOrderField.LastModifiedAt }]
        };
    }, [filters]);

    const extractData = useCallback((response: QuestDetailQueryResponse): QuestDetailNode | null => {
        return response?.quests?.edges?.[0]?.node || null;
    }, []);

    const columns: VirtualizedTableColumn<QuestDetailNode>[] = useMemo(() => [
        {
            key: 'quest-details',
            label: 'Quest details',
            width: 'minmax(200px,2fr)',
            render: (quest) => (
                <div className="flex flex-col min-w-0">
                    <div className="truncate" title={quest.name ?? 'N/A'}>{quest.name ?? 'N/A'}</div>
                    <div className="text-sm flex flex-row gap-1 items-center text-gray-500 truncate" title={quest.tome?.name ?? 'N/A'}>
                        {quest.tome?.name ?? 'N/A'}
                    </div>
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
            key: 'updated',
            label: 'Updated',
            width: 'minmax(100px,1fr)',
            render: (quest) => {
                const lastUpdated = quest.lastUpdatedTask?.edges?.[0]?.node?.lastModifiedAt;
                let formattedTime = 'N/A';
                if (lastUpdated) {
                    try {
                        formattedTime = formatDistance(new Date(lastUpdated), currentDate);
                    } catch {
                        formattedTime = 'Invalid date';
                    }
                }
                return (
                    <div className="text-sm flex items-center min-w-0">
                        <span className="truncate">{formattedTime}</span>
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
            key: 'finished',
            label: 'Finished',
            width: 'minmax(80px,100px)',
            render: (quest) => {
                const finished = quest.tasksFinished?.totalCount ?? 0;
                const total = quest.tasksTotal?.totalCount ?? 0;
                return (
                    <div className="flex items-center">
                        <Badge badgeStyle={{ color: finished < total ? "none" : "green" }}>
                            {finished}/{total}
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
            key: 'output',
            label: 'Output',
            width: 'minmax(80px,100px)',
            render: (quest) => {
                const output = quest.tasksOutput?.totalCount ?? 0;
                return (
                    <div className="flex items-center">
                        <Badge badgeStyle={{ color: output === 0 ? "none" : "purple" }}>
                            {output}
                        </Badge>
                    </div>
                );
            },
            renderSkeleton: () => (
                <div className="flex items-center">
                    <div className="h-6 bg-gray-200 rounded animate-pulse w-8"></div>
                </div>
            ),
        },
        {
            key: 'error',
            label: 'Error',
            width: 'minmax(80px,100px)',
            render: (quest) => {
                const error = quest.tasksError?.totalCount ?? 0;
                return (
                    <div className="flex items-center">
                        <Badge badgeStyle={{ color: error === 0 ? "none" : "red" }}>
                            {error}
                        </Badge>
                    </div>
                );
            },
            renderSkeleton: () => (
                <div className="flex items-center">
                    <div className="h-6 bg-gray-200 rounded animate-pulse w-8"></div>
                </div>
            ),
        },
        {
            key: 'creator',
            label: 'Creator',
            width: 'minmax(120px,150px)',
            render: (quest) => (
                <div className="flex items-center">
                    <UserImageAndName userData={quest.creator} />
                </div>
            ),
            renderSkeleton: () => (
                <div className="flex items-center">
                    <div className="h-8 w-8 bg-gray-200 rounded-full animate-pulse"></div>
                </div>
            ),
        },
    ], [currentDate]);

    return (
        <VirtualizedTable<QuestDetailNode, QuestDetailQueryResponse>
            items={questIds}
            columns={columns}
            query={GET_QUEST_DETAIL_QUERY}
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
