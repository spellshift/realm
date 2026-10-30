import React, { useState } from "react";
import {
    CommandLineIcon,
    DocumentTextIcon,
    NoSymbolIcon,
    ArrowDownTrayIcon,
} from "@heroicons/react/24/outline";
import {
    ChevronDown,
    ChevronUp,
    CircleCheck,
    CirclePause,
    CircleUser,
    CircleX,
    Clock,
    Copy,
    FileCode2,
    FileText,
    KeyRound,
    ListVideo,
    Play,
} from "lucide-react";
import { ShellTaskItem } from "../hooks/useShellData";
import { formatFileSize } from "../utils/networkGraphUtils";

export type NormalizedTaskStatus = "queued" | "running" | "completed" | "failed";

export type NormalizedTaskItem = ShellTaskItem & {
    cleanedInput: string;
    status: NormalizedTaskStatus;
};

export interface AggregatedHistoryGroup {
    key: string;
    primaryTask: NormalizedTaskItem;
    occurrences: NormalizedTaskItem[];
    count: number;
    credCount: number;
    fileCount: number;
    processCount: number;
}

export type HistoryCardTab = "input" | "output" | "error" | "processes" | "files" | "creds";

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

function getActiveTextContent(activeTab: HistoryCardTab, task: NormalizedTaskItem): string {
    if (activeTab === "output") {
        return task.output || "";
    }
    if (activeTab === "error") {
        return task.error || "";
    }
    return task.cleanedInput;
}

function formatTaskTime(createdAt?: string): string | null {
    if (!createdAt) {
        return null;
    }
    return new Date(createdAt).toLocaleTimeString([], {
        hour: "2-digit",
        minute: "2-digit",
    });
}

const HistoryTaskStatusIcon: React.FC<{ status: NormalizedTaskStatus }> = ({ status }) => {
    const iconClass = "w-4 h-4 shrink-0";

    if (status === "failed") {
        return <CircleX className={`${iconClass} text-red-500`} aria-label="Failed" />;
    }
    if (status === "completed") {
        return <CircleCheck className={`${iconClass} text-green-500`} aria-label="Completed" />;
    }
    if (status === "running") {
        return <Clock className={`${iconClass} text-gray-400`} aria-label="Running" />;
    }
    return <CirclePause className={`${iconClass} text-gray-400`} aria-label="Queued" />;
};

const ExpandToggleIcon: React.FC<{ isExpanded: boolean }> = ({ isExpanded }) => {
    if (isExpanded) {
        return (
            <>
                <ChevronUp className="w-3 h-3" />
                <span>Show less</span>
            </>
        );
    }
    return (
        <>
            <ChevronDown className="w-3 h-3" />
            <span>Show full</span>
        </>
    );
};

const HistoryFilesPanel: React.FC<{
    files: NonNullable<ShellTaskItem["reportedFiles"]>;
}> = ({ files }) => {
    return (
        <div className="divide-y divide-gray-800/60 px-3 py-1">
            {files.map((file) => (
                <div key={file.id} className="py-1.5 flex items-center justify-between gap-2">
                    <div className="min-w-0 flex-1">
                        <div className="text-xs text-gray-200 font-mono truncate">{file.path}</div>
                        <div className="text-[11px] text-gray-500">{formatFileSize(file.size)}</div>
                    </div>
                    <a
                        href={`/cdn/hostfiles/${file.id}`}
                        download
                        aria-label={`Download ${file.path}`}
                        className="text-gray-400 hover:text-gray-200 p-1 rounded hover:bg-gray-800 shrink-0 transition-colors"
                    >
                        <ArrowDownTrayIcon className="w-3.5 h-3.5" />
                    </a>
                </div>
            ))}
        </div>
    );
};

const HistoryCredsPanel: React.FC<{
    credentials: NonNullable<ShellTaskItem["reportedCredentials"]>;
}> = ({ credentials }) => {
    return (
        <div className="divide-y divide-gray-800/60 px-3 py-1">
            {credentials.map((cred) => (
                <div key={cred.id} className="py-1.5 flex items-center justify-between gap-2 text-xs">
                    <span className="font-mono text-gray-200 truncate">{cred.principal}</span>
                    <span className="text-[11px] text-gray-500 font-mono shrink-0">
                        {cred.kind.replace(/^KIND_/, "")}
                    </span>
                </div>
            ))}
        </div>
    );
};

