import React, { useState, useMemo, useEffect } from "react";
import {
    CheckCircle2,
    Clock,
    Copy,
    Eye,
    FileCode2,
    History,
    KeyRound,
    Loader2,
    Play,
    Search,
    X,
    XCircle,
} from "lucide-react";
import { ShellPivotItem, ShellTaskItem } from "../hooks/useShellData";

export type HistoryStatusFilter = "all" | "active" | "completed" | "failed";

interface ShellHistoryDrawerProps {
    isOpen: boolean;
    onClose: () => void;
    shellTasks: ShellTaskItem[];
    localHistory: string[];
    pivots?: ShellPivotItem[];
    initialStatusFilter?: HistoryStatusFilter;
    focusQueueTrigger?: number;
    onRunCode: (code: string) => void;
    onLoadIntoComposer: (code: string) => void;
    onOpenPivotTab?: (type: "pty" | "ssh", target: string, pivotId?: string) => void;
}

const stripInternalMacro = (input: string): string => {
    const prefix = "for _nonomacroclowntown in range(1):\n\t_nonomacroclowntown = ";
    const suffix = "print(_nonomacroclowntown['stdout']);print(_nonomacroclowntown['stderr'])\n";
    let cleaned = input;
    if (cleaned.startsWith(prefix)) {
        cleaned = cleaned.slice(prefix.length);
    }
    if (cleaned.endsWith(suffix)) {
        cleaned = cleaned.slice(0, -suffix.length);
    }
    return cleaned.trim();
};

