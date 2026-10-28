import React, { useState } from "react";
import { AlertTriangle, Clock, ExternalLink, ListOrdered, Loader2, Sparkles, Terminal, WifiOff, X } from "lucide-react";
import { ShellTaskItem } from "../hooks/useShellData";
import { ConnectionStatus } from "../../../lib/browser-adapter";

interface InterpreterStatusBannerProps {
  closedAt?: string | null;
  connectionStatus: ConnectionStatus;
  connectionMessage?: string;
  stuckTask: { task: ShellTaskItem; elapsedSeconds: number } | null;
  queuedTasks: ShellTaskItem[];
  runningTasks: ShellTaskItem[];
  timeUntilCallback: string;
  onSpawnFreshShell: () => void;
  isSpawningShell?: boolean;
  onOpenHistoryDrawer: () => void;
  onOpenNuxDrawer: () => void;
  showNuxWelcome: boolean;
  onDismissNuxWelcome: () => void;
  portalId: number | null;
  onOpenPortal: () => void;
}

const InterpreterStatusBanner: React.FC<InterpreterStatusBannerProps> = ({
  closedAt,
  connectionStatus,
  connectionMessage,
  stuckTask,
  queuedTasks,
  runningTasks,
  timeUntilCallback,
  onSpawnFreshShell,
  isSpawningShell = false,
  onOpenHistoryDrawer,
  onOpenNuxDrawer,
  showNuxWelcome,
  onDismissNuxWelcome,
  portalId,
  onOpenPortal,
}) => {
  const [dismissedStuckTaskId, setDismissedStuckTaskId] = useState<string | null>(null);

  const showStuckAlert = stuckTask && stuckTask.task.id !== dismissedStuckTaskId;

  return (
    <div className="flex flex-col gap-2 mb-2">
      {/* 1. First-time Operator NUX Welcome Strip (non-blocking) */}
      {showNuxWelcome && !closedAt && (
        <div
          data-testid="shell-nux-banner"
          className="flex flex-wrap items-center justify-between gap-2 px-3.5 py-2 rounded-md bg-gradient-to-r from-purple-950/80 via-[#1f2433] to-indigo-950/80 border border-purple-500/30 text-xs text-gray-200"
        >
          <div className="flex items-center gap-2.5 min-w-0">
            <span className="p-1 rounded bg-purple-500/20 text-purple-300 flex-shrink-0">
              <Sparkles size={14} />
            </span>
            <div className="truncate">
              <span className="font-semibold text-white">Welcome to the Eldritch Shell.</span>{" "}
              <span className="text-gray-300">
                Run <code className="px-1 py-0.5 rounded bg-black/40 text-purple-300 font-mono">!cmd</code> for shell commands,{" "}
                <code className="px-1 py-0.5 rounded bg-black/40 text-emerald-300 font-mono">portal.create()</code> for low-latency interactive mode &amp; PTY/SSH pivots, or use the{" "}
                <strong className="text-white">Multi-Line Composer</strong> below.
              </span>
            </div>
          </div>
          <div className="flex items-center gap-2 flex-shrink-0">
            {!portalId && (
              <button
                type="button"
                onClick={onOpenPortal}
                className="px-2.5 py-1 rounded bg-emerald-600/80 hover:bg-emerald-600 text-white font-medium transition-colors"
              >
                Open Interactive Portal
              </button>
            )}
            <button
              type="button"
              onClick={onOpenNuxDrawer}
              className="px-2.5 py-1 rounded bg-purple-600/80 hover:bg-purple-600 text-white font-medium transition-colors"
            >
              Operator Guide &amp; Docs
            </button>
            <button
              type="button"
              onClick={onDismissNuxWelcome}
              aria-label="Dismiss welcome guide"
              className="p-1 rounded text-gray-400 hover:text-white hover:bg-white/10 transition-colors"
            >
              <X size={14} />
            </button>
          </div>
        </div>
      )}

      {/* 2. Terminated Shell Banner */}
      {closedAt && (
        <div
          role="alert"
          className="flex flex-wrap items-center justify-between gap-2 px-3.5 py-2.5 rounded-md bg-red-950/70 border border-red-500/40 text-xs text-red-100"
        >
          <div className="flex items-center gap-2">
            <AlertTriangle size={16} className="text-red-400 flex-shrink-0" />
            <div>
              <span className="font-semibold">Shell Session Terminated</span>{" "}
              <span className="text-red-200/90">
                (closed at {new Date(closedAt).toLocaleString()}). This terminal is read-only, but full command history remains accessible.
              </span>
            </div>
          </div>
          <div className="flex items-center gap-2">
            <button
              type="button"
              onClick={onOpenHistoryDrawer}
              className="px-2.5 py-1 rounded bg-black/30 hover:bg-black/50 text-red-100 font-medium transition-colors"
            >
              View History
            </button>
            <button
              type="button"
              onClick={onSpawnFreshShell}
              disabled={isSpawningShell}
              className="inline-flex items-center gap-1.5 px-3 py-1 rounded bg-red-600 hover:bg-red-500 disabled:opacity-50 text-white font-semibold transition-colors"
            >
              {isSpawningShell ? <Loader2 size={13} className="animate-spin" /> : <Terminal size={13} />}
              Spawn Fresh Shell
            </button>
          </div>
        </div>
      )}

      {/* 3. WebSocket Disconnected Banner */}
      {!closedAt && connectionStatus !== "connected" && (
        <div
          role="alert"
          className="flex flex-wrap items-center justify-between gap-2 px-3.5 py-2 rounded-md bg-red-950/60 border border-red-500/40 text-xs text-red-200"
        >
          <div className="flex items-center gap-2">
            <WifiOff size={15} className="text-red-400 flex-shrink-0" />
            <span>
              <strong className="text-red-100">
                {connectionStatus === "reconnecting" ? "Reconnecting to Tavern..." : "Disconnected from Tavern WebSocket."}
              </strong>{" "}
              {connectionMessage && <span>({connectionMessage}) </span>}
              Terminal input will resume automatically once connected.
            </span>
          </div>
        </div>
      )}

      {/* 4. Stuck / Long-Running Interpreter Banner */}
      {!closedAt && showStuckAlert && (
        <div
          role="alert"
          data-testid="stuck-interpreter-banner"
          className="flex flex-wrap items-center justify-between gap-3 px-3.5 py-2.5 rounded-md bg-amber-950/75 border border-amber-500/50 text-xs text-amber-100"
        >
          <div className="flex items-start gap-2.5 min-w-0 flex-1">
            <AlertTriangle size={16} className="text-amber-400 flex-shrink-0 mt-0.5" />
            <div className="min-w-0">
              <div className="flex items-center gap-2 flex-wrap">
                <span className="font-semibold text-amber-200">
                  Interpreter Busy or Potentially Stuck ({stuckTask.elapsedSeconds}s elapsed)
                </span>
                <code className="px-1.5 py-0.5 rounded bg-black/40 text-amber-300 font-mono truncate max-w-xs">
                  {stuckTask.task.input.split("\n")[0]}
                </code>
              </div>
              <p className="text-amber-200/80 mt-0.5">
                The agent&apos;s Eldritch interpreter has been executing this task for {stuckTask.elapsedSeconds}s. You can keep queuing commands to run after it finishes, or transition to a new shell with a fresh interpreter.
              </p>
            </div>
          </div>
          <div className="flex items-center gap-2 flex-shrink-0">
            <button
              type="button"
              onClick={onSpawnFreshShell}
              disabled={isSpawningShell}
              className="inline-flex items-center gap-1.5 px-3 py-1.5 rounded bg-amber-500 hover:bg-amber-400 text-gray-950 font-semibold transition-colors disabled:opacity-50"
            >
              {isSpawningShell ? <Loader2 size={13} className="animate-spin" /> : <ExternalLink size={13} />}
              Spawn Fresh Shell (New Interpreter)
            </button>
            <button
              type="button"
              onClick={() => setDismissedStuckTaskId(stuckTask.task.id)}
              className="px-2.5 py-1.5 rounded bg-black/30 hover:bg-black/50 text-amber-200 transition-colors"
            >
              Keep Waiting
            </button>
          </div>
        </div>
      )}

      {/* 5. Calm Queued Execution Strip (for slow beacons / queued tasks) */}
      {!closedAt && !showStuckAlert && (queuedTasks.length > 0 || runningTasks.length > 0) && (
        <div
          data-testid="queued-tasks-strip"
          className="flex flex-wrap items-center justify-between gap-2 px-3.5 py-1.5 rounded-md bg-[#1b2234] border border-blue-500/30 text-xs text-blue-200"
        >
          <div className="flex items-center gap-2">
            {runningTasks.length > 0 ? (
              <Loader2 size={14} className="text-blue-400 animate-spin flex-shrink-0" />
            ) : (
              <Clock size={14} className="text-blue-400 flex-shrink-0" />
            )}
            <span>
              {runningTasks.length > 0 && (
                <strong className="text-blue-100 mr-1.5">
                  {runningTasks.length} executing
                </strong>
              )}
              {queuedTasks.length > 0 && (
                <>
                  <strong className="text-blue-100">
                    {queuedTasks.length} {queuedTasks.length === 1 ? "command" : "commands"} queued in sequence
                  </strong>
                  {timeUntilCallback && (
                    <span className="text-blue-300/90"> • Next beacon check-in {timeUntilCallback}</span>
                  )}
                  <span className="text-blue-300/75 hidden md:inline">
                    {" "}— additional commands you submit will run in order.
                  </span>
                </>
              )}
            </span>
          </div>
          <button
            type="button"
            onClick={onOpenHistoryDrawer}
            className="inline-flex items-center gap-1 px-2 py-0.5 rounded bg-blue-500/20 hover:bg-blue-500/30 text-blue-200 font-medium transition-colors"
          >
            <ListOrdered size={12} />
            Inspect Queue
          </button>
        </div>
      )}
    </div>
  );
};

export default InterpreterStatusBanner;