const HistoryProcessesPanel: React.FC<{ count: number }> = ({ count }) => {
    return (
        <div className="px-3 py-2 text-xs text-gray-400">
            {count} {count === 1 ? "process" : "processes"} reported
        </div>
    );
};

export interface HistoryTaskCardProps {
    group: AggregatedHistoryGroup;
    currentShellId?: string;
    onRunCode: (code: string) => void;
    onLoadIntoComposer: (code: string) => void;
}

export const HistoryTaskCard: React.FC<HistoryTaskCardProps> = ({
    group,
    currentShellId,
    onRunCode,
    onLoadIntoComposer,
}) => {
    const [activeTab, setActiveTab] = useState<HistoryCardTab>("input");
    const [isFullExpanded, setIsFullExpanded] = useState(false);

    const task = group.primaryTask;
    const hasOutput = Boolean(task.output && task.output.trim());
    const hasError = Boolean(task.error && task.error.trim());

    const reportedCreds = group.occurrences.flatMap((t) => t.reportedCredentials || []);
    const reportedFiles = group.occurrences.flatMap((t) => t.reportedFiles || []);

    const isTextTab = activeTab === "input" || activeTab === "output" || activeTab === "error";
    const activeTextContent = getActiveTextContent(activeTab, task);

    const { summary, isTruncated } = summarizeText(activeTextContent);
    const displayedText = isFullExpanded ? activeTextContent : summary;

    const isOtherSession = Boolean(
        currentShellId && task.shellId && String(task.shellId) !== String(currentShellId)
    );

    const formattedTime = formatTaskTime(task.createdAt);

    const getTabClassName = (tab: HistoryCardTab) => {
        const base = "flex flex-row gap-1 items-center py-1.5 px-2.5 text-xs transition-colors";
        if (activeTab === tab) {
            return `${base} bg-gray-950 text-gray-100 rounded-t border-t-2 border-gray-700`;
        }
        return `${base} text-gray-400 hover:text-gray-200`;
    };

    return (
        <div
            data-testid="history-task-card"
            className="rounded-lg border border-gray-800 bg-gray-900/30 w-full text-gray-200 overflow-hidden"
        >
            {/* Header row modeled after TaskCard header */}
            <div className="flex flex-row items-center justify-between gap-2 px-3 py-2 bg-gray-900/50">
                <div className="flex flex-row items-center gap-2 min-w-0 flex-wrap">
                    <HistoryTaskStatusIcon status={task.status} />

                    {task.creator?.name && (
                        <div className="flex flex-row gap-1 items-center text-xs text-gray-300">
                            <CircleUser className="w-3.5 h-3.5 text-gray-500" />
                            <span>{task.creator.name}</span>
                        </div>
                    )}

                    {formattedTime && (
                        <span className="text-[11px] text-gray-500">{formattedTime}</span>
                    )}

                    {group.count > 1 && (
                        <span
                            data-testid="history-aggregated-count"
                            title={`Aggregated ${group.count} executions with matching output`}
                            className="px-1.5 py-0.5 rounded bg-gray-800 text-gray-300 font-mono text-[10px]"
                        >
                            ×{group.count}
                        </span>
                    )}

                    {isOtherSession && (
                        <span className="px-1.5 py-0.5 rounded bg-gray-800/80 text-gray-400 font-mono text-[10px]">
                            Shell #{task.shellId}
                        </span>
                    )}
                </div>

                {/* Minimalist actions */}
                <div className="flex items-center gap-1 shrink-0">
                    <button
                        type="button"
                        onClick={() => onRunCode(task.cleanedInput)}
                        className="inline-flex items-center gap-1 px-1.5 py-0.5 rounded text-[11px] text-gray-300 hover:text-white hover:bg-gray-800 transition-colors"
                        title="Re-run command"
                    >
                        <Play className="w-3 h-3" />
                        <span>Re-run</span>
                    </button>
                    <button
                        type="button"
                        onClick={() => onLoadIntoComposer(task.cleanedInput)}
                        className="inline-flex items-center gap-1 px-1.5 py-0.5 rounded text-[11px] text-gray-400 hover:text-gray-200 hover:bg-gray-800 transition-colors"
                        title="Open in Composer"
                    >
                        <FileCode2 className="w-3 h-3" />
                        <span>Composer</span>
                    </button>
                    <button
                        type="button"
                        onClick={() => navigator.clipboard?.writeText(activeTextContent)}
                        className="inline-flex items-center gap-1 p-1 rounded text-[11px] text-gray-400 hover:text-gray-200 hover:bg-gray-800 transition-colors"
                        title="Copy"
                        aria-label="Copy"
                    >
                        <Copy className="w-3 h-3" />
                    </button>
                </div>
            </div>

            {/* TabList modeled after TaskCard tabs */}
            <div className="border-t border-gray-800 flex flex-col">
                <div className="flex flex-row items-center justify-between gap-2 bg-gray-900 px-2 pt-1 border-b border-gray-800">
                    <div
                        role="tablist"
                        aria-label="Task history tabs"
                        className="flex flex-row gap-1 text-gray-400 text-xs overflow-x-auto no-scrollbar"
                    >
                        <button
                            type="button"
                            role="tab"
                            aria-selected={activeTab === "input"}
                            onClick={() => setActiveTab("input")}
                            className={getTabClassName("input")}
                        >
                            <CommandLineIcon className="w-3.5 h-3.5" />
                            <span>Input</span>
                        </button>

                        {hasOutput && (
                            <button
                                type="button"
                                role="tab"
                                aria-selected={activeTab === "output"}
                                onClick={() => setActiveTab("output")}
                                className={getTabClassName("output")}
                            >
                                <DocumentTextIcon className="w-3.5 h-3.5" />
                                <span>Output</span>
                            </button>
                        )}

                        {hasError && (
                            <button
                                type="button"
                                role="tab"
                                aria-selected={activeTab === "error"}
                                onClick={() => setActiveTab("error")}
                                className={getTabClassName("error")}
                            >
                                <NoSymbolIcon className="w-3.5 h-3.5" />
                                <span>Error</span>
                            </button>
                        )}

                        {group.processCount > 0 && (
                            <button
                                type="button"
                                role="tab"
                                aria-selected={activeTab === "processes"}
                                onClick={() => setActiveTab("processes")}
                                className={getTabClassName("processes")}
                            >
                                <ListVideo className="w-3.5 h-3.5" />
                                <span>Processes ({group.processCount})</span>
                            </button>
                        )}

                        {group.fileCount > 0 && (
                            <button
                                type="button"
                                role="tab"
                                aria-selected={activeTab === "files"}
                                onClick={() => setActiveTab("files")}
                                className={getTabClassName("files")}
                            >
                                <FileText className="w-3.5 h-3.5" />
                                <span>Files ({group.fileCount})</span>
                            </button>
                        )}

                        {group.credCount > 0 && (
                            <button
                                type="button"
                                role="tab"
                                aria-selected={activeTab === "creds"}
                                onClick={() => setActiveTab("creds")}
                                className={getTabClassName("creds")}
                            >
                                <KeyRound className="w-3.5 h-3.5" />
                                <span>Creds ({group.credCount})</span>
                            </button>
                        )}
                    </div>

                    {isTextTab && isTruncated && (
                        <button
                            type="button"
                            data-testid="history-toggle-full"
                            onClick={() => setIsFullExpanded((prev) => !prev)}
                            className="inline-flex items-center gap-0.5 px-1.5 py-0.5 text-[11px] text-gray-400 hover:text-gray-200 shrink-0 transition-colors"
                        >
                            <ExpandToggleIcon isExpanded={isFullExpanded} />
                        </button>
                    )}
                </div>

                {/* TabPanels */}
                <div className="bg-gray-950">
                    {isTextTab && (
                        <pre
                            className={`text-xs font-mono p-2.5 text-gray-200 overflow-x-auto whitespace-pre-wrap break-words ${
                                isFullExpanded ? "max-h-72 overflow-y-auto" : "max-h-24"
                            }`}
                        >
                            {displayedText}
                        </pre>
                    )}
                    {activeTab === "processes" && (
                        <HistoryProcessesPanel count={group.processCount} />
                    )}
                    {activeTab === "files" && <HistoryFilesPanel files={reportedFiles} />}
                    {activeTab === "creds" && <HistoryCredsPanel credentials={reportedCreds} />}
                </div>
            </div>
        </div>
    );
};

export default HistoryTaskCard;