const ShellHistoryDrawer: React.FC<ShellHistoryDrawerProps> = ({
    isOpen,
    onClose,
    shellTasks,
    localHistory,
    initialStatusFilter = "all",
    focusQueueTrigger = 0,
    onRunCode,
    onLoadIntoComposer,
}) => {
    const [searchQuery, setSearchQuery] = useState("");
    const [statusFilter, setStatusFilter] = useState<HistoryStatusFilter>(initialStatusFilter);
    const [expandedTaskId, setExpandedTaskId] = useState<string | null>(null);

    useEffect(() => {
        if (focusQueueTrigger > 0) {
            setStatusFilter("active");
        }
    }, [focusQueueTrigger]);

    type NormalizedTaskItem = ShellTaskItem & {
        cleanedInput: string;
        status: "queued" | "running" | "completed" | "failed";
    };

    const normalizedTasks = useMemo<NormalizedTaskItem[]>(() => {
        if (shellTasks.length > 0) {
            return shellTasks.map((t) => {
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
        }

        // Fallback to localStorage history when server task history isn't loaded
        return [...localHistory].reverse().map((cmd, idx) => ({
            id: `local-${idx}`,
            createdAt: "",
            input: cmd,
            cleanedInput: cmd,
            status: "completed" as const,
            output: "",
            error: "",
            creator: undefined,
            reportedCredentials: [],
            reportedFiles: [],
            reportedProcesses: [],
        }));
    }, [shellTasks, localHistory]);

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

    if (!isOpen) return null;

    return (
        <div
            data-testid="shell-history-drawer"
            className="w-full md:w-[420px] shrink-0 border-l border-[#262b38] bg-[#141720] text-gray-200 flex flex-col h-full overflow-hidden shadow-2xl z-20"
        >
            {/* Drawer Header */}
            <div className="flex items-center justify-between px-4 py-3 border-b border-[#262b38] bg-[#181c27]">
                <div className="flex items-center gap-2">
                    <History className="w-4 h-4 text-purple-400" />
                    <h2 className="text-sm font-semibold text-white">History</h2>
                    <span className="text-[11px] font-mono px-1.5 py-0.5 rounded bg-white/5 text-gray-400">
                        {normalizedTasks.length}
                    </span>
                </div>
                <button
                    type="button"
                    onClick={onClose}
                    aria-label="Close history drawer"
                    className="p-1 rounded hover:bg-white/10 text-gray-400 hover:text-white transition-colors"
                >
                    <X className="w-4 h-4" />
                </button>
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
                        className="w-full pl-8 pr-3 py-1.5 rounded bg-[#0f1117] border border-[#2a3040] text-xs text-gray-200 placeholder-gray-500 focus:outline-none focus:border-purple-500"
                    />
                </div>
                <div className="flex items-center gap-1.5 flex-wrap text-[11px]">
                    {(["all", "active", "completed", "failed"] as const).map((f) => (
                        <button
                            key={f}
                            type="button"
                            onClick={() => setStatusFilter(f)}
                            className={`px-2 py-0.5 rounded capitalize font-medium transition-colors ${
                                statusFilter === f
                                    ? "bg-purple-600 text-white"
                                    : "bg-[#1e222d] text-gray-400 hover:text-gray-200"
                            }`}
                        >
                            {f === "active" ? "Queued / Running" : f}
                        </button>
                    ))}
                </div>
            </div>

            {/* Task List */}
            <div className="flex-1 overflow-y-auto p-3 space-y-2">
                {filteredTasks.length === 0 ? (
                    <div className="text-center py-8 text-xs text-gray-500">
                        No matching commands found.
                    </div>
                ) : (
                    filteredTasks.map((task) => {
                        const isExpanded = expandedTaskId === task.id;
                        const hasOutput = Boolean(
                            (task.output && task.output.trim()) ||
                                (task.error && task.error.trim())
                        );
                        const credCount = task.reportedCredentials?.length || 0;
                        const fileCount = task.reportedFiles?.length || 0;

                        return (
                            <div
                                key={task.id}
                                className="rounded border border-[#262b38] bg-[#181c26] hover:border-[#343c50] transition-colors p-2.5 space-y-2"
                            >
                                {/* Status + Creator + Timestamp */}
                                <div className="flex items-center justify-between gap-2 text-[11px]">
                                    <div className="flex items-center gap-1.5 flex-wrap">
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
                                                Completed
                                            </span>
                                        )}
                                        {task.status === "failed" && (
                                            <span className="inline-flex items-center gap-1 px-1.5 py-0.5 rounded bg-red-500/15 text-red-300 border border-red-500/30 font-medium">
                                                <XCircle className="w-3 h-3" />
                                                Error
                                            </span>
                                        )}

                                        {credCount > 0 && (
                                            <span className="inline-flex items-center gap-1 px-1.5 py-0.5 rounded bg-purple-500/20 text-purple-200 border border-purple-500/40 font-medium">
                                                <KeyRound className="w-3 h-3" />
                                                {credCount} {credCount === 1 ? "Cred" : "Creds"} Reported
                                            </span>
                                        )}
                                        {fileCount > 0 && (
                                            <span className="inline-flex items-center gap-1 px-1.5 py-0.5 rounded bg-sky-500/20 text-sky-200 border border-sky-500/40 font-medium">
                                                <FileCode2 className="w-3 h-3" />
                                                {fileCount} {fileCount === 1 ? "File" : "Files"} Reported
                                            </span>
                                        )}
                                    </div>

                                    <div className="flex items-center gap-2 text-gray-500 shrink-0">
                                        {task.creator?.name && <span>@{task.creator.name}</span>}
                                        {task.createdAt && (
                                            <span>
                                                {new Date(task.createdAt).toLocaleTimeString([], {
                                                    hour: "2-digit",
                                                    minute: "2-digit",
                                                    second: "2-digit",
                                                })}
                                            </span>
                                        )}
                                    </div>
                                </div>

                                {/* Command Code Snippet */}
                                <pre className="text-xs font-mono text-gray-100 bg-[#0f1117] rounded p-2 overflow-x-auto whitespace-pre-wrap break-words max-h-36">
                                    {task.cleanedInput}
                                </pre>

                                {/* Expandable Output / Error Preview */}
                                {isExpanded && hasOutput && (
                                    <div className="rounded bg-black/50 border border-[#2a3040] p-2 space-y-1">
                                        <div className="text-[10px] uppercase tracking-wider text-gray-400 font-semibold">
                                            Task Output
                                        </div>
                                        {task.output && (
                                            <pre className="text-[11px] font-mono text-gray-300 whitespace-pre-wrap break-words max-h-48 overflow-y-auto">
                                                {task.output}
                                            </pre>
                                        )}
                                        {task.error && (
                                            <pre className="text-[11px] font-mono text-red-400 whitespace-pre-wrap break-words max-h-48 overflow-y-auto">
                                                {task.error}
                                            </pre>
                                        )}
                                    </div>
                                )}

                                {/* Action Buttons */}
                                <div className="flex items-center justify-between gap-1 pt-1 border-t border-white/5 text-[11px]">
                                    <div className="flex items-center gap-1">
                                        <button
                                            type="button"
                                            onClick={() => onRunCode(task.cleanedInput)}
                                            className="inline-flex items-center gap-1 px-2 py-1 rounded bg-purple-600/20 hover:bg-purple-600/35 text-purple-200 border border-purple-500/30 font-medium transition-colors"
                                            title="Re-run command immediately"
                                        >
                                            <Play className="w-3 h-3" />
                                            Re-run
                                        </button>
                                        <button
                                            type="button"
                                            onClick={() => onLoadIntoComposer(task.cleanedInput)}
                                            className="inline-flex items-center gap-1 px-2 py-1 rounded bg-[#222736] hover:bg-[#2c3347] text-gray-300 transition-colors"
                                            title="Open and edit in multi-line Composer"
                                        >
                                            <FileCode2 className="w-3 h-3" />
                                            Composer
                                        </button>
                                        <button
                                            type="button"
                                            onClick={() =>
                                                navigator.clipboard?.writeText(task.cleanedInput)
                                            }
                                            className="inline-flex items-center gap-1 px-2 py-1 rounded bg-[#222736] hover:bg-[#2c3347] text-gray-400 hover:text-gray-200 transition-colors"
                                            title="Copy command"
                                        >
                                            <Copy className="w-3 h-3" />
                                        </button>
                                    </div>

                                    {hasOutput && (
                                        <button
                                            type="button"
                                            onClick={() =>
                                                setExpandedTaskId(isExpanded ? null : task.id)
                                            }
                                            className="inline-flex items-center gap-1 px-2 py-1 rounded text-gray-400 hover:text-gray-200 hover:bg-white/5 transition-colors"
                                        >
                                            <Eye className="w-3 h-3" />
                                            {isExpanded ? "Hide Output" : "View Output"}
                                        </button>
                                    )}
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
