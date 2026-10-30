import React, { useState, useMemo, useEffect } from "react";
import { MagnifyingGlassIcon, XMarkIcon } from "@heroicons/react/24/outline";
import { History } from "lucide-react";
import { ShellPivotItem, ShellTaskItem } from "../hooks/useShellData";
import { stripInternalMacro } from "../hooks/shellUtils";
import HistoryTaskCard, {
    AggregatedHistoryGroup,
    NormalizedTaskItem,
    NormalizedTaskStatus,
    summarizeText,
} from "./HistoryTaskCard";

export { summarizeText };

export type HistoryStatusFilter = "all" | "active" | "completed" | "failed";

interface ShellHistoryDrawerProps {
    isOpen: boolean;
    onClose: () => void;
    shellTasks: ShellTaskItem[];
    allShellTasks?: ShellTaskItem[];
    currentShellId?: string;
    localHistory?: string[];
    pivots?: ShellPivotItem[];
    initialStatusFilter?: HistoryStatusFilter;
    focusQueueTrigger?: number;
    onRunCode: (code: string) => void;
    onLoadIntoComposer: (code: string) => void;
    onOpenPivotTab?: (type: "pty" | "ssh", target: string, pivotId?: string) => void;
}

const STATUS_FILTERS: { key: HistoryStatusFilter; label: string }[] = [
    { key: "all", label: "All" },
    { key: "active", label: "Queued / Running" },
    { key: "completed", label: "Completed" },
    { key: "failed", label: "Failed" },
];

const resolveTaskStatus = (task: ShellTaskItem): NormalizedTaskStatus => {
    const isFailed = Boolean(task.error && task.error.trim().length > 0);
    if (isFailed) {
        return "failed";
    }
    const isFinished = Boolean(task.execFinishedAt) || Boolean(task.output);
    if (isFinished) {
        return "completed";
    }
    const isRunning = Boolean(task.claimedAt || task.execStartedAt);
    if (isRunning) {
        return "running";
    }
    return "queued";
};

const EmptyHistoryState: React.FC = () => (
    <div className="text-center py-8 text-xs text-gray-500">No matching commands found.</div>
);

const HistoryGroupList: React.FC<{
    groups: AggregatedHistoryGroup[];
    currentShellId?: string;
    onRunCode: (code: string) => void;
    onLoadIntoComposer: (code: string) => void;
}> = ({ groups, currentShellId, onRunCode, onLoadIntoComposer }) => (
    <>
        {groups.map((group) => (
            <HistoryTaskCard
                key={group.key}
                group={group}
                currentShellId={currentShellId}
                onRunCode={onRunCode}
                onLoadIntoComposer={onLoadIntoComposer}
            />
        ))}
    </>
);

