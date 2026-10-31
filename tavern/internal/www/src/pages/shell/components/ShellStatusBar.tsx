import React, { useState } from "react";
import { ConnectionStatus } from "../../../lib/browser-adapter";
import {
  Wifi,
  WifiOff,
  RefreshCw,
  Clock,
  Code2,
  Loader2,
  NotebookPen,
  Settings,
  Shell,
  Unplug,
  X,
} from "lucide-react";
import NotificationBell from "../../../components/notifications/NotificationBell";
import { ShellTaskItem } from "../hooks/useShellData";
import { CallbackStats, computeCallbackStats } from "../hooks/useCallbackTimer";
import { stripInternalMacro } from "../hooks/shellUtils";

interface ShellStatusBarProps {
  portalId: number | null;
  timeUntilCallback: string;
  isMissedCallback: boolean;
  callbackStats?: CallbackStats;
  callbackInterval?: number | null;
  connectionStatus: ConnectionStatus;
  connectionMessage?: string;
  isConnecting?: boolean;
  closedAt?: string | null;
  queuedCount?: number;
  runningCount?: number;
  queuedTasks?: ShellTaskItem[];
  runningTasks?: ShellTaskItem[];
  queueTourOpen?: boolean;
  sidebarOpen?: boolean;
  onToggleSidebar?: () => void;
  sessionSettingsOpen?: boolean;
  onToggleSessionSettings?: () => void;
  composerOpen?: boolean;
  onToggleComposer?: () => void;
  scratchpadOpen?: boolean;
  onToggleScratchpad?: () => void;
  onNewPortal?: () => void;
  onClosePortal?: () => void;
  onOpenQueue?: () => void;
  onReconnect?: () => void;
}

const truncateCommand = (cmd: string, maxLen = 72): string => {
  const cleaned = stripInternalMacro(cmd);
  const firstLine = cleaned.trim().split("\n")[0] || "";
  if (firstLine.length <= maxLen) return firstLine;
  return `${firstLine.slice(0, maxLen)}...`;
};

const StatusIconWithPopover = ({
  icon,
  message,
  onClick,
}: {
  icon: React.ReactNode;
  message?: string;
  onClick?: () => void;
}) => {
  const [showPopover, setShowPopover] = useState(false);

  return (
    <div
      className={`relative flex items-center ${onClick ? "cursor-pointer" : ""}`}
      onMouseEnter={() => setShowPopover(true)}
      onMouseLeave={() => setShowPopover(false)}
      onClick={onClick}
    >
      {icon}
      {showPopover && message && (
        <div className="absolute bottom-full right-0 mb-2 w-64 p-2 bg-gray-900 border border-gray-700 rounded-md shadow-lg text-xs text-gray-200 z-50 break-words pointer-events-none">
          {message}
        </div>
      )}
    </div>
  );
};

