import { FC } from "react";
import { Bug } from "lucide-react";
import { HostBeaconCounts } from "./types";

interface HostItemProps {
  host: HostBeaconCounts;
}

export const HostItem: FC<HostItemProps> = ({ host }) => {
  const onlineCount = host.onlineBeacons.totalCount;
  const recentlyLostCount = host.recentlyLostBeacons.totalCount;
  const isHostLost = onlineCount === 0;
  console.log(host);

  if (isHostLost) {
    return (
      <div className="border-gray-200 border-2 rounded-full p-2 flex flex-row gap-1 items-center font-semibold">
        <Bug fill="currentColor" className="text-red-600 w-4 h-4" />
        <div className="text-sm text-red-600">Host lost</div>
      </div>
    );
  }

  return (
    <div className="border-gray-200 border-2 rounded-full p-2 flex flex-row gap-1 items-center font-semibold">
      <Bug fill="currentColor" className="text-purple-600 w-4 h-4" />
      <div className="text-sm text-purple-600">{onlineCount}</div>
      {recentlyLostCount > 0 && (
        <>
          <Bug fill="currentColor" className="text-red-600 w-4 h-4" />
          <div className="text-sm text-red-600">-{recentlyLostCount}</div>
        </>
      )}
    </div>
  );
};
