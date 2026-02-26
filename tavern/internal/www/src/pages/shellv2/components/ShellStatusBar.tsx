import React from "react";
import { Info } from "lucide-react";
import { Tooltip } from "@chakra-ui/react";

interface ShellStatusBarProps {
  portalId: number | null;
  timeUntilCallback: string;
  isMissedCallback: boolean;
}

const ShellStatusBar: React.FC<ShellStatusBarProps> = ({ portalId, timeUntilCallback, isMissedCallback }) => {
  return (
    <div className="flex justify-between items-center mt-2 text-sm text-gray-400 h-6">
      <div className="flex items-center gap-2">
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
