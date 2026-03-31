import { FC } from "react";
import { HostItem } from "./HostItem";
import { HostByTagCardProps } from "./types";
import { EmptyState, EmptyStateType } from "../../../components/tavern-base-ui/EmptyState";
import { useHostByTagData } from "./useHostByTagData";

export const HostByTagCard: FC<HostByTagCardProps> = ({ tagName, tagKind }) => {
  const { hosts, totalOnlineHosts, totalOnlineBeacons, loading, error } = useHostByTagData({
    tagName,
    tagKind,
  });

  // Filter to only show hosts with recently lost beacons or offline hosts
  const filteredHosts = hosts.filter(
    (host) => host.recentlyLostBeacons.totalCount > 0 || host.onlineBeacons.totalCount === 0
  );

  if (loading && hosts.length === 0) {
    return (
      <div className="bg-white rounded-lg border border-gray-200 p-4">
        <EmptyState type={EmptyStateType.loading} label="Loading hosts..." />
      </div>
    );
  }

  if (error) {
    return (
      <div className="bg-white rounded-lg border border-gray-200 p-4">
        <EmptyState
          type={EmptyStateType.error}
          label="Failed to load hosts"
          details={error.message}
        />
      </div>
    );
  }

  return (
    <div className="bg-white rounded-lg border border-gray-200 p-4 flex flex-col gap-4">
      <div className="flex flex-col gap-1">
        <h1 className="text-lg font-semibold text-gray-900">{tagName}</h1>
        <div className="flex flex-row gap-1 text-sm">
          <h2>{totalOnlineBeacons} active beacons</h2>
          <span>•</span>
          <h2>{totalOnlineHosts} hosts</h2>
        </div>
      </div>
      <div className="flex flex-row flex-wrap gap-1">
        {filteredHosts.map((host) => (
          <HostItem key={host.id} host={host} />
        ))}
      </div>
    </div>
  );
};

export default HostByTagCard;
