import React from "react";
import { formatDistance } from "date-fns";
import { ChevronRight, ChevronDown } from "lucide-react";
import UserImageAndName from "../../../components/UserImageAndName";
import Badge from "../../../components/tavern-base-ui/badge/Badge";
import { QuestEdge } from "../../../utils/interfacesQuery";

export type RowDataProps = {
    quests: QuestEdge[];
    navigate: (path: string) => void;
    currentDate: Date;
    expandedRows: Set<string>;
    toggleRow: (id: string) => void;
}

type QuestRowProps = RowDataProps & {
    index: number;
    style: React.CSSProperties;
};

// Must match JulesQuests.tsx
const COL_WIDTHS = {
    chevron: "w-[40px] flex-none flex justify-center",
    name: "flex-[2_1_200px]",
    updated: "w-[120px] flex-none",
    finished: "w-[80px] flex-none",
    output: "w-[80px] flex-none",
    error: "w-[80px] flex-none",
    creator: "w-[150px] flex-none",
};

export const QuestRow = ({ index, style, quests, navigate, currentDate, expandedRows, toggleRow }: QuestRowProps) => {
    const questEdge = quests[index];
    if (!questEdge) return <div style={style} />;

    const quest = questEdge.node;
    const isExpanded = expandedRows.has(quest.id);

    // Derived fields
    const lastUpdatedRaw = quest.lastUpdatedTask?.edges?.[0]?.node?.lastModifiedAt;
    const updatedDate = lastUpdatedRaw ? new Date(lastUpdatedRaw) : null;

    const totalTasks = quest.tasksTotal?.totalCount ?? 0;
    const finishedTasks = quest.tasksFinished?.totalCount ?? 0;
    const errorTasks = quest.tasksError?.totalCount ?? 0;
    const outputTasks = quest.tasksOutput?.totalCount ?? 0;

    const isFinished = totalTasks > 0 && finishedTasks === totalTasks;

    // Padding matches header usually (px-6)
    const cellClass = "px-4 sm:px-6 py-4 flex items-center overflow-hidden h-full";
    const nameCellClass = "px-4 sm:px-6 py-4 flex items-center overflow-hidden text-ellipsis whitespace-nowrap h-full";

    return (
        <div style={style} className="bg-white border-b border-gray-100 hover:bg-gray-50 flex flex-col box-border" data-testid="quest-row">
            <div className="flex flex-row items-center h-[80px] w-full">
                <div className={`${COL_WIDTHS.chevron} cursor-pointer h-full items-center`} onClick={(e) => { e.stopPropagation(); toggleRow(quest.id); }}>
                    {isExpanded ? <ChevronDown size={16} /> : <ChevronRight size={16} />}
                </div>

                <div className={`${COL_WIDTHS.name} ${nameCellClass} font-medium text-blue-600 cursor-pointer`} onClick={() => navigate(`/quests/${quest.id}`)}>
                    {quest.name || "Untitled Quest"}
                </div>

                <div className={`${COL_WIDTHS.updated} ${cellClass} text-sm text-gray-500`}>
                    {updatedDate ? formatDistance(updatedDate, currentDate, { addSuffix: true }) : "-"}
                </div>

                <div className={`${COL_WIDTHS.finished} ${cellClass}`}>
                     {isFinished ? <Badge badgeStyle={{ color: "green" }}>Yes</Badge> : <Badge badgeStyle={{ color: "gray" }}>{finishedTasks}/{totalTasks}</Badge>}
                </div>

                <div className={`${COL_WIDTHS.output} ${cellClass}`}>
                     {outputTasks > 0 ? <Badge badgeStyle={{ color: "purple" }}>{outputTasks}</Badge> : "-"}
                </div>

                <div className={`${COL_WIDTHS.error} ${cellClass}`}>
                     {errorTasks > 0 ? <Badge badgeStyle={{ color: "red" }}>{errorTasks}</Badge> : "-"}
                </div>

                <div className={`${COL_WIDTHS.creator} ${cellClass}`}>
                    {quest.creator && <UserImageAndName userData={quest.creator} />}
                </div>
            </div>

            {/* Expanded Details */}
            {isExpanded && (
                <div className="bg-gray-50 px-6 py-4 border-t border-gray-100 text-sm h-[220px] overflow-y-auto">
                    <div className="grid grid-cols-2 gap-4">
                        <div>
                            <span className="font-semibold text-gray-500 text-xs uppercase block mb-1">ID</span>
                            <span className="font-mono bg-white px-2 py-1 rounded border border-gray-200 block w-fit">{quest.id}</span>
                        </div>
                        <div>
                            <span className="font-semibold text-gray-500 text-xs uppercase block mb-1">Last Updated</span>
                            <span>{updatedDate ? updatedDate.toLocaleString() : "-"}</span>
                        </div>
                        {quest.parameters && (
                            <div className="col-span-2">
                                <span className="font-semibold text-gray-500 text-xs uppercase block mb-1">Parameters</span>
                                <pre className="bg-white p-3 rounded border border-gray-200 overflow-x-auto text-xs font-mono">
                                    {quest.parameters}
                                </pre>
                            </div>
                        )}
                        <div className="col-span-2">
                             <span className="font-semibold text-gray-500 text-xs uppercase block mb-1">Full Node</span>
                             <pre className="bg-white p-3 rounded border border-gray-200 overflow-x-auto text-xs font-mono">
                                 {JSON.stringify(quest, null, 2)}
                             </pre>
                        </div>
                    </div>
                </div>
            )}
        </div>
    );
};
