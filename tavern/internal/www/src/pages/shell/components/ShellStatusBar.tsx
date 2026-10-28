import React from "react";
import { Clock, Info, Loader2, RefreshCw, Wifi, WifiOff } from "lucide-react";
import { Tooltip } from "@chakra-ui/react";
import { ConnectionStatus } from "../../../lib/browser-adapter";

interface ShellStatusBarProps {
  portalId: number | null;
  timeUntilCallback: string;
  isMissedCallback: boolean;
  connectionStatus: ConnectionStatus;
  connectionMessage?: string;
  closedAt?: string;
  queuedCount?: number;
  runningCount?: number;
  onOpenHistory?: () => void;
}

const ShellStatusBar: React.FC<ShellStatusBarProps> = ({
  portalId,
  timeUntilCallback,
  isMissedCallback,
  connectionStatus,
  connectionMessage,
  closedAt,
  queuedCount = 0,
  runningCount = 0,
  onOpenHistory,
}) => {
  const getConnectionIcon = () => {
    switch (connectionStatus) {
      case "connected":
        return (
          <Tooltip label="Connected to Tavern">
            <span className="text-green-500 flex items-center gap-1.5">
              <Wifi size={16} />
              <span className="hidden sm:inline text-xs font-medium text-green-400">WS Connected</span>
            </span>
          </Tooltip>
        );
      case "disconnected": {
        const label = connectionMessage
          ? `Disconnected from Tavern: ${connectionMessage}`
          : "Disconnected from Tavern";
        return (
          <Tooltip label={label}>
            <span className="text-red-500 flex items-center gap-1.5">
              <WifiOff size={16} />
              <span className="hidden sm:inline text-xs font-medium text-red-400">
                WS Disconnected
              </span>
            </span>
          </Tooltip>
        );
      }
      case "reconnecting":
        return (
          <Tooltip label="Reconnecting to Tavern">
            <span className="text-yellow-500 animate-spin">
              <RefreshCw size={16} />
            </span>
          </Tooltip>
        );
    }
  };

  return (
    <div className="flex justify-between items-center mt-2 pt-1.5 border-t border-[#262b36] text-xs text-gray-400 min-h-[26px]">
      <div className="flex items-center gap-3 md:gap-4 flex-wrap">
        <div className="flex items-center">{getConnectionIcon()}</div>

        {closedAt && (
          <div className="flex items-center gap-1 px-2 py-0.5 rounded bg-red-950/60 border border-red-500/40">
            <span className="font-bold text-red-500 uppercase">Terminated</span>
            <Tooltip
              label={`This shell session was closed at ${new Date(closedAt).toLocaleString()}. Input is disabled.`}
            >
              <span>
                <Info size={13} className="text-red-500" />
              </span>
            </Tooltip>
          </div>
        )}

        <div className="flex items-center gap-2">
          {portalId ? (
            <div className="flex items-center gap-1 group relative cursor-help">
              <span
                className={`font-semibold ${
                  connectionStatus === "connected"
                    ? "text-green-500"
                    : connectionStatus === "reconnecting"
                    ? "text-yellow-500"
                    : "text-gray-500"
                }`}
              >
                Portal Active
                <span className="hidden md:inline"> (ID: {portalId})</span>
              </span>
              <Tooltip label="This shell is currently using an established portal connection for low-latency i/o. You may utilize this portal for SOCKS5 proxying or pivoting (e.g. with SSH)">
                <span>
                  <Info
                    size={13}
                    className={
                      connectionStatus === "connected"
                        ? "text-green-500"
                        : connectionStatus === "reconnecting"
                        ? "text-yellow-500"
                        : "text-gray-500"
                    }
                  />
                </span>
              </Tooltip>
            </div>
          ) : (
            <div className="flex items-center gap-1 group relative cursor-help">
              <span>non-interactive</span>
              <Tooltip label="This shell is currently in non-interactive mode. Input will be asynchronously queued for the beacon and output will be submitted through beacon callbacks. To upgrade to an interactive low-latency shell, you may open a 'Portal' on the beacon, which leverages an established connection to provide low-latency interactivity.">
                <span>
                  <Info size={13} />
                </span>
              </Tooltip>
            </div>
          )}
        </div>

        {(queuedCount > 0 || runningCount > 0) && (
          <button
            type="button"
            onClick={onOpenHistory}
            className="inline-flex items-center gap-2 px-2 py-0.5 rounded bg-[#1d222e] hover:bg-[#262d3d] border border-[#323a4f] text-gray-200 transition-colors"
          >
            {runningCount > 0 && (
              <span className="inline-flex items-center gap-1 text-amber-300">
                <Loader2 size={11} className="animate-spin" />
                {runningCount} executing
              </span>
            )}
            {queuedCount > 0 && (
              <span className="inline-flex items-center gap-1 text-blue-300">
                <Clock size={11} />
                {queuedCount} queued
              </span>
            )}
          </button>
        )}
      </div>

      {timeUntilCallback && (
        <div className={isMissedCallback ? "text-red-500 font-bold" : "text-gray-400"}>
          {timeUntilCallback}
        </div>
      )}
    </div>
  );
};

export default ShellStatusBar;
