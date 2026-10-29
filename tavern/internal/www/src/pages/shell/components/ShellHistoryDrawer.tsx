import React, { useState, useMemo, useEffect } from "react";
import {
    CheckCircle2,
    ChevronDown,
    ChevronUp,
    Clock,
    Copy,
    FileCode2,
    History,
    KeyRound,
    Layers,
    Loader2,
    Play,
    Search,
    X,
    XCircle,
} from "lucide-react";
import { ShellPivotItem, ShellTaskItem } from "../hooks/useShellData";
import { stripInternalMacro } from "../hooks/shellUtils";

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

const SUMMARY_MAX_LINES = 2;
const SUMMARY_MAX_CHARS = 120;

export function summarizeText(text: string): {
    summary: string;
    isTruncated: boolean;
    totalLines: number;
} {
    const normalized = text.replace(/\r\n/g, "\n").trim();
    const lines = normalized ? normalized.split("\n") : [""];
    const totalLines = lines.length;

    if (totalLines > SUMMARY_MAX_LINES) {
        const sliced = lines.slice(0, SUMMARY_MAX_LINES).join("\n");
        const summary =
            sliced.length > SUMMARY_MAX_CHARS
                ? `${sliced.slice(0, SUMMARY_MAX_CHARS)}...`
                : `${sliced}...`;
        return { summary, isTruncated: true, totalLines };
    }

    if (normalized.length > SUMMARY_MAX_CHARS) {
        return {
            summary: `${normalized.slice(0, SUMMARY_MAX_CHARS)}...`,
            isTruncated: true,
            totalLines,
        };
    }

    return { summary: normalized, isTruncated: false, totalLines };
}

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
    const [activeTabByGroup, setActiveTabByGroup] = useState<
        Record<string, "input" | "output" | "error">
    >({});
    const [fullExpandedByGroup, setFullExpandedByGroup] = useState<Record<string, boolean>>({});

    useEffect(() => {
        if (focusQueueTrigger > 0) {
            setStatusFilter("active");
        }
    }, [focusQueueTrigger]);

    type NormalizedTaskItem = ShellTaskItem & {
        cleanedInput: string;
        status: "queued" | "running" | "completed" | "failed";
    };

    type AggregatedHistoryGroup = {
        key: string;
        primaryTask: NormalizedTaskItem;
        occurrences: NormalizedTaskItem[];
        count: number;
        credCount: number;
        fileCount: number;
    };

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
        return [...sourceTasks].reverse().map((t) => {
            const cleanedInput = stripInternalMacro(t.input);
            const isFailed = Boolean(t.error && t.error.trim().length > 0);
            const isFinished = Boolean(t.execFinishedAt) || isFailed || Boolean(t.output);
            const isRunning = Boolean(t.claimedAt || t.execStartedAt) && !isFinished;
            const status: "queued" | "running" | "completed" | "failed" = isFailed
                ? "failed"
                : isFinished
                  ? "completed"
                  : isRunning
                    ? "running"
                    : "queued";

            return {
                ...t,
                cleanedInput,
                status,
            };
        });
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
            if (searchQuery.trim()) {
                const q = searchQuery.toLowerCase();
                const inInput = t.cleanedInput.toLowerCase().includes(q);
                const inOutput = (t.output || "").toLowerCase().includes(q);
                const inError = (t.error || "").toLowerCase().includes(q);
                const inCreator = (t.creator?.name || "").toLowerCase().includes(q);
                return inInput || inOutput || inError || inCreator;
            }
            return true;
        });
    }, [normalizedTasks, statusFilter, searchQuery]);

    // Aggregate tasks with identical/similar input + output/error/status to reduce clutter
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
            } else {
                groups.set(sig, {
                    key: task.id,
                    primaryTask: task,
                    occurrences: [task],
                    count: 1,
                    credCount: task.reportedCredentials?.length || 0,
                    fileCount: task.reportedFiles?.length || 0,
                });
            }
        }
        return Array.from(groups.values());
    }, [filteredTasks]);

    if (!isOpen) return null;

    return (
        <div
            data-testid="shell-history-drawer"
            className="w-full md:w-[420px] shrink-0 border-l border-[#262b38] bg-[#141720] text-gray-200 flex flex-col h-full overflow-hidden shadow-2xl z-20"
        >
            {/* Drawer Header */}
            <div className="flex items-center justify-between px-4 py-2.5 border-b border-[#262b38] bg-[#181c27]">
                <div className="flex items-center gap-2">
                    <History className="w-4 h-4 text-purple-400" />
                    <h2 className="text-sm font-semibold text-white">History</h2>
                    <span className="text-[11px] font-mono px-1.5 py-0.5 rounded bg-white/5 text-gray-400">
                        {aggregatedGroups.length}
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
                        className={`inline-flex items-center gap-1 rounded-md px-2.5 py-1 text-xs font-semibold transition-colors ${
                            currentSessionOnly
                                ? "bg-purple-600 text-white shadow-sm"
                                : "bg-[#1e222d] hover:bg-[#272d3d] text-gray-300 border border-[#2f364a]"
                        }`}
                    >
                        <span>{currentSessionOnly ? "Current Session" : "All Sessions"}</span>
                    </button>

                    <button
                        type="button"
                        onClick={onClose}
                        aria-label="Close history drawer"
                        className="p-1.5 rounded-md hover:bg-white/10 text-gray-400 hover:text-white transition-colors"
                    >
                        <X className="w-4 h-4" />
                    </button>
                </div>
            </div>

            {/* Search & Status Filters */}
            <div className="p-3 border-b border-[#262b38] space-y-2 bg-[#161922]">
                <div className="relative">
                    <Search className="w-3.5 h-3.5 text-gray-400 absolute left-2.5 top-1/2 -translate-y-1/2" />
                    <input
                        type="text"
                        value={searchQuery}
                        onChange={(e) => setSearchQuery(e.target.value)}
                        placeholder="Search command, output, or operator..."
                        className="w-full pl-8 pr-3 py-1.5 rounded-md bg-[#0f1117] border border-[#2a3040] text-xs text-gray-200 placeholder-gray-500 focus:outline-none focus:border-purple-500"
                    />
                </div>
                <div className="flex items-center gap-1.5 flex-wrap text-xs">
                    {(["all", "active", "completed", "failed"] as const).map((f) => (
                        <button
                            key={f}
                            type="button"
                            onClick={() => setStatusFilter(f)}
                            className={`px-2.5 py-1 rounded-md capitalize font-semibold transition-colors ${
                                statusFilter === f
                                    ? "bg-purple-600 text-white shadow-sm"
                                    : "bg-[#1e222d] text-gray-400 hover:text-gray-200 border border-[#2f364a]"
                            }`}
                        >
                            {f === "active" ? "Queued / Running" : f}
                        </button>
                    ))}
                </div>
            </div>

            {/* Aggregated Task List */}
            <div className="flex-1 overflow-y-auto p-3 space-y-2.5">
                {aggregatedGroups.length === 0 ? (
                    <div className="text-center py-8 text-xs text-gray-500">
                        No matching commands found.
                    </div>
                ) : (
                    aggregatedGroups.map((group) => {
                        const task = group.primaryTask;
                        const hasOutput = Boolean(task.output && task.output.trim());
                        const hasError = Boolean(task.error && task.error.trim());
                        const activeTab = activeTabByGroup[group.key] || "input";
                        const isFullExpanded = Boolean(fullExpandedByGroup[group.key]);

                        const activeContent =
                            activeTab === "output"
                                ? task.output || ""
                                : activeTab === "error"
                                  ? task.error || ""
                                  : task.cleanedInput;

                        const { summary, isTruncated, totalLines } = summarizeText(activeContent);
                        const displayedText = isFullExpanded ? activeContent : summary;

                        const isOtherSession =
                            currentShellId &&
                            task.shellId &&
                            String(task.shellId) !== String(currentShellId);

                        return (
                            <div
                                key={group.key}
                                data-testid="history-task-card"
                                className="rounded-md border border-[#262b38] bg-[#181c26] hover:border-[#343c50] transition-colors p-2.5 space-y-2"
                            >
                                {/* Top Meta Row: Status + Aggregation Count + Artifacts + Timestamp */}
                                <div className="flex items-center justify-between gap-2 text-[11px]">
                                    <div className="flex items-center gap-1.5 flex-wrap min-w-0">
                                        {task.status === "queued" && (
                                            <span className="inline-flex items-center gap-1 px-1.5 py-0.5 rounded bg-amber-500/15 text-amber-300 border border-amber-500/30 font-medium">
                                                <Clock className="w-3 h-3" />
                                                Queued
                                            </span>
                                        )}
                                        {task.status === "running" && (
                                            <span className="inline-flex items-center gap-1 px-1.5 py-0.5 rounded bg-blue-500/15 text-blue-300 border border-blue-500/30 font-medium">
                                                <Loader2 className="w-3 h-3 animate-spin" />
                                                Running
                                            </span>
                                        )}
                                        {task.status === "completed" && (
                                            <span className="inline-flex items-center gap-1 px-1.5 py-0.5 rounded bg-emerald-500/15 text-emerald-300 border border-emerald-500/30 font-medium">
                                                <CheckCircle2 className="w-3 h-3" />
                                                Done
                                            </span>
                                        )}
                                        {task.status === "failed" && (
                                            <span className="inline-flex items-center gap-1 px-1.5 py-0.5 rounded bg-red-500/15 text-red-300 border border-red-500/30 font-medium">
                                                <XCircle className="w-3 h-3" />
                                                Error
                                            </span>
                                        )}

                                        {group.count > 1 && (
                                            <span
                                                data-testid="history-aggregated-count"
                                                title={`Aggregated ${group.count} executions with matching output`}
                                                className="inline-flex items-center gap-1 px-1.5 py-0.5 rounded bg-purple-500/20 text-purple-200 border border-purple-500/40 font-mono font-semibold"
                                            >
                                                <Layers className="w-3 h-3" />
                                                ×{group.count}
                                            </span>
                                        )}

                                        {isOtherSession && (
                                            <span className="px-1.5 py-0.5 rounded bg-white/5 text-gray-400 font-mono text-[10px]">
                                                Shell #{task.shellId}
                                            </span>
                                        )}

                                        {group.credCount > 0 && (
                                            <span className="inline-flex items-center gap-1 px-1.5 py-0.5 rounded bg-purple-500/20 text-purple-200 border border-purple-500/40 font-medium">
                                                <KeyRound className="w-3 h-3" />
                                                {group.credCount}{" "}
                                                {group.credCount === 1 ? "Cred" : "Creds"} Reported
                                            </span>
                                        )}
                                        {group.fileCount > 0 && (
                                            <span className="inline-flex items-center gap-1 px-1.5 py-0.5 rounded bg-sky-500/20 text-sky-200 border border-sky-500/40 font-medium">
                                                <FileCode2 className="w-3 h-3" />
                                                {group.fileCount}{" "}
                                                {group.fileCount === 1 ? "File" : "Files"} Reported
                                            </span>
                                        )}
                                    </div>

                                    <div className="flex items-center gap-1.5 text-gray-500 shrink-0">
                                        {task.creator?.name && <span>@{task.creator.name}</span>}
                                        {task.createdAt && (
                                            <span>
                                                {new Date(task.createdAt).toLocaleTimeString([], {
                                                    hour: "2-digit",
                                                    minute: "2-digit",
                                                })}
                                            </span>
                                        )}
                                    </div>
                                </div>

                                {/* Task Card Style Tabs (Input / Output / Error) + Show Full Toggle */}
                                <div className="flex items-center justify-between gap-2 border-b border-[#262b38] pb-1">
                                    <div
                                        role="tablist"
                                        aria-label="Task history tabs"
                                        className="flex items-center gap-1"
                                    >
                                        <button
                                            type="button"
                                            role="tab"
                                            aria-selected={activeTab === "input"}
                                            onClick={() =>
                                                setActiveTabByGroup((prev) => ({
                                                    ...prev,
                                                    [group.key]: "input",
                                                }))
                                            }
                                            className={`px-2 py-0.5 rounded text-[11px] font-semibold transition-colors ${
                                                activeTab === "input"
                                                    ? "bg-purple-600/25 text-purple-200 border border-purple-500/40"
                                                    : "text-gray-400 hover:text-gray-200"
                                            }`}
                                        >
                                            Input
                                        </button>

                                        {hasOutput && (
                                            <button
                                                type="button"
                                                role="tab"
                                                aria-selected={activeTab === "output"}
                                                onClick={() =>
                                                    setActiveTabByGroup((prev) => ({
                                                        ...prev,
                                                        [group.key]:
                                                            prev[group.key] === "output"
                                                                ? "input"
                                                                : "output",
                                                    }))
                                                }
                                                className={`px-2 py-0.5 rounded text-[11px] font-semibold transition-colors ${
                                                    activeTab === "output"
                                                        ? "bg-purple-600/25 text-purple-200 border border-purple-500/40"
                                                        : "text-gray-400 hover:text-gray-200"
                                                }`}
                                            >
                                                {activeTab === "output" ? "Output" : "View Output"}
                                            </button>
                                        )}

                                        {hasError && (
                                            <button
                                                type="button"
                                                role="tab"
                                                aria-selected={activeTab === "error"}
                                                onClick={() =>
                                                    setActiveTabByGroup((prev) => ({
                                                        ...prev,
                                                        [group.key]:
                                                            prev[group.key] === "error"
                                                                ? "input"
                                                                : "error",
                                                    }))
                                                }
                                                className={`px-2 py-0.5 rounded text-[11px] font-semibold transition-colors ${
                                                    activeTab === "error"
                                                        ? "bg-red-500/25 text-red-200 border border-red-500/40"
                                                        : "text-red-400/80 hover:text-red-300"
                                                }`}
                                            >
                                                Error
                                            </button>
                                        )}
                                    </div>

                                    {isTruncated && (
                                        <button
                                            type="button"
                                            data-testid="history-toggle-full"
                                            onClick={() =>
                                                setFullExpandedByGroup((prev) => ({
                                                    ...prev,
                                                    [group.key]: !isFullExpanded,
                                                }))
                                            }
                                            className="inline-flex items-center gap-0.5 px-1.5 py-0.5 rounded text-[11px] font-medium text-purple-300 hover:text-purple-200 hover:bg-white/5 transition-colors"
                                        >
                                            {isFullExpanded ? (
                                                <>
                                                    <ChevronUp className="w-3 h-3" />
                                                    <span>Show less</span>
                                                </>
                                            ) : (
                                                <>
                                                    <ChevronDown className="w-3 h-3" />
                                                    <span>Show full ({totalLines} lines)</span>
                                                </>
                                            )}
                                        </button>
                                    )}
                                </div>

                                {/* Tab Content Panel (Truncated/Summarized by default, expandable to full) */}
                                <pre
                                    className={`text-xs font-mono rounded-md p-2 overflow-x-auto whitespace-pre-wrap break-words ${
                                        activeTab === "error"
                                            ? "bg-red-950/30 text-red-300 border border-red-500/20"
                                            : "bg-[#0f1117] text-gray-100"
                                    } ${isFullExpanded ? "max-h-72 overflow-y-auto" : "max-h-24"}`}
                                >
                                    {displayedText}
                                </pre>

                                {/* Action Buttons with consistent Tavern button padding & borders */}
                                <div className="flex items-center justify-between gap-1.5 pt-1">
                                    <div className="flex items-center gap-1.5">
                                        <button
                                            type="button"
                                            onClick={() => onRunCode(task.cleanedInput)}
                                            className="inline-flex items-center gap-1 px-2.5 py-1 rounded-md bg-purple-600 hover:bg-purple-500 text-white text-xs font-semibold shadow-sm transition-colors"
                                            title="Re-run command immediately"
                                        >
                                            <Play className="w-3 h-3 fill-current" />
                                            Re-run
                                        </button>
                                        <button
                                            type="button"
                                            onClick={() => onLoadIntoComposer(task.cleanedInput)}
                                            className="inline-flex items-center gap-1 px-2.5 py-1 rounded-md bg-[#1e222d] hover:bg-[#272d3d] border border-[#2f364a] text-gray-200 text-xs font-semibold shadow-sm transition-colors"
                                            title="Open and edit in multi-line Composer"
                                        >
                                            <FileCode2 className="w-3 h-3" />
                                            Composer
                                        </button>
                                    </div>

                                    <button
                                        type="button"
                                        onClick={() =>
                                            navigator.clipboard?.writeText(activeContent)
                                        }
                                        className="inline-flex items-center gap-1 px-2 py-1 rounded-md bg-[#1e222d] hover:bg-[#272d3d] border border-[#2f364a] text-gray-300 hover:text-white text-xs font-semibold shadow-sm transition-colors"
                                        title={`Copy ${activeTab}`}
                                    >
                                        <Copy className="w-3 h-3" />
                                        Copy
                                    </button>
                                </div>
                            </div>
                        );
                    })
                )}
            </div>
        </div>
    );
};

export default ShellHistoryDrawer;
