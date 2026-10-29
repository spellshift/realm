import React, { useState } from "react";
import { ConnectionStatus } from "../../../lib/browser-adapter";
import { Wifi, WifiOff, RefreshCw, Clock, Loader2 } from "lucide-react";
import { ShellTaskItem } from "../hooks/useShellData";

interface ShellStatusBarProps {
  portalId: number | null;
  timeUntilCallback: string;
  isMissedCallback: boolean;
  connectionStatus: ConnectionStatus;
  connectionMessage?: string;
  closedAt?: string | null;
  queuedCount?: number;
  runningCount?: number;
  queuedTasks?: ShellTaskItem[];
  runningTasks?: ShellTaskItem[];
  queueTourOpen?: boolean;
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

const ShellStatusBar: React.FC<ShellStatusBarProps> = ({
  portalId,
  timeUntilCallback,
  isMissedCallback,
  connectionStatus,
  connectionMessage,
  closedAt,
  queuedCount,
  runningCount,
  queuedTasks = [],
  runningTasks = [],
  queueTourOpen = false,
  onOpenQueue,
  onReconnect,
}) => {
  const [queueHovered, setQueueHovered] = useState(false);
  const [queuePinned, setQueuePinned] = useState(false);

  const effectiveRunningCount = runningCount ?? runningTasks.length;
  const effectiveQueuedCount = queuedCount ?? queuedTasks.length;
  const hasActiveQueue = effectiveRunningCount > 0 || effectiveQueuedCount > 0;
  const showQueuePopover = (queueHovered || queuePinned) && hasActiveQueue;

  const getConnectionStatusIcon = () => {
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
    <div
      data-tour="shell-status-bar"
      className="bg-[#161922] px-3 py-1 border-t border-[#262b38] text-xs text-gray-400 flex justify-between items-center select-none"
    >
      <div className="flex items-center gap-4">
        {closedAt ? (
          <span className="flex items-center gap-2 text-red-400 font-semibold">
            <span className="w-2 h-2 rounded-full bg-red-500"></span>
            <span>
              Terminated:{" "}
              {new Date(closedAt).toLocaleString(undefined, {
                dateStyle: "medium",
                timeStyle: "short",
              })}
            </span>
          </span>
        ) : portalId ? (
          <span
            data-tour="shell-non-interactive"
            className="flex items-center gap-2 text-green-400 font-semibold"
          >
            <span className="w-2 h-2 rounded-full bg-green-400 animate-pulse"></span>
            <span>Portal Active</span>
            <span className="text-xs text-gray-500 font-normal">(ID: {portalId})</span>
          </span>
        ) : (
          <div className="flex items-center gap-2">
            <span
              data-tour="shell-non-interactive"
              className="inline-flex items-center gap-2"
            >
              <span className="w-2 h-2 rounded-full bg-yellow-500"></span>
              <span className="text-yellow-500">async</span>
            </span>
            {timeUntilCallback && (
              <>
                <span className="text-gray-600">|</span>
                <span
                  data-tour="shell-next-callback"
                  className="inline-flex items-center gap-2"
                >
                  <span>Next callback:</span>
                  <span className={isMissedCallback ? "text-red-500 font-bold" : "text-gray-300"}>
                    {timeUntilCallback}
                  </span>
                </span>
              </>
            )}
          </div>
        )}
      </div>

      {/* Right side: Queue Indicator Popover & WebSocket Status */}
      <div className="flex items-center gap-3">
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
        <div className="flex items-center gap-2">{getConnectionStatusIcon()}</div>
      </div>
    </div>
  );
};

export default ShellStatusBar;
