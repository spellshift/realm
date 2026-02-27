import React from "react";
import { Info, Wifi, WifiOff, RefreshCw } from "lucide-react";
import { Tooltip } from "@chakra-ui/react";
import { ConnectionStatus } from "../../../lib/headless-adapter";

interface ShellStatusBarProps {
  portalId: number | null;
  timeUntilCallback: string;
  isMissedCallback: boolean;
  connectionStatus: ConnectionStatus;
}

const ShellStatusBar: React.FC<ShellStatusBarProps> = ({ portalId, timeUntilCallback, isMissedCallback, connectionStatus }) => {
  const getConnectionStatusIndicator = () => {
    switch (connectionStatus) {
      case "connected":
        return (
          <Tooltip label="Connected to Shell">
             <div className="flex items-center gap-1 text-green-500">
               <Wifi size={14} />
               <span className="text-xs">Connected</span>
             </div>
          </Tooltip>
        );
      case "reconnecting":
        return (
          <Tooltip label="Connection lost. Reconnecting...">
             <div className="flex items-center gap-1 text-yellow-500">
               <RefreshCw size={14} className="animate-spin" />
               <span className="text-xs">Reconnecting...</span>
             </div>
          </Tooltip>
        );
      case "disconnected":
      default:
        return (
          <Tooltip label="Disconnected from Shell">
             <div className="flex items-center gap-1 text-red-500">
               <WifiOff size={14} />
               <span className="text-xs">Disconnected</span>
             </div>
          </Tooltip>
        );
    }
  };

  return (
    <div className="flex justify-between items-center mt-2 text-sm text-gray-400 h-6">
      <div className="flex items-center gap-4">
        {getConnectionStatusIndicator()}

        <div className="h-4 w-px bg-gray-700"></div>

        {portalId ? (
          <span className="text-green-500 font-semibold">Portal Active (ID: {portalId})</span>
        ) : (
          <div className="flex items-center gap-1 group relative cursor-help">
            <span>non-interactive</span>
            <Tooltip label="This shell is currently in non-interactive mode. Input will be asynchronously queued for the beacon and output will be submitted through beacon callbacks. To upgrade to an interactive low-latency shell, you may open a 'Portal' on the beacon, which leverages an established connection to provide low-latency interactivity.">
              <span><Info size={14} /></span>
            </Tooltip>
          </div>
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
