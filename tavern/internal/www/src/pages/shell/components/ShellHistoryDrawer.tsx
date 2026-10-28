import React, { useMemo, useState } from "react";
import {
  CheckCircle2,
  ChevronDown,
  ChevronRight,
  Clock,
  Code2,
  Copy,
  Cpu,
  FileText,
  History,
  KeyRound,
  Loader2,
  Play,
  Search,
  SquareTerminal,
  Terminal,
  X,
  XCircle,
} from "lucide-react";
import { ShellPivotItem, ShellTaskItem } from "../hooks/useShellData";

interface ShellHistoryDrawerProps {
  isOpen: boolean;
  onClose: () => void;
  shellTasks: ShellTaskItem[];
  localHistory: string[];
  pivots: ShellPivotItem[];
  onRunCode: (code: string) => void;
  onLoadIntoComposer: (code: string) => void;
  onOpenPivotTab: (type: string, target: string, pivotId?: string) => void;
}

type FilterStatus = "all" | "active" | "completed" | "error";

const formatDuration = (start?: string | null, end?: string | null): string | null => {
  if (!start || !end) return null;
  const ms = new Date(end).getTime() - new Date(start).getTime();
  if (isNaN(ms) || ms < 0) return null;
  if (ms < 1000) return `${ms}ms`;
  return `${(ms / 1000).toFixed(1)}s`;
};

