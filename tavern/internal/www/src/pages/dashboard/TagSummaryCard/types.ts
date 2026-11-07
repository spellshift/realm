export type TagKind = "group" | "service";

export interface TagNode {
  id: string;
  name: string;
  kind: string;
}

export interface HostWithTagStats {
  id: string;
  lastSeenAt: string | null;
  tags: {
    edges: Array<{ node: TagNode }>;
  };
  onlineBeacons: { totalCount: number };
  recentlyLostBeacons: { totalCount: number };
  allBeacons: { totalCount: number };
  beaconsWithQuests: {
    edges: Array<{
      node: {
        tasks: {
          edges: Array<{
            node: {
              quest: { id: string };
            };
          }>;
        };
      };
    }>;
  };
}

export interface GetAllHostsWithTagStatsResponse {
  hosts: {
    edges: Array<{ node: HostWithTagStats }>;
  };
}

export interface TagRow {
  tagId: string;
  tagName: string;
  tagKind: TagKind;
  onlineHosts: number;
  lostHosts: number;
  onlineBeacons: number;
  recentlyLostBeacons: number;
  lastCallbackAt: string | null;
  questCount: number;
}