const PortalActionDialog: React.FC<{
  mode: "open" | "close" | null;
  onCancel: () => void;
  onConfirmOpen?: () => void;
  onConfirmClose?: () => void;
}> = ({ mode, onCancel, onConfirmOpen, onConfirmClose }) => {
  if (!mode) {
    return null;
  }

  const isClose = mode === "close";

  const handleConfirm = () => {
    if (isClose) {
      onConfirmClose?.();
    } else {
      onConfirmOpen?.();
    }
    onCancel();
  };

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 p-4"
      onClick={onCancel}
    >
      <div
        role="dialog"
        aria-modal="true"
        aria-label={isClose ? "Close Portal" : "Open Portal"}
        data-testid={isClose ? "close-portal-dialog" : "open-portal-dialog"}
        className="w-full max-w-md rounded-lg bg-gray-900 border border-gray-700 shadow-2xl p-4 text-xs text-gray-200 space-y-3"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="flex items-center justify-between border-b border-gray-800 pb-2.5">
          <div className="flex items-center gap-2 font-semibold text-sm text-white">
            {isClose ? (
              <Unplug className="w-4 h-4 text-red-400" />
            ) : (
              <Shell className="w-4 h-4 text-purple-400" />
            )}
            <span>{isClose ? "Close Portal" : "Open Portal"}</span>
          </div>
          <button
            type="button"
            onClick={onCancel}
            aria-label="Close dialog"
            className="text-gray-400 hover:text-white transition-colors"
          >
            <X className="w-4 h-4" />
          </button>
        </div>

        <p className="text-gray-300 leading-relaxed">
          {isClose
            ? "Are you sure you want to close the active portal? Warning: closing this portal will immediately terminate and close all associated pivots (including active SSH and PTY sessions)."
            : "Opens an interactive portal on this beacon, which works similar to a SOCKS5 proxy. This establishes a real-time stream between the agent and server for low-latency execution, interactive PTY, or pivoting operations."}
        </p>

        <div className="flex items-center justify-end gap-2 pt-1">
          <button
            type="button"
            onClick={onCancel}
            className="px-3 py-1.5 rounded-md font-semibold text-gray-300 hover:bg-gray-800 transition-colors"
          >
            Cancel
          </button>
          <button
            type="button"
            onClick={handleConfirm}
            className={`px-3.5 py-1.5 rounded-md font-semibold text-white transition-colors ${
              isClose
                ? "bg-red-600 hover:bg-red-500"
                : "bg-purple-600 hover:bg-purple-500"
            }`}
          >
            {isClose ? "Close Portal" : "Open Portal"}
          </button>
        </div>
      </div>
    </div>
  );
};

const CallbackStatsPopover: React.FC<{
  stats: CallbackStats;
  timeUntilCallback: string;
  isMissedCallback: boolean;
}> = ({ stats, timeUntilCallback, isMissedCallback }) => {
  const statusLabel = isMissedCallback ? "Delayed" : "On Schedule";
  const statusBadgeClass = isMissedCallback
    ? "bg-red-500/15 text-red-400 border-red-500/30"
    : "bg-emerald-500/15 text-emerald-400 border-emerald-500/30";
  const barColorClass =
    stats.onTimePercentage >= 90
      ? "bg-emerald-500"
      : stats.onTimePercentage >= 70
      ? "bg-amber-500"
      : "bg-red-500";

  return (
    <div
      data-testid="status-bar-callback-popover"
      role="tooltip"
      aria-label="Callback statistics"
      className="absolute bottom-full right-0 mb-2 w-64 p-2.5 bg-gray-900 border border-gray-700 rounded-lg shadow-xl text-xs text-gray-200 z-50 space-y-2 pointer-events-none"
    >
      <div className="flex items-center justify-between border-b border-gray-800 pb-1.5">
        <span className="font-semibold text-white">Callback Stats</span>
        <span
          className={`px-1.5 py-0.5 rounded text-[10px] font-semibold border ${statusBadgeClass}`}
        >
          {statusLabel}
        </span>
      </div>

      <div className="space-y-1.5">
        <div className="flex items-center justify-between">
          <span className="text-gray-400">Callback Interval</span>
          <span
            data-testid="callback-stats-interval"
            className="font-mono font-semibold text-gray-100"
          >
            {stats.formattedInterval}
          </span>
        </div>

        <div className="flex items-center justify-between">
          <span className="text-gray-400">On-Time (Past 100)</span>
          <span
            data-testid="callback-stats-ontime"
            className="font-mono font-semibold text-gray-100"
          >
            {stats.onTimeCount} / {stats.sampleCount} ({stats.onTimePercentage}%)
          </span>
        </div>

        <div className="w-full h-1.5 rounded-full bg-gray-800 overflow-hidden border border-gray-700">
          <div
            className={`h-full transition-all ${barColorClass}`}
            style={{ width: `${Math.min(100, Math.max(0, stats.onTimePercentage))}%` }}
          />
        </div>

        <div className="flex items-center justify-between text-xs text-gray-400 pt-0.5">
          <span>Next Check-in</span>
          <span className={isMissedCallback ? "text-red-400 font-semibold" : "text-gray-300"}>
            {timeUntilCallback}
          </span>
        </div>

        {stats.avgLatencyMs !== null && (
          <div className="flex items-center justify-between text-xs text-gray-400">
            <span>Avg Drift</span>
            <span className="font-mono text-gray-300">{stats.avgLatencyMs}ms</span>
          </div>
        )}
      </div>
    </div>
  );
};

