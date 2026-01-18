import React, { CSSProperties, useCallback, useState } from "react";
import { List, ListProps, RowComponentProps } from "react-window";
import { AutoSizer } from "react-virtualized-auto-sizer";
import { useNavigate } from "react-router-dom";
import { formatDistance } from "date-fns";
import { ChevronRight, ChevronDown } from "lucide-react";

import { PageWrapper } from "../../components/page-wrapper";
import { PageNavItem } from "../../utils/enums";
import QuestHeader from "./components/QuestHeader";
import { FilterControls, FilterPageType } from "../../context/FilterContext/index";
import { SortingControls } from "../../context/SortContext/index";
import { useJulesQuests } from "./useJulesQuests";
import UserImageAndName from "../../components/UserImageAndName";
import Badge from "../../components/tavern-base-ui/badge/Badge";
import { QuestEdge, QuestNode, UserNode } from "../../utils/interfacesQuery";

// --- Styles ---
const HEADER_CELL_CLASS = "px-4 sm:px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider overflow-hidden text-ellipsis whitespace-nowrap";
const ROW_CELL_CLASS = "px-4 sm:px-6 py-4 flex items-center overflow-hidden";
const ROW_CONTENT_CLASS = "hover:bg-gray-100 flex flex-row items-center bg-white h-[80px]";

// We will use flexbox for columns.
const COL_WIDTHS = {
    chevron: "w-[40px] flex-none flex justify-center",
    name: "flex-[2_1_200px]",
    updated: "w-[120px] flex-none",
    finished: "w-[80px] flex-none",
    output: "w-[80px] flex-none",
    error: "w-[80px] flex-none",
    creator: "w-[150px] flex-none",
};

// Define the custom props we are passing to the row
type RowDataProps = {
    quests: QuestEdge[];
    navigate: (path: string) => void;
    currentDate: Date;
    expandedRows: Set<string>;
    toggleRow: (id: string) => void;
}

