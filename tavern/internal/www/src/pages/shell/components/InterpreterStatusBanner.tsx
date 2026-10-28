import React, { useState } from "react";
import { AlertTriangle, Clock, ExternalLink, ListOrdered, Loader2, Terminal, WifiOff } from "lucide-react";
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
}) => {
  const [dismissedStuckTaskId, setDismissedStuckTaskId] = useState<string | null>(null);

  const showStuckAlert = Boolean(stuckTask && stuckTask.task.id !== dismissedStuckTaskId);
  const showDisconnected = !closedAt && connectionStatus !== "connected";
  const showQueuedStrip =
    !closedAt && !showStuckAlert && (queuedTasks.length > 0 || runningTasks.length > 0);

  if (!closedAt && !showDisconnected && !showStuckAlert && !showQueuedStrip) {
    return null;
  }

  return (
    <div className="flex flex-col gap-2 mb-2">
      {/* 1. Terminated Shell Banner */}
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
              Spawn new shell
            </button>
          </div>
        </div>
      )}

      {/* 2. WebSocket Disconnected Banner */}
      {showDisconnected && (
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

      {/* 3. Stuck / Long-Running Interpreter Banner */}
      {!closedAt && showStuckAlert && stuckTask && (
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
              Spawn new shell
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

      {/* 4. Calm Queued Execution Strip (for slow beacons / queued tasks) */}
      {showQueuedStrip && (
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
