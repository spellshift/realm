import React, { useState } from "react";
import { ConnectionStatus } from "../../../lib/browser-adapter";
import {
  Wifi,
  WifiOff,
  RefreshCw,
  Clock,
  Code2,
  Loader2,
  Settings,
  Shell,
  Unplug,
  X,
} from "lucide-react";
import NotificationBell from "../../../components/notifications/NotificationBell";
import { ShellTaskItem } from "../hooks/useShellData";

interface ShellStatusBarProps {
  portalId: number | null;
  timeUntilCallback: string;
  isMissedCallback: boolean;
  connectionStatus: ConnectionStatus;
  connectionMessage?: string;
  isConnecting?: boolean;
  closedAt?: string | null;
  queuedCount?: number;
  runningCount?: number;
  queuedTasks?: ShellTaskItem[];
  runningTasks?: ShellTaskItem[];
  queueTourOpen?: boolean;
  sessionSettingsOpen?: boolean;
  onToggleSessionSettings?: () => void;
  composerOpen?: boolean;
  onToggleComposer?: () => void;
  onNewPortal?: () => void;
  onClosePortal?: () => void;
  onOpenQueue?: () => void;
  onReconnect?: () => void;
}

const truncateCommand = (cmd: string, maxLen = 72): string => {
  const firstLine = cmd.trim().split("\n")[0] || "";
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
        <div className="absolute bottom-full right-0 mb-2 w-64 p-2 bg-[#1e222d] border border-[#2f364a] rounded shadow-lg text-xs text-gray-200 z-50 break-words pointer-events-none">
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
        className="w-full max-w-md rounded-lg bg-[#1c202b] border border-[#343b4d] shadow-2xl p-4 text-xs text-gray-200 space-y-3"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="flex items-center justify-between border-b border-[#2e3444] pb-2.5">
          <div className="flex items-center gap-2 font-semibold text-sm text-white">
            {isClose ? (
              <Unplug size={16} className="text-red-400" />
            ) : (
              <Shell size={16} className="text-purple-400" />
            )}
            <span>{isClose ? "Close Portal" : "Open Portal"}</span>
          </div>
          <button
            type="button"
            onClick={onCancel}
            aria-label="Close dialog"
            className="text-gray-400 hover:text-white transition-colors"
          >
            <X size={15} />
          </button>
        </div>

        <p className="text-gray-300 leading-relaxed">
          {isClose
            ? "Are you sure you want to close the active portal? Warning: closing this portal will immediately terminate and close all associated pivots (including active SSH and PTY sessions)."
            : "Open an interactive portal on this beacon? This establishes a real-time stream (portal.create()) for low-latency execution and interactive PTY or SSH pivots."}
        </p>

        <div className="flex items-center justify-end gap-2 pt-1">
          <button
            type="button"
            onClick={onCancel}
            className="px-3 py-1.5 rounded-md font-semibold text-gray-300 hover:bg-[#2c3242] transition-colors"
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

const ShellStatusBar: React.FC<ShellStatusBarProps> = ({
  portalId,
  timeUntilCallback,
  isMissedCallback,
  connectionStatus,
  connectionMessage,
  isConnecting = false,
  closedAt,
  queuedCount,
  runningCount,
  queuedTasks = [],
  runningTasks = [],
  queueTourOpen = false,
  sessionSettingsOpen = false,
  onToggleSessionSettings,
  composerOpen = false,
  onToggleComposer,
  onNewPortal,
  onClosePortal,
  onOpenQueue,
  onReconnect,
}) => {
  const [queueHovered, setQueueHovered] = useState(false);
  const [queuePinned, setQueuePinned] = useState(false);
  const [portalDialogMode, setPortalDialogMode] = useState<"open" | "close" | null>(null);

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
        return <Wifi className="w-3.5 h-3.5 text-green-500" />;
      case "disconnected":
        return (
          <StatusIconWithPopover
            icon={<WifiOff className="w-3.5 h-3.5 text-red-500" />}
            message={connectionMessage || "Disconnected (click to reconnect)"}
            onClick={onReconnect}
          />
        );
      case "reconnecting":
        return (
          <StatusIconWithPopover
            icon={<RefreshCw className="w-3.5 h-3.5 text-yellow-500 animate-spin" />}
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
        className="bg-[#161922] px-3 py-1 border-t border-[#262b38] text-xs text-gray-400 flex justify-between items-center select-none"
      >
        {/* Left side: Session Settings, Notification Bell, Composer */}
        <div className="flex items-center gap-2">
          <button
            type="button"
            data-tour="shell-session-settings-toggle"
            data-testid="session-settings-toggle"
            aria-label="Session settings"
            aria-expanded={sessionSettingsOpen}
            title="Session settings (RC)"
            onClick={onToggleSessionSettings}
            className={`inline-flex items-center justify-center p-1 rounded transition-colors ${
              sessionSettingsOpen
                ? "text-gray-100 bg-white/10"
                : "text-gray-400 hover:text-gray-200 hover:bg-white/5"
            }`}
          >
            <Settings className="w-3.5 h-3.5" />
          </button>

          <NotificationBell
            placement="top-start"
            buttonClassName="relative inline-flex items-center justify-center p-1 rounded transition-colors text-gray-400 hover:text-gray-200 hover:bg-white/5"
            iconClassName="w-3.5 h-3.5"
          />

          {onToggleComposer && (
            <button
              type="button"
              data-tour="shell-composer-toggle"
              data-testid="status-bar-composer-toggle"
              aria-label="Toggle multi-line composer"
              aria-expanded={composerOpen}
              title="Eldritch Composer"
              onClick={onToggleComposer}
              className={`inline-flex items-center justify-center p-1 rounded transition-colors ${
                composerOpen
                  ? "text-white bg-purple-600"
                  : "text-gray-400 hover:text-gray-200 hover:bg-white/5"
              }`}
            >
              <Code2 className="w-3.5 h-3.5" />
            </button>
          )}

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
        <div className="flex items-center gap-3">
          {!closedAt && !portalId && timeUntilCallback && (
            <span
              data-tour="shell-next-callback"
              className="inline-flex items-center gap-1.5"
            >
              <span>Next</span>
              <span className={isMissedCallback ? "text-red-500 font-bold" : "text-gray-300"}>
                {timeUntilCallback}
              </span>
            </span>
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
                className="flex items-center gap-2 px-1.5 py-0.5 rounded hover:bg-white/10 transition-colors cursor-pointer"
              >
                {effectiveRunningCount > 0 && (
                  <span className="inline-flex items-center gap-1 text-blue-300">
                    <Loader2 className="w-3 h-3 animate-spin" />
                    {effectiveRunningCount} executing
                  </span>
                )}
                {(effectiveQueuedCount > 0 || (!hasActiveQueue && queueTourOpen)) && (
                  <span className="inline-flex items-center gap-1 text-amber-300">
                    <Clock className="w-3 h-3" />
                    {effectiveQueuedCount > 0 ? effectiveQueuedCount : 1} queued
                  </span>
                )}
              </button>

              {showQueuePopover && (
                <div
                  data-testid="status-bar-queue-popover"
                  role="dialog"
                  aria-label="Queued and executing commands"
                  className="absolute bottom-full right-0 mb-2 w-80 max-h-64 overflow-y-auto shell-scrollbar p-2.5 bg-[#1a1e29] border border-[#2f364a] rounded-lg shadow-xl text-xs text-gray-200 z-50 space-y-2"
                >
                  <div className="flex items-center justify-between border-b border-[#262b38] pb-1.5">
                    <span className="font-semibold text-white">Active Queue</span>
                    <span className="text-[11px] text-gray-400">
                      {effectiveRunningCount} executing • {effectiveQueuedCount} queued
                    </span>
                  </div>

                  {runningTasks.length > 0 || queuedTasks.length > 0 ? (
                    <div className="space-y-1.5">
                      {runningTasks.map((task) => (
                        <div
                          key={task.id}
                          className="flex items-start gap-2 p-1.5 rounded bg-[#12151e] border border-[#262b38]"
                        >
                          <Loader2 className="w-3.5 h-3.5 text-blue-400 animate-spin shrink-0 mt-0.5" />
                          <div className="min-w-0 flex-1">
                            <div className="text-[10px] font-semibold uppercase tracking-wide text-blue-300">
                              Executing
                            </div>
                            <code className="font-mono text-[11px] text-gray-200 block truncate">
                              {truncateCommand(task.input)}
                            </code>
                          </div>
                        </div>
                      ))}
                      {queuedTasks.map((task) => (
                        <div
                          key={task.id}
                          className="flex items-start gap-2 p-1.5 rounded bg-[#12151e] border border-[#262b38]"
                        >
                          <Clock className="w-3.5 h-3.5 text-amber-400 shrink-0 mt-0.5" />
                          <div className="min-w-0 flex-1">
                            <div className="text-[10px] font-semibold uppercase tracking-wide text-amber-300">
                              Queued
                            </div>
                            <code className="font-mono text-[11px] text-gray-200 block truncate">
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
                className="inline-flex items-center gap-1.5 px-1.5 py-0.5 rounded hover:bg-white/10 text-green-400 font-semibold transition-colors cursor-pointer"
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
                className="inline-flex items-center gap-1.5 px-1.5 py-0.5 rounded hover:bg-white/10 transition-colors cursor-pointer"
              >
                <span className="w-2 h-2 rounded-full bg-yellow-500"></span>
                <span className="text-yellow-500">async</span>
              </button>
            ))}

          <div className="flex items-center gap-2">{getConnectionStatusIcon()}</div>
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