const JulesQuests = () => {
    const {
        data,
        loading,
        error,
        loadMore,
        hasNextPage
    } = useJulesQuests();

    const navigate = useNavigate();
    const quests = data?.quests?.edges || [];
    const itemCount = quests.length;

    // State for expanded rows
    const [expandedRows, setExpandedRows] = useState<Set<string>>(new Set());

    const toggleRow = (id: string) => {
        setExpandedRows(prev => {
            const next = new Set(prev);
            if (next.has(id)) {
                next.delete(id);
            } else {
                next.add(id);
            }
            return next;
        });
    };

    // Handle Infinite Scroll
    const onRowsRendered = ({ stopIndex }: { stopIndex: number }) => {
        if (hasNextPage && stopIndex >= itemCount - 5) {
            loadMore();
        }
    };

    const currentDate = new Date();

    // Row component
    const Row = ({ index, style, expandedRows, toggleRow, quests, navigate, currentDate }: RowComponentProps<RowDataProps>) => {
        const row = quests[index];
        if (!row) return <div style={style}>Loading...</div>;

        const quest = row.node;
        const isExpanded = expandedRows.has(quest.id);

        const lastUpdated = quest?.lastUpdatedTask?.edges?.[0]?.node?.lastModifiedAt;
        let timeString = 'N/A';
        try {
            if (lastUpdated) {
                timeString = formatDistance(new Date(lastUpdated), currentDate);
            }
        } catch {
            timeString = 'Invalid date';
        }

        const finished = quest?.tasksFinished?.totalCount ?? 0;
        const allTasks = quest?.tasksTotal?.totalCount ?? 0;
        const finishedColor = finished < allTasks ? "none" : "green";

        const outputCount = quest?.tasksOutput?.totalCount ?? 0;
        const outputColor = outputCount === 0 ? "none" : 'purple';

        const errorCount = quest?.tasksError?.totalCount ?? 0;
        const errorColor = errorCount === 0 ? "none" : 'red';

        const creator = quest?.creator as UserNode;

        return (
            <div
                style={style}
                className="flex flex-col border-b border-gray-200 bg-white"
            >
                {/* Main Row Content */}
                <div
                    className={ROW_CONTENT_CLASS}
                    onClick={() => navigate(`/tasks/${quest?.id}`)}
                    role="button"
                    tabIndex={0}
                    onKeyDown={(e) => {
                        if (e.key === 'Enter') navigate(`/tasks/${quest?.id}`);
                    }}
                >
                    {/* Chevron */}
                    <div
                        data-testid={`expand-toggle-${quest.id}`}
                        className={`${COL_WIDTHS.chevron} cursor-pointer p-2 hover:bg-gray-200 rounded-full mx-2`}
                        onClick={(e) => {
                            e.stopPropagation();
                            toggleRow(quest.id);
                        }}
                    >
                        {isExpanded ? <ChevronDown size={20} /> : <ChevronRight size={20} />}
                    </div>

                    {/* Name */}
                    <div className={`${ROW_CELL_CLASS} ${COL_WIDTHS.name}`}>
                        <div className="flex flex-col truncate w-full">
                            <div className="truncate font-medium text-gray-900">{quest?.name ?? 'N/A'}</div>
                            <div className="text-sm flex flex-row gap-1 items-center text-gray-500 truncate">
                                {quest?.tome?.name ?? 'N/A'}
                            </div>
                        </div>
                    </div>

                    {/* Updated */}
                    <div className={`${ROW_CELL_CLASS} ${COL_WIDTHS.updated}`}>
                        <span className="text-sm text-gray-500">{timeString}</span>
                    </div>

                    {/* Finished */}
                    <div className={`${ROW_CELL_CLASS} ${COL_WIDTHS.finished}`}>
                        <Badge badgeStyle={{ color: finishedColor }}>
                            {finished}/{allTasks}
                        </Badge>
                    </div>

                    {/* Output */}
                    <div className={`${ROW_CELL_CLASS} ${COL_WIDTHS.output}`}>
                        <Badge badgeStyle={{ color: outputColor }}>
                            {outputCount}
                        </Badge>
                    </div>

                    {/* Error */}
                    <div className={`${ROW_CELL_CLASS} ${COL_WIDTHS.error}`}>
                        <Badge badgeStyle={{ color: errorColor }}>
                            {errorCount}
                        </Badge>
                    </div>

                    {/* Creator */}
                    <div className={`${ROW_CELL_CLASS} ${COL_WIDTHS.creator}`}>
                        {creator && <UserImageAndName userData={creator} />}
                    </div>
                </div>

                {/* Expanded Details */}
                {isExpanded && (
                    <div className="flex-1 bg-gray-50 p-6 border-t border-gray-100 shadow-inner overflow-auto" onClick={(e) => e.stopPropagation()}>
                        <div className="grid grid-cols-2 gap-4 text-sm">
                            <div>
                                <h4 className="font-semibold text-gray-700 mb-2">Details</h4>
                                <div className="space-y-1">
                                    <p><span className="text-gray-500">ID:</span> {quest.id}</p>
                                    <p><span className="text-gray-500">Status:</span> {finished === allTasks ? 'Completed' : 'In Progress'}</p>
                                    <p><span className="text-gray-500">Tasks:</span> {allTasks}</p>
                                </div>
                            </div>
                            <div>
                                <h4 className="font-semibold text-gray-700 mb-2">Description</h4>
                                <p className="text-gray-600 whitespace-pre-wrap">{quest.tome?.description || "No description provided."}</p>
                            </div>
                        </div>
                        <div className="mt-4">
                             <button
                                className="text-blue-600 hover:text-blue-800 text-sm font-medium"
                                onClick={() => navigate(`/tasks/${quest?.id}`)}
                             >
                                View all tasks &rarr;
                             </button>
                        </div>
                    </div>
                )}
            </div>
        );
    };

    const getItemSize = (index: number, props: RowDataProps) => {
        const quest = props.quests[index];
        if (!quest) return 80;
        return props.expandedRows.has(quest.node.id) ? 300 : 80;
    };

    return (
        <PageWrapper currNavItem={PageNavItem.quests}>
            <div className="flex flex-col h-[calc(100vh-4rem)] gap-4">
                <QuestHeader />

                {/* Controls Section (Filters/Sorts) */}
                <div className="flex flex-row justify-between items-center bg-white p-4 rounded-md shadow-sm border border-gray-200">
                    <div className="flex-1">
                        <FilterControls type={FilterPageType.QUEST} />
                    </div>
                    <div className="ml-4">
                        <SortingControls type={PageNavItem.quests} />
                    </div>
                </div>

                {/* Table Container */}
                <div className="flex-1 flex flex-col bg-white rounded-md shadow overflow-hidden border border-gray-200">
                    {/* Header */}
                    <div className="flex flex-row bg-gray-50 border-b border-gray-200 shrink-0 pr-[15px]">
                        <div className={`${HEADER_CELL_CLASS} ${COL_WIDTHS.chevron}`}></div>
                        <div className={`${HEADER_CELL_CLASS} ${COL_WIDTHS.name}`}>Quest details</div>
                        <div className={`${HEADER_CELL_CLASS} ${COL_WIDTHS.updated}`}>Updated</div>
                        <div className={`${HEADER_CELL_CLASS} ${COL_WIDTHS.finished}`}>Finished</div>
                        <div className={`${HEADER_CELL_CLASS} ${COL_WIDTHS.output}`}>Output</div>
                        <div className={`${HEADER_CELL_CLASS} ${COL_WIDTHS.error}`}>Error</div>
                        <div className={`${HEADER_CELL_CLASS} ${COL_WIDTHS.creator}`}>Creator</div>
                    </div>

                    {/* Body */}
                    <div className="flex-1 min-h-0">
                        {loading && itemCount === 0 ? (
                            <div className="flex items-center justify-center h-full text-gray-500">
                                Loading...
                            </div>
                        ) : error ? (
                            <div className="flex items-center justify-center h-full text-red-500">
                                Error: {error.message}
                            </div>
                        ) : (
                            <AutoSizer renderProp={({ height, width }) => (
                                <List<RowDataProps>
                                    style={{ height: height ?? 0, width: width ?? 0 }}
                                    rowCount={itemCount}
                                    rowHeight={getItemSize}
                                    onRowsRendered={onRowsRendered}
                                    rowComponent={Row}
                                    rowProps={{ quests, navigate, currentDate, expandedRows, toggleRow }}
                                    className="scrollbar-thin scrollbar-thumb-gray-300 scrollbar-track-transparent"
                                />
                            )} />
                        )}
                    </div>

                    {/* Footer / Status */}
                    <div className="bg-gray-50 px-4 py-2 border-t border-gray-200 text-xs text-gray-500 flex justify-between">
                         <div>Total: {data?.quests?.totalCount ?? 0}</div>
                         <div>{loading ? 'Refreshing...' : 'Updated'}</div>
                    </div>
                </div>
            </div>
        </PageWrapper>
    );
};

export default JulesQuests;