const ShellHistoryDrawer: React.FC<ShellHistoryDrawerProps> = ({
    isOpen,
    onClose,
    shellTasks,
    allShellTasks,
    currentShellId,
    initialStatusFilter = "all",
    focusQueueTrigger = 0,
    onRunCode,
    onLoadIntoComposer,
}) => {
    const [searchQuery, setSearchQuery] = useState("");
    const [statusFilter, setStatusFilter] = useState<HistoryStatusFilter>(initialStatusFilter);
    const [currentSessionOnly, setCurrentSessionOnly] = useState<boolean>(false);

    useEffect(() => {
        if (focusQueueTrigger > 0) {
            setStatusFilter("active");
        }
    }, [focusQueueTrigger]);

    const sourceTasks = useMemo<ShellTaskItem[]>(() => {
        const pool =
            !currentSessionOnly && allShellTasks && allShellTasks.length > 0
                ? allShellTasks
                : shellTasks;

        if (currentSessionOnly && currentShellId) {
            return pool.filter(
                (t) => !t.shellId || String(t.shellId) === String(currentShellId)
            );
        }
        return pool;
    }, [currentSessionOnly, allShellTasks, shellTasks, currentShellId]);

    const normalizedTasks = useMemo<NormalizedTaskItem[]>(() => {
        return [...sourceTasks].reverse().map((t) => ({
            ...t,
            cleanedInput: stripInternalMacro(t.input),
            status: resolveTaskStatus(t),
        }));
    }, [sourceTasks]);

    const filteredTasks = useMemo(() => {
        return normalizedTasks.filter((t) => {
            if (statusFilter === "active" && t.status !== "queued" && t.status !== "running") {
                return false;
            }
            if (statusFilter === "completed" && t.status !== "completed") {
                return false;
            }
            if (statusFilter === "failed" && t.status !== "failed") {
                return false;
            }
            if (!searchQuery.trim()) {
                return true;
            }
            const q = searchQuery.toLowerCase();
            const inInput = t.cleanedInput.toLowerCase().includes(q);
            const inOutput = (t.output || "").toLowerCase().includes(q);
            const inError = (t.error || "").toLowerCase().includes(q);
            const inCreator = (t.creator?.name || "").toLowerCase().includes(q);
            return inInput || inOutput || inError || inCreator;
        });
    }, [normalizedTasks, statusFilter, searchQuery]);

    const aggregatedGroups = useMemo<AggregatedHistoryGroup[]>(() => {
        const groups = new Map<string, AggregatedHistoryGroup>();
        for (const task of filteredTasks) {
            const normInput = task.cleanedInput.trim().replace(/\s+/g, " ");
            const normOutput = (task.output || "").trim().replace(/\s+/g, " ");
            const normError = (task.error || "").trim().replace(/\s+/g, " ");
            const sig = `${task.status}::${normInput}::${normOutput}::${normError}`;

            const existing = groups.get(sig);
            if (existing) {
                existing.occurrences.push(task);
                existing.count += 1;
                existing.credCount += task.reportedCredentials?.length || 0;
                existing.fileCount += task.reportedFiles?.length || 0;
                existing.processCount += task.reportedProcesses?.length || 0;
                continue;
            }

            groups.set(sig, {
                key: task.id,
                primaryTask: task,
                occurrences: [task],
                count: 1,
                credCount: task.reportedCredentials?.length || 0,
                fileCount: task.reportedFiles?.length || 0,
                processCount: task.reportedProcesses?.length || 0,
            });
        }
        return Array.from(groups.values());
    }, [filteredTasks]);

    if (!isOpen) {
        return null;
    }

    const scopeButtonClass = currentSessionOnly
        ? "bg-gray-800 text-white border border-gray-700"
        : "bg-gray-800/60 hover:bg-gray-800 text-gray-400 hover:text-gray-200 border border-gray-800";

    return (
        <aside
            data-testid="shell-history-drawer"
            className="w-full md:w-96 max-w-full shrink-0 border-l border-gray-800 bg-gray-900 text-gray-200 flex flex-col h-full overflow-hidden z-20"
        >
            {/* Drawer Header */}
            <div className="flex items-center justify-between px-3 py-2 border-b border-gray-800 bg-gray-900">
                <div className="flex items-center gap-1.5">
                    <History className="w-4 h-4 text-gray-400" />
                    <h2 className="text-xs font-semibold uppercase tracking-wider text-gray-200">
                        History
                    </h2>
                    <span className="text-xs font-mono text-gray-500">
                        ({aggregatedGroups.length})
                    </span>
                </div>

                <div className="flex items-center gap-1.5">
                    <button
                        type="button"
                        data-testid="history-session-scope-toggle"
                        aria-pressed={currentSessionOnly}
                        onClick={() => setCurrentSessionOnly((prev) => !prev)}
                        title={
                            currentSessionOnly
                                ? "Showing current session only (click for cross-session history)"
                                : "Showing cross-session history (click to limit to current session)"
                        }
                        className={`inline-flex items-center gap-1 rounded-md px-2 py-0.5 text-xs font-medium transition-colors ${scopeButtonClass}`}
                    >
                        <span>{currentSessionOnly ? "Current Session" : "All Sessions"}</span>
                    </button>

                    <button
                        type="button"
                        onClick={onClose}
                        aria-label="Close history drawer"
                        className="p-1 rounded-md text-gray-400 hover:text-white hover:bg-gray-800 transition-colors"
                    >
                        <XMarkIcon className="w-4 h-4" />
                    </button>
                </div>
            </div>

            {/* Search & Status Filters */}
            <div className="px-3 pt-2 border-b border-gray-800 bg-gray-900">
                <div className="relative mb-2">
                    <MagnifyingGlassIcon className="w-4 h-4 text-gray-400 absolute left-2.5 top-1/2 -translate-y-1/2 pointer-events-none" />
                    <input
                        type="text"
                        value={searchQuery}
                        onChange={(e) => setSearchQuery(e.target.value)}
                        placeholder="Search command, output, or operator..."
                        className="w-full bg-gray-800 border border-gray-700 rounded-md pl-8 pr-2.5 py-1.5 text-xs text-gray-200 placeholder-gray-400 focus:outline-none focus:border-purple-500"
                    />
                </div>
                <div className="flex items-center gap-3 overflow-x-auto no-scrollbar">
                    {STATUS_FILTERS.map((filter) => {
                        const isActive = statusFilter === filter.key;
                        const tabClass = isActive
                            ? "border-purple-500 text-white"
                            : "border-transparent text-gray-400 hover:text-gray-200";
                        return (
                            <button
                                key={filter.key}
                                type="button"
                                onClick={() => setStatusFilter(filter.key)}
                                className={`pb-2 text-xs font-semibold border-b-2 transition-colors whitespace-nowrap ${tabClass}`}
                            >
                                {filter.label}
                            </button>
                        );
                    })}
                </div>
            </div>

            {/* Aggregated Task List */}
            <div className="flex-1 overflow-y-auto p-3 space-y-2.5">
                {aggregatedGroups.length === 0 ? (
                    <EmptyHistoryState />
                ) : (
                    <HistoryGroupList
                        groups={aggregatedGroups}
                        currentShellId={currentShellId}
                        onRunCode={onRunCode}
                        onLoadIntoComposer={onLoadIntoComposer}
                    />
                )}
            </div>
        </aside>
    );
};

export default ShellHistoryDrawer;