const ShellStatusBar: React.FC<ShellStatusBarProps> = ({
  portalId,
  timeUntilCallback,
  isMissedCallback,
  callbackStats,
  callbackInterval,
  connectionStatus,
  connectionMessage,
  isConnecting = false,
  closedAt,
  queuedCount,
  runningCount,
  queuedTasks = [],
  runningTasks = [],
  queueTourOpen = false,
  sidebarOpen = false,
  onToggleSidebar,
  sessionSettingsOpen = false,
  onToggleSessionSettings,
  composerOpen = false,
  onToggleComposer,
  scratchpadOpen = false,
  onToggleScratchpad,
  onNewPortal,
  onClosePortal,
  onOpenQueue,
  onReconnect,
}) => {
  const [callbackHovered, setCallbackHovered] = useState(false);
  const [queueHovered, setQueueHovered] = useState(false);
  const [queuePinned, setQueuePinned] = useState(false);
  const [portalDialogMode, setPortalDialogMode] = useState<"open" | "close" | null>(null);

  const effectiveCallbackStats =
    callbackStats ?? computeCallbackStats(null, callbackInterval);

  const effectiveRunningCount = runningCount ?? runningTasks.length;
  const effectiveQueuedCount = queuedCount ?? queuedTasks.length;
  const hasActiveQueue = effectiveRunningCount > 0 || effectiveQueuedCount > 0;
  const showQueuePopover = (queueHovered || queuePinned) && hasActiveQueue;

  const getConnectionStatusIcon = () => {
    if (isConnecting) {
      return null;
    }
    switch (connectionStatus) {
      case "connected":
        return <Wifi className="w-4 h-4 text-green-500" />;
      case "disconnected":
        return (
          <StatusIconWithPopover
            icon={<WifiOff className="w-4 h-4 text-red-500" />}
            message={connectionMessage || "Disconnected (click to reconnect)"}
            onClick={onReconnect}
          />
        );
      case "reconnecting":
        return (
          <StatusIconWithPopover
            icon={<RefreshCw className="w-4 h-4 text-yellow-500 animate-spin" />}
            message={connectionMessage || "Reconnecting... (click to retry now)"}
            onClick={onReconnect}
          />
        );
    }
  };

  return (
    <>
      <div
        data-tour="shell-status-bar"
        className="relative z-30 bg-gray-900 px-3 py-1 border-t border-gray-800 text-xs text-gray-400 flex justify-between items-center select-none"
      >
        {/* Left side: Notification Bell, Session Settings, Composer */}
        <div className="flex items-center gap-1.5">
          <NotificationBell
            placement="top-start"
            buttonClassName="relative inline-flex items-center justify-center p-1 rounded-md transition-colors text-gray-400 hover:text-white hover:bg-gray-800"
            iconClassName="w-4 h-4"
          />

          <button
            type="button"
            data-tour="shell-session-settings-toggle"
            data-testid="session-settings-toggle"
            aria-label="Session settings"
            aria-expanded={sessionSettingsOpen}
            title="Session settings (RC)"
            onClick={onToggleSessionSettings}
            className={`inline-flex items-center justify-center p-1 rounded-md transition-colors ${
              sessionSettingsOpen
                ? "text-white bg-purple-600"
                : "text-gray-400 hover:text-white hover:bg-gray-800"
            }`}
          >
            <Settings className="w-4 h-4" />
          </button>

          {onToggleComposer && (
            <button
              type="button"
              data-tour="shell-composer-toggle"
              data-testid="status-bar-composer-toggle"
              aria-label="Toggle multi-line composer"
              aria-expanded={composerOpen}
              title="Eldritch Composer"
              onClick={onToggleComposer}
              className={`inline-flex items-center justify-center p-1 rounded-md transition-colors ${
                composerOpen
                  ? "text-white bg-purple-600"
                  : "text-gray-400 hover:text-white hover:bg-gray-800"
              }`}
            >
              <Code2 className="w-4 h-4" />
            </button>
          )}

          <button
            type="button"
            data-tour="shell-scratchpad-toggle"
            data-testid="status-bar-scratchpad-toggle"
            aria-label="Toggle scratchpad"
            aria-expanded={scratchpadOpen}
            title="Scratchpad"
            onClick={onToggleScratchpad}
            className={`inline-flex items-center justify-center p-1 rounded-md transition-colors ${
              scratchpadOpen
                ? "text-white bg-purple-600"
                : "text-gray-400 hover:text-white hover:bg-gray-800"
            }`}
          >
            <NotebookPen className="w-4 h-4" />
          </button>

          {closedAt && (
            <span className="flex items-center gap-2 text-red-400 font-semibold ml-1">
              <span className="w-2 h-2 rounded-full bg-red-500"></span>
              <span>
                Terminated:{" "}
                {new Date(closedAt).toLocaleString(undefined, {
                  dateStyle: "medium",
                  timeStyle: "short",
                })}
              </span>
            </span>
          )}
        </div>

        {/* Right side: Next / Queue / Async (or Portal Active) / Connection Status */}
        <div className="flex items-center gap-2">
          {!closedAt && !portalId && timeUntilCallback && (
            <div
              data-tour="shell-next-callback"
              data-testid="status-bar-next-callback"
              className="relative inline-grid place-items-center min-w-[8.5rem] px-2 py-0.5 rounded-md hover:bg-gray-800 transition-colors cursor-default whitespace-nowrap after:content-['Next_in_a_few_seconds'] after:col-start-1 after:row-start-1 after:invisible after:pointer-events-none after:select-none"
              onMouseEnter={() => setCallbackHovered(true)}
              onMouseLeave={() => setCallbackHovered(false)}
            >
              <span className="col-start-1 row-start-1 inline-flex items-center gap-1.5">
                <span>Next</span>
                <span className={isMissedCallback ? "text-red-500 font-bold" : "text-gray-300"}>
                  {timeUntilCallback}
                </span>
              </span>
              {callbackHovered && (
                <CallbackStatsPopover
                  stats={effectiveCallbackStats}
                  timeUntilCallback={timeUntilCallback}
                  isMissedCallback={isMissedCallback}
                />
              )}
            </div>
          )}

          {(hasActiveQueue || queueTourOpen) && (
            <div
              className="relative flex items-center"
              onMouseEnter={() => setQueueHovered(true)}
              onMouseLeave={() => setQueueHovered(false)}
            >
              <button
                type="button"
                data-tour="shell-queue-indicator"
                data-testid="status-bar-queue-button"
                aria-label="View active task queue"
                aria-expanded={showQueuePopover}
                onClick={() => {
                  setQueuePinned((prev) => !prev);
                  onOpenQueue?.();
                }}
                className="flex items-center gap-2 px-2 py-0.5 rounded-md hover:bg-gray-800 transition-colors cursor-pointer"
              >
                {effectiveRunningCount > 0 && (
                  <span className="inline-flex items-center gap-1 text-blue-300">
                    <Loader2 className="w-3.5 h-3.5 animate-spin" />
                    {effectiveRunningCount} executing
                  </span>
                )}
                {(effectiveQueuedCount > 0 || (!hasActiveQueue && queueTourOpen)) && (
                  <span className="inline-flex items-center gap-1 text-amber-300">
                    <Clock className="w-3.5 h-3.5" />
                    {effectiveQueuedCount > 0 ? effectiveQueuedCount : 1} queued
                  </span>
                )}
              </button>

              {showQueuePopover && (
                <div
                  data-testid="status-bar-queue-popover"
                  role="dialog"
                  aria-label="Queued and executing commands"
                  className="absolute bottom-full right-0 mb-2 w-80 max-h-64 overflow-y-auto shell-scrollbar p-2.5 bg-gray-900 border border-gray-700 rounded-lg shadow-xl text-xs text-gray-200 z-50 space-y-2"
                >
                  <div className="flex items-center justify-between border-b border-gray-800 pb-1.5">
                    <span className="font-semibold text-white">Active Queue</span>
                    <span className="text-xs text-gray-400">
                      {effectiveRunningCount} executing • {effectiveQueuedCount} queued
                    </span>
                  </div>

                  {runningTasks.length > 0 || queuedTasks.length > 0 ? (
                    <div className="space-y-1.5">
                      {runningTasks.map((task) => (
                        <div
                          key={task.id}
                          className="flex items-start gap-2 p-2 rounded-md bg-gray-800/70 border border-gray-700"
                        >
                          <Loader2 className="w-3.5 h-3.5 text-blue-400 animate-spin shrink-0 mt-0.5" />
                          <div className="min-w-0 flex-1">
                            <div className="text-[10px] font-semibold uppercase tracking-wide text-blue-300">
                              Executing
                            </div>
                            <code className="font-mono text-xs text-gray-200 block truncate">
                              {truncateCommand(task.input)}
                            </code>
                          </div>
                        </div>
                      ))}
                      {queuedTasks.map((task) => (
                        <div
                          key={task.id}
                          className="flex items-start gap-2 p-2 rounded-md bg-gray-800/70 border border-gray-700"
                        >
                          <Clock className="w-3.5 h-3.5 text-amber-400 shrink-0 mt-0.5" />
                          <div className="min-w-0 flex-1">
                            <div className="text-[10px] font-semibold uppercase tracking-wide text-amber-300">
                              Queued
                            </div>
                            <code className="font-mono text-xs text-gray-200 block truncate">
                              {truncateCommand(task.input)}
                            </code>
                          </div>
                        </div>
                      ))}
                    </div>
                  ) : (
                    <div className="text-gray-400">
                      {effectiveRunningCount > 0 && `${effectiveRunningCount} executing`}
                      {effectiveRunningCount > 0 && effectiveQueuedCount > 0 && ", "}
                      {effectiveQueuedCount > 0 && `${effectiveQueuedCount} queued`}
                    </div>
                  )}
                </div>
              )}
            </div>
          )}

          {!closedAt &&
            (portalId ? (
              <button
                type="button"
                data-tour="shell-non-interactive"
                data-testid="status-bar-portal-active-button"
                onClick={() => setPortalDialogMode("close")}
                className="inline-flex items-center gap-1.5 px-2 py-0.5 rounded-md hover:bg-gray-800 text-green-400 font-semibold transition-colors cursor-pointer"
              >
                <span className="w-2 h-2 rounded-full bg-green-400 animate-pulse"></span>
                <span>Portal Active</span>
              </button>
            ) : (
              <button
                type="button"
                data-tour="shell-non-interactive"
                data-testid="status-bar-async-button"
                onClick={() => setPortalDialogMode("open")}
                className="inline-flex items-center gap-1.5 px-2 py-0.5 rounded-md hover:bg-gray-800 transition-colors cursor-pointer"
              >
                <span className="w-2 h-2 rounded-full bg-yellow-500"></span>
                <span className="text-yellow-500">async</span>
              </button>
            ))}

          <div className="flex items-center gap-2 pl-1">{getConnectionStatusIcon()}</div>
        </div>
      </div>

      <PortalActionDialog
        mode={portalDialogMode}
        onCancel={() => setPortalDialogMode(null)}
        onConfirmOpen={onNewPortal}
        onConfirmClose={onClosePortal}
      />
    </>
  );
};

export default ShellStatusBar;
