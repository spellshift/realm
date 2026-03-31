import { UseHostByTagDataResult, HostBeaconCounts } from "./types";

const createHost = (
  id: string,
  name: string,
  primaryIP: string,
  platform: string,
  onlineCount: number,
  recentlyLostCount: number,
  allCount: number
): HostBeaconCounts => ({
  id,
  name,
  primaryIP,
  platform,
  onlineBeacons: { totalCount: onlineCount },
  recentlyLostBeacons: { totalCount: recentlyLostCount },
  allBeacons: { totalCount: allCount },
});

const testHosts: HostBeaconCounts[] = [
  // 1. Fully offline hosts (0 online beacons) - 2 hosts
  createHost(
    "19",
    "legacy-server-01",
    "192.168.1.100",
    "PLATFORM_WINDOWS",
    0,
    2,
    2
  ),
  createHost(
    "20",
    "legacy-server-02",
    "192.168.1.101",
    "PLATFORM_WINDOWS",
    0,
    2,
    2
  ),

  // 2. Degraded hosts (online but with recently lost beacons) - 2 hosts
  createHost(
    "13",
    "load-balancer-01",
    "192.168.1.70",
    "PLATFORM_LINUX",
    3,
    2,
    5
  ),
  createHost(
    "14",
    "load-balancer-02",
    "192.168.1.71",
    "PLATFORM_LINUX",
    3,
    2,
    5
  ),

  // 3. Fully healthy online hosts (no recently lost beacons) - 16 hosts
  createHost(
    "1",
    "web-server-01",
    "192.168.1.10",
    "PLATFORM_LINUX",
    5,
    0,
    5
  ),
  createHost(
    "2",
    "web-server-02",
    "192.168.1.11",
    "PLATFORM_LINUX",
    4,
    0,
    4
  ),
  createHost(
    "3",
    "db-server-01",
    "192.168.1.20",
    "PLATFORM_LINUX",
    3,
    0,
    3
  ),
  createHost(
    "4",
    "db-server-02",
    "192.168.1.21",
    "PLATFORM_LINUX",
    3,
    0,
    3
  ),
  createHost(
    "5",
    "app-server-01",
    "192.168.1.30",
    "PLATFORM_LINUX",
    6,
    0,
    6
  ),
  createHost(
    "6",
    "app-server-02",
    "192.168.1.31",
    "PLATFORM_LINUX",
    5,
    0,
    5
  ),
  createHost(
    "7",
    "cache-server-01",
    "192.168.1.40",
    "PLATFORM_LINUX",
    4,
    0,
    4
  ),
  createHost(
    "8",
    "cache-server-02",
    "192.168.1.41",
    "PLATFORM_LINUX",
    4,
    0,
    4
  ),
  createHost(
    "9",
    "api-gateway-01",
    "192.168.1.50",
    "PLATFORM_LINUX",
    7,
    0,
    7
  ),
  createHost(
    "10",
    "api-gateway-02",
    "192.168.1.51",
    "PLATFORM_LINUX",
    6,
    0,
    6
  ),
  createHost(
    "11",
    "worker-node-01",
    "192.168.1.60",
    "PLATFORM_LINUX",
    5,
    0,
    5
  ),
  createHost(
    "12",
    "worker-node-02",
    "192.168.1.61",
    "PLATFORM_LINUX",
    5,
    0,
    5
  ),
  createHost(
    "15",
    "proxy-server-01",
    "192.168.1.80",
    "PLATFORM_LINUX",
    4,
    0,
    4
  ),
  createHost(
    "16",
    "proxy-server-02",
    "192.168.1.81",
    "PLATFORM_LINUX",
    4,
    0,
    4
  ),
  createHost(
    "17",
    "mail-server-01",
    "192.168.1.90",
    "PLATFORM_LINUX",
    5,
    0,
    5
  ),
  createHost(
    "18",
    "mail-server-02",
    "192.168.1.91",
    "PLATFORM_LINUX",
    5,
    0,
    5
  ),
];

export const testData: UseHostByTagDataResult = {
  hosts: testHosts,
  totalOnlineHosts: testHosts.filter((host) => host.onlineBeacons.totalCount > 0).length,
  totalOnlineBeacons: testHosts.reduce((sum, host) => sum + host.onlineBeacons.totalCount, 0),
  loading: false,
  error: undefined,
};

// Alternative test data scenarios
export const testDataAllOnline: UseHostByTagDataResult = {
  hosts: [
    createHost("1", "server-01", "10.0.0.1", "PLATFORM_LINUX", 5, 0, 5),
    createHost("2", "server-02", "10.0.0.2", "PLATFORM_LINUX", 3, 0, 3),
    createHost("3", "server-03", "10.0.0.3", "PLATFORM_WINDOWS", 4, 0, 4),
  ],
  totalOnlineHosts: 3,
  totalOnlineBeacons: 12,
  loading: false,
  error: undefined,
};

export const testDataAllOffline: UseHostByTagDataResult = {
  hosts: [
    createHost("1", "dead-server-01", "10.0.1.1", "PLATFORM_LINUX", 0, 0, 3),
    createHost("2", "dead-server-02", "10.0.1.2", "PLATFORM_LINUX", 0, 0, 2),
    createHost("3", "dead-server-03", "10.0.1.3", "PLATFORM_WINDOWS", 0, 0, 4),
  ],
  totalOnlineHosts: 0,
  totalOnlineBeacons: 0,
  loading: false,
  error: undefined,
};

export const testDataRecentlyLost: UseHostByTagDataResult = {
  hosts: [
    createHost("1", "flaky-server-01", "10.0.2.1", "PLATFORM_LINUX", 0, 3, 3),
    createHost("2", "flaky-server-02", "10.0.2.2", "PLATFORM_LINUX", 0, 2, 4),
    createHost("3", "flaky-server-03", "10.0.2.3", "PLATFORM_WINDOWS", 0, 4, 5),
  ],
  totalOnlineHosts: 0,
  totalOnlineBeacons: 0,
  loading: false,
  error: undefined,
};

export const testDataMixed: UseHostByTagDataResult = {
  hosts: [
    createHost("1", "healthy-server", "10.0.3.1", "PLATFORM_LINUX", 5, 0, 5),
    createHost("2", "degraded-server", "10.0.3.2", "PLATFORM_LINUX", 2, 3, 5),
    createHost("3", "failing-server", "10.0.3.3", "PLATFORM_WINDOWS", 0, 4, 4),
    createHost("4", "dead-server", "10.0.3.4", "PLATFORM_WINDOWS", 0, 0, 2),
  ],
  totalOnlineHosts: 2,
  totalOnlineBeacons: 7,
  loading: false,
  error: undefined,
};

export const testDataLoading: UseHostByTagDataResult = {
  hosts: [],
  totalOnlineHosts: 0,
  totalOnlineBeacons: 0,
  loading: true,
  error: undefined,
};

export const testDataError: UseHostByTagDataResult = {
  hosts: [],
  totalOnlineHosts: 0,
  totalOnlineBeacons: 0,
  loading: false,
  error: new Error("Failed to fetch hosts data"),
};
