export interface HostByTagCardProps {
  tagName: string;
  tagKind: "group" | "service";
}

export interface HostBeaconCounts {
  id: string;
  name: string | null;
  primaryIP: string | null;
  platform: string;
  onlineBeacons: {
    totalCount: number;
  };
  recentlyLostBeacons: {
    totalCount: number;
  };
  allBeacons: {
    totalCount: number;
  };
}

export interface GetHostsByTagResponse {
  hosts: {
    totalCount: number;
    edges: Array<{
      node: HostBeaconCounts;
    }>;
  };
}

export interface UseHostByTagDataResult {
  hosts: HostBeaconCounts[];
  totalOnlineHosts: number;
  totalOnlineBeacons: number;
  loading: boolean;
  error?: Error;
}

export interface TagNode {
  id: string;
  name: string;
  kind: string;
}

export interface GetTagsForDashboardResponse {
  tags: {
    totalCount: number;
    edges: Array<{
      node: TagNode;
    }>;
  };
}