const ShellHistoryDrawer: React.FC<ShellHistoryDrawerProps> = ({
  isOpen,
  onClose,
  shellTasks,
  localHistory,
  pivots,
  onRunCode,
  onLoadIntoComposer,
  onOpenPivotTab,
}) => {
  const [activeSubTab, setActiveSubTab] = useState<"tasks" | "pivots">("tasks");
  const [searchQuery, setSearchQuery] = useState("");
  const [statusFilter, setStatusFilter] = useState<FilterStatus>("all");
  const [expandedTaskIds, setExpandedTaskIds] = useState<Set<string>>(new Set());
  const [copiedId, setCopiedId] = useState<string | null>(null);

  const toggleExpand = (id: string) => {
    setExpandedTaskIds((prev) => {
      const next = new Set(prev);
      if (next.has(id)) {
        next.delete(id);
      } else {
        next.add(id);
      }
      return next;
    });
  };

  const copyText = (text: string, id: string) => {
    navigator.clipboard.writeText(text);
    setCopiedId(id);
    setTimeout(() => setCopiedId((curr) => (curr === id ? null : curr)), 1500);
  };

  const filteredTasks = useMemo(() => {
    const reversed = [...shellTasks].reverse();
    return reversed.filter((t) => {
      const isQueued = !t.claimedAt && !t.execFinishedAt;
      const isRunning = !!t.claimedAt && !t.execFinishedAt;
      const hasError = !!t.error;
      const isCompleted = !!t.execFinishedAt && !hasError;

      if (statusFilter === "active" && !isQueued && !isRunning) return false;
      if (statusFilter === "completed" && !isCompleted) return false;
      if (statusFilter === "error" && !hasError) return false;

      if (searchQuery.trim()) {
        const q = searchQuery.toLowerCase();
        const inInput = t.input?.toLowerCase().includes(q);
        const inOutput = t.output?.toLowerCase().includes(q);
        const inError = t.error?.toLowerCase().includes(q);
        const inCreator = t.creator?.name?.toLowerCase().includes(q);
        return inInput || inOutput || inError || inCreator;
      }
      return true;
    });
  }, [shellTasks, statusFilter, searchQuery]);

  const filteredLocalHistory = useMemo(() => {
    const unique = Array.from(new Set([...localHistory].reverse()));
    if (!searchQuery.trim()) return unique;
    const q = searchQuery.toLowerCase();
    return unique.filter((cmd) => cmd.toLowerCase().includes(q));
  }, [localHistory, searchQuery]);

  if (!isOpen) return null;

  return (
    <aside
      aria-label="Command History & Queue"
      data-testid="shell-history-drawer"
      className="w-full md:w-96 lg:w-[420px] flex-shrink-0 flex flex-col rounded-md bg-[#161920] border border-[#2e3440] h-full overflow-hidden shadow-xl"
    >
      {/* Header */}
      <div className="flex items-center justify-between px-3.5 py-2.5 bg-[#1b1f2a] border-b border-[#2e3440]">
        <div className="flex items-center gap-2">
          <History size={16} className="text-purple-400" />
          <h2 className="text-sm font-semibold text-white">History &amp; Task Queue</h2>
          <span className="px-1.5 py-0.5 text-[11px] rounded-full bg-purple-500/20 text-purple-300 font-mono">
            {shellTasks.length}
          </span>
        </div>
        <button
          type="button"
          onClick={onClose}
          aria-label="Close history drawer"
          className="p-1 rounded text-gray-400 hover:text-white hover:bg-[#2a2f3d] transition-colors"
        >
          <X size={16} />
        </button>
      </div>

      {/* Sub-tabs: Commands vs Pivots */}
      <div className="flex border-b border-[#2e3440] bg-[#181b24] text-xs">
        <button
          type="button"
          onClick={() => setActiveSubTab("tasks")}
          className={`flex-1 py-2 px-3 font-medium border-b-2 transition-colors ${
            activeSubTab === "tasks"
              ? "border-purple-500 text-white bg-[#1d222e]"
              : "border-transparent text-gray-400 hover:text-gray-200"
          }`}
        >
          Commands &amp; Queue ({shellTasks.length})
        </button>
        <button
          type="button"
          onClick={() => setActiveSubTab("pivots")}
          className={`flex-1 py-2 px-3 font-medium border-b-2 transition-colors ${
            activeSubTab === "pivots"
              ? "border-purple-500 text-white bg-[#1d222e]"
              : "border-transparent text-gray-400 hover:text-gray-200"
          }`}
        >
          Pivot Sessions ({pivots.length})
        </button>
      </div>

      {activeSubTab === "tasks" ? (
        <>
          {/* Search & Status Filters */}
          <div className="p-2.5 border-b border-[#2e3440] bg-[#161920] space-y-2">
            <div className="relative">
              <Search size={14} className="text-gray-500 absolute left-2.5 top-2.5" />
              <input
                type="text"
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
                placeholder="Search commands or output..."
                className="w-full pl-8 pr-3 py-1.5 rounded bg-[#101217] border border-[#2e3440] text-xs text-gray-200 placeholder-gray-500 focus:outline-none focus:border-purple-500"
              />
            </div>

            <div className="flex items-center gap-1 text-[11px]">
              {(
                [
                  { id: "all", label: "All" },
                  { id: "active", label: "Queued / Running" },
                  { id: "completed", label: "Completed" },
                  { id: "error", label: "Errors" },
                ] as { id: FilterStatus; label: string }[]
              ).map((f) => (
                <button
                  key={f.id}
                  type="button"
                  onClick={() => setStatusFilter(f.id)}
                  className={`px-2 py-1 rounded font-medium transition-colors ${
                    statusFilter === f.id
                      ? "bg-purple-600 text-white"
                      : "bg-[#1e222d] text-gray-400 hover:text-gray-200"
                  }`}
                >
                  {f.label}
                </button>
              ))}
            </div>
          </div>

          {/* Task List */}
          <div className="flex-1 overflow-y-auto p-2.5 space-y-2">
            {filteredTasks.length > 0 ? (
              filteredTasks.map((task) => {
                const isQueued = !task.claimedAt && !task.execFinishedAt;
                const isRunning = !!task.claimedAt && !task.execFinishedAt;
                const hasError = !!task.error;
                const hasOutput = !!(task.output || task.error);
                const isExpanded = expandedTaskIds.has(task.id);
                const duration = formatDuration(task.execStartedAt || task.claimedAt, task.execFinishedAt);

                return (
                  <div
                    key={task.id}
                    className="rounded-md bg-[#1b1f29] border border-[#2b3140] overflow-hidden text-xs"
                  >
                    <div className="flex items-center justify-between px-2.5 py-1.5 bg-[#202532] border-b border-[#2b3140]">
                      <div className="flex items-center gap-1.5 min-w-0">
                        {isQueued && (
                          <span className="inline-flex items-center gap-1 px-1.5 py-0.5 rounded bg-blue-500/20 text-blue-300 font-medium">
                            <Clock size={11} />
                            Queued
                          </span>
                        )}
                        {isRunning && (
                          <span className="inline-flex items-center gap-1 px-1.5 py-0.5 rounded bg-amber-500/20 text-amber-300 font-medium">
                            <Loader2 size={11} className="animate-spin" />
                            Running
                          </span>
                        )}
                        {!isQueued && !isRunning && !hasError && (
                          <span className="inline-flex items-center gap-1 px-1.5 py-0.5 rounded bg-emerald-500/15 text-emerald-400 font-medium">
                            <CheckCircle2 size={11} />
                            Done
                          </span>
                        )}
                        {hasError && (
                          <span className="inline-flex items-center gap-1 px-1.5 py-0.5 rounded bg-red-500/20 text-red-400 font-medium">
                            <XCircle size={11} />
                            Error
                          </span>
                        )}

                        {duration && <span className="text-gray-500 font-mono">{duration}</span>}
                        {task.creator?.name && (
                          <span className="text-gray-400 truncate">• {task.creator.name}</span>
                        )}
                      </div>

                      <span className="text-[11px] text-gray-500 flex-shrink-0">
                        {task.createdAt ? new Date(task.createdAt).toLocaleTimeString() : ""}
                      </span>
                    </div>

                    {/* Command Input */}
                    <div className="p-2.5 font-mono text-gray-200 bg-[#14171f] whitespace-pre-wrap break-all max-h-32 overflow-y-auto">
                      {task.input}
                    </div>

                    {/* Knowledge Graph Artifacts Reported by this Task */}
                    {((task.reportedCredentials && task.reportedCredentials.length > 0) ||
                      (task.reportedFiles && task.reportedFiles.length > 0) ||
                      (task.reportedProcesses && task.reportedProcesses.length > 0)) && (
                      <div className="px-2.5 py-1.5 bg-[#161b26] border-t border-[#262b38] flex flex-wrap items-center gap-1.5 text-[11px]">
                        {task.reportedCredentials && task.reportedCredentials.length > 0 && (
                          <span className="inline-flex items-center gap-1 px-1.5 py-0.5 rounded bg-amber-500/15 text-amber-300 border border-amber-500/30 font-medium">
                            <KeyRound size={10} />
                            {task.reportedCredentials.length} Cred
                            {task.reportedCredentials.length === 1 ? "" : "s"} Reported
                          </span>
                        )}
                        {task.reportedFiles && task.reportedFiles.length > 0 && (
                          <span className="inline-flex items-center gap-1 px-1.5 py-0.5 rounded bg-blue-500/15 text-blue-300 border border-blue-500/30 font-medium">
                            <FileText size={10} />
                            {task.reportedFiles.length} File
                            {task.reportedFiles.length === 1 ? "" : "s"} Reported
                          </span>
                        )}
                        {task.reportedProcesses && task.reportedProcesses.length > 0 && (
                          <span className="inline-flex items-center gap-1 px-1.5 py-0.5 rounded bg-emerald-500/15 text-emerald-300 border border-emerald-500/30 font-medium">
                            <Cpu size={10} />
                            {task.reportedProcesses.length} Process
                            {task.reportedProcesses.length === 1 ? "" : "es"}
                          </span>
                        )}
                      </div>
                    )}

                    {/* Action Bar */}
                    <div className="flex items-center justify-between px-2.5 py-1.5 bg-[#1b1f29] border-t border-[#262b38]">
                      <div>
                        {hasOutput ? (
                          <button
                            type="button"
                            onClick={() => toggleExpand(task.id)}
                            className="inline-flex items-center gap-1 text-purple-300 hover:text-purple-200 font-medium"
                          >
                            {isExpanded ? <ChevronDown size={13} /> : <ChevronRight size={13} />}
                            {isExpanded ? "Hide Output" : "View Output"}
                          </button>
                        ) : (
                          <span className="text-gray-500 italic">
                            {isQueued
                              ? "Waiting for beacon check-in"
                              : isRunning
                              ? "Executing in interpreter..."
                              : "No output"}
                          </span>
                        )}
                      </div>

                      <div className="flex items-center gap-1.5">
                        <button
                          type="button"
                          onClick={() => copyText(task.input, `in-${task.id}`)}
                          title="Copy command"
                          className="inline-flex items-center gap-1 px-2 py-0.5 rounded bg-[#262c3a] hover:bg-[#32394a] text-gray-300 transition-colors"
                        >
                          <Copy size={11} />
                          {copiedId === `in-${task.id}` ? "Copied" : "Copy"}
                        </button>
                        <button
                          type="button"
                          onClick={() => onLoadIntoComposer(task.input)}
                          title="Load into Multi-Line Composer"
                          className="inline-flex items-center gap-1 px-2 py-0.5 rounded bg-[#262c3a] hover:bg-[#32394a] text-gray-300 transition-colors"
                        >
                          <Code2 size={11} />
                          Edit
                        </button>
                        <button
                          type="button"
                          onClick={() => onRunCode(task.input)}
                          title="Re-run command immediately"
                          className="inline-flex items-center gap-1 px-2 py-0.5 rounded bg-purple-600/80 hover:bg-purple-600 text-white font-medium transition-colors"
                        >
                          <Play size={11} />
                          Re-run
                        </button>
                      </div>
                    </div>

                    {/* Expandable Output / Error Transcript */}
                    {isExpanded && hasOutput && (
                      <div className="border-t border-[#2b3140] bg-[#0e1015] p-2.5 font-mono text-[11px]">
                        <div className="flex items-center justify-between mb-1 text-gray-500">
                          <span>Execution Output</span>
                          <button
                            type="button"
                            onClick={() =>
                              copyText(
                                [task.output, task.error].filter(Boolean).join("\n"),
                                `out-${task.id}`
                              )
                            }
                            className="inline-flex items-center gap-1 text-gray-400 hover:text-white"
                          >
                            <Copy size={11} />
                            {copiedId === `out-${task.id}` ? "Copied!" : "Copy Output"}
                          </button>
                        </div>
                        {task.output && (
                          <pre className="text-gray-300 whitespace-pre-wrap break-words max-h-60 overflow-y-auto">
                            {task.output}
                          </pre>
                        )}
                        {task.error && (
                          <pre className="text-red-400 whitespace-pre-wrap break-words max-h-60 overflow-y-auto mt-1">
                            {task.error}
                          </pre>
                        )}
                      </div>
                    )}
                  </div>
                );
              })
            ) : filteredLocalHistory.length > 0 ? (
              <div className="space-y-2">
                <div className="text-[11px] text-gray-400 px-1">
                  Recent Browser Command History ({filteredLocalHistory.length})
                </div>
                {filteredLocalHistory.slice(0, 40).map((cmd, idx) => (
                  <div
                    key={idx}
                    className="rounded-md bg-[#1b1f29] border border-[#2b3140] p-2.5 text-xs space-y-2"
                  >
                    <pre className="font-mono text-gray-200 whitespace-pre-wrap break-all max-h-24 overflow-y-auto">
                      {cmd}
                    </pre>
                    <div className="flex justify-end gap-1.5">
                      <button
                        type="button"
                        onClick={() => onLoadIntoComposer(cmd)}
                        className="inline-flex items-center gap-1 px-2 py-0.5 rounded bg-[#262c3a] hover:bg-[#32394a] text-gray-300"
                      >
                        <Code2 size={11} />
                        Edit
                      </button>
                      <button
                        type="button"
                        onClick={() => onRunCode(cmd)}
                        className="inline-flex items-center gap-1 px-2 py-0.5 rounded bg-purple-600/80 hover:bg-purple-600 text-white"
                      >
                        <Play size={11} />
                        Re-run
                      </button>
                    </div>
                  </div>
                ))}
              </div>
            ) : (
              <div className="text-center py-8 text-xs text-gray-500">
                No matching commands in history yet.
              </div>
            )}
          </div>
        </>
      ) : (
        /* Pivots Sub-Tab */
        <div className="flex-1 overflow-y-auto p-2.5 space-y-2">
          {pivots.length === 0 ? (
            <div className="text-center py-8 px-4 text-xs text-gray-400 space-y-2">
              <p className="font-medium text-gray-300">No SSH or PTY pivot sessions yet.</p>
              <p className="text-gray-500">
                Once an interactive portal is active (<code className="font-mono text-purple-300">portal.create()</code>),
                you can open PTY or SSH sessions from the toolbar or by running <code className="font-mono text-purple-300">pty()</code> /{" "}
                <code className="font-mono text-purple-300">ssh(&quot;user:pass@host:22&quot;)</code>.
              </p>
            </div>
          ) : (
            [...pivots].reverse().map((pivot) => {
              const isSsh = pivot.kind?.toUpperCase() === "SSH";
              const isClosed = !!pivot.closedAt;
              return (
                <div
                  key={pivot.id}
                  className="rounded-md bg-[#1b1f29] border border-[#2b3140] p-3 text-xs space-y-2"
                >
                  <div className="flex items-center justify-between">
                    <div className="flex items-center gap-2 min-w-0">
                      {isSsh ? (
                        <Terminal size={14} className="text-purple-400 flex-shrink-0" />
                      ) : (
                        <SquareTerminal size={14} className="text-emerald-400 flex-shrink-0" />
                      )}
                      <span className="font-mono font-semibold text-gray-200 truncate">
                        {isSsh ? `SSH • ${pivot.destination}` : `PTY #${pivot.id}`}
                      </span>
                    </div>
                    <span
                      className={`px-2 py-0.5 rounded-full text-[11px] font-medium ${
                        isClosed
                          ? "bg-gray-700/60 text-gray-300"
                          : "bg-emerald-500/15 text-emerald-400 border border-emerald-500/30"
                      }`}
                    >
                      {isClosed ? "Closed" : "Active"}
                    </span>
                  </div>

                  {pivot.closedAt && (
                    <div className="text-[11px] text-gray-500">
                      Closed at {new Date(pivot.closedAt).toLocaleString()}
                    </div>
                  )}

                  <div className="flex justify-end">
                    <button
                      type="button"
                      onClick={() =>
                        onOpenPivotTab(
                          isSsh ? "ssh" : "pty",
                          isSsh ? pivot.destination : `PTY #${pivot.id}`,
                          pivot.id
                        )
                      }
                      className="inline-flex items-center gap-1.5 px-2.5 py-1 rounded bg-[#262c3a] hover:bg-[#32394a] text-gray-200 font-medium transition-colors"
                    >
                      {isClosed ? "View Saved Transcript" : "Switch to Tab"}
                    </button>
                  </div>
                </div>
              );
            })
          )}
        </div>
      )}
    </aside>
  );
};

export default ShellHistoryDrawer;
