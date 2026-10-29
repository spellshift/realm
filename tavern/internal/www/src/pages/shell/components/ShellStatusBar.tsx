import React, { useState } from "react";
import { ConnectionStatus } from "../../../lib/browser-adapter";
import { Wifi, WifiOff, RefreshCw, Clock, Loader2 } from "lucide-react";

interface ShellStatusBarProps {
  portalId: number | null;
  timeUntilCallback: string;
  isMissedCallback: boolean;
  connectionStatus: ConnectionStatus;
  connectionMessage?: string;
  closedAt?: string | null;
  queuedCount?: number;
  runningCount?: number;
  onOpenQueue?: () => void;
  onReconnect?: () => void;
}

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
  queuedCount = 0,
  runningCount = 0,
  onOpenQueue,
  onReconnect,
}) => {
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
    <div className="bg-[#161922] px-3 py-1 border border-[#262b38] rounded-md text-xs text-gray-400 flex justify-between items-center font-mono select-none">
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
              <span className="text-yellow-500">non-interactive</span>
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

      {/* Right side: Queue Indicator & WebSocket Status */}
      <div className="flex items-center gap-3">
        {(runningCount > 0 || queuedCount > 0) && (
          <button
            type="button"
            data-testid="status-bar-queue-button"
            aria-label="Open task queue"
            onClick={onOpenQueue}
            title="Click to view active task queue in History"
            className="flex items-center gap-2 px-1.5 py-0.5 rounded hover:bg-white/10 transition-colors cursor-pointer"
          >
            {runningCount > 0 && (
              <span className="inline-flex items-center gap-1 text-blue-300">
                <Loader2 className="w-3 h-3 animate-spin" />
                {runningCount} executing
              </span>
            )}
            {queuedCount > 0 && (
              <span className="inline-flex items-center gap-1 text-amber-300">
                <Clock className="w-3 h-3" />
                {queuedCount} queued
              </span>
            )}
          </button>
        )}
        <div className="flex items-center gap-2">{getConnectionStatusIcon()}</div>
      </div>
    </div>
  );
};

export default ShellStatusBar;
