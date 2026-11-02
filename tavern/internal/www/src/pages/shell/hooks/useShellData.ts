import { useQuery } from "@apollo/client";
import { useEffect, useMemo, useRef, useState } from "react";
import {
    GET_BEACON_STATUS,
    GET_PORTAL_STATUS,
    GET_SHELL,
    GET_SHELL_ACTIVE_USERS,
    GET_SHELL_CROSS_SESSION_HISTORY,
    GET_SHELL_HISTORY,
    GET_SHELL_KNOWLEDGE_GRAPH,
    GET_SHELL_TOMES,
} from "../graphql";
import { buildUnifiedGraphData } from "../utils/networkGraphUtils";

export interface ShellTaskItem {
    id: string;
    shellId?: string;
    beaconName?: string;
    hostName?: string;
    createdAt: string;
    claimedAt?: string | null;
    execStartedAt?: string | null;
    execFinishedAt?: string | null;
    input: string;
    output?: string | null;
    error?: string | null;
    creator?: {
        id: string;
        name: string;
        photoURL?: string;
    };
    reportedCredentials?: {
        id: string;
        principal: string;
        kind: string;
    }[];
    reportedFiles?: {
        id: string;
        path: string;
        size: number;
    }[];
    reportedProcesses?: {
        id: string;
    }[];
}

export interface ShellPivotItem {
    id: string;
    createdAt?: string;
    kind: string;
    destination: string;
    activeTarget?: string;
    customTitle?: string;
    port?: number;
    closedAt?: string | null;
    shellId?: string;
    hostId?: string;
    hostName?: string;
    isCurrentHost?: boolean;
}

export interface ShellTomeItem {
    id: string;
    name: string;
    description: string;
    tactic?: string;
    eldritch: string;
    paramDefs?: string;
    supportModel?: string;
}

export interface KnowledgeCredentialItem {
    id: string;
    principal: string;
    kind: string;
    secret: string;
    createdAt: string;
    hostId: string;
    hostName: string;
    hostPrimaryIP?: string;
    isCurrentHost: boolean;
}

export interface KnowledgeFileItem {
    id: string;
    path: string;
    size: number;
    owner?: string;
    permissions?: string;
    createdAt: string;
    hostId?: string;
    hostName?: string;
    isCurrentHost?: boolean;
}

export interface KnowledgeShellItem {
    id: string;
    createdAt: string;
    closedAt?: string | null;
    beaconId: string;
    beaconName: string;
    beaconPrincipal?: string;
    ownerName?: string;
    hostId?: string;
    hostName?: string;
    isCurrentHost?: boolean;
}

export interface KnowledgeHostItem {
    id: string;
    hostId?: string;
    networkHostId?: string;
    name: string;
    hostname?: string;
    primaryIP?: string;
    ipAddress?: string;
    subnet?: string;
    openPorts?: number[];
    networks?: { id: string; name: string; subnet: string; description?: string }[];
    externalIP?: string;
    platform?: string;
    lastSeenAt?: string;
    tags: { id: string; name: string; kind: string }[];
    beaconCount: number;
    credentialCount: number;
    credentials: KnowledgeCredentialItem[];
    isCurrentHost: boolean;
    sharedTags: string[];
    sameSubnet: boolean;
    isReachable?: boolean;
}

export const STUCK_TASK_THRESHOLD_MS = 15_000;

export const MAX_SHELL_LOAD_RETRIES = 5;

export interface UseShellDataOptions {
    historyOpen?: boolean;
    knowledgeOpen?: boolean;
}

export const useShellData = (
    shellId: string | undefined,
    options?: UseShellDataOptions
) => {
    const historyOpen = Boolean(options?.historyOpen);
    const knowledgeOpen = Boolean(options?.knowledgeOpen);
    const [portalId, setPortalId] = useState<number | null>(null);
    const [nowTick, setNowTick] = useState<number>(() => Date.now());
    const [shellRetryCount, setShellRetryCount] = useState<number>(0);

    // Fetch shell details first
    const {
        loading: rawLoading,
        error: rawError,
        data: shellData,
        refetch: refetchShell,
    } = useQuery(GET_SHELL, {
        variables: { id: shellId },
        skip: !shellId,
        fetchPolicy: "network-only",
    });

    // Reset retry counter when shellId changes or shell loads successfully
    useEffect(() => {
        setShellRetryCount(0);
    }, [shellId, shellData?.node?.id]);

    // Automatically retry GET_SHELL on transient database lock / network errors
    const shouldRetryShellLoad = Boolean(
        shellId && !rawLoading && rawError && !shellData?.node?.id && shellRetryCount < MAX_SHELL_LOAD_RETRIES
    );
    useEffect(() => {
        if (!shouldRetryShellLoad) return;
        const delayMs = 250 * (shellRetryCount + 1);
        const timer = setTimeout(() => {
            setShellRetryCount((c) => c + 1);
            refetchShell().catch(() => {
                // Handled by subsequent retry or final error state
            });
        }, delayMs);
        return () => clearTimeout(timer);
    }, [shouldRetryShellLoad, shellRetryCount, refetchShell]);

    const loading = rawLoading || shouldRetryShellLoad;
    const error = shouldRetryShellLoad ? undefined : rawError;

    // Fetch shell history (current session tasks & pivots)
    const { data: historyData, refetch: refetchCurrentHistory } = useQuery(GET_SHELL_HISTORY, {
        variables: { id: shellId },
        skip: !shellId,
        fetchPolicy: "cache-and-network",
        pollInterval: 3000,
    });

    // Fetch cross-session shell history (fetched once on load, polled only while History drawer is open)
    const { data: crossSessionHistoryData, refetch: refetchCrossSessionHistory } = useQuery(
        GET_SHELL_CROSS_SESSION_HISTORY,
        {
            skip: !shellId,
            fetchPolicy: "cache-and-network",
            pollInterval: historyOpen ? 5000 : 0,
        }
    );

    const refetchHistory = useMemo(
        () => () => {
            refetchCurrentHistory();
            if (historyOpen) {
                refetchCrossSessionHistory();
            }
        },
        [refetchCurrentHistory, refetchCrossSessionHistory, historyOpen]
    );

    useEffect(() => {
        if (historyOpen && shellId) {
            refetchCrossSessionHistory();
        }
    }, [historyOpen, shellId, refetchCrossSessionHistory]);

    // Fetch beacon status
    const beaconId = shellData?.node?.beacon?.id;
    const { data: beaconData } = useQuery(GET_BEACON_STATUS, {
        variables: { id: beaconId },
        skip: !beaconId,
        pollInterval: 2000
    });

    // Fetch knowledge graph context for the current host & network (fetched on load & artifact updates, polled continuously)
    const hostId = shellData?.node?.beacon?.host?.id;
    const [lastKnowledgeSyncAt, setLastKnowledgeSyncAt] = useState<number>(() => Date.now());
    const { data: knowledgeData, refetch: refetchKnowledgeGraph } = useQuery(GET_SHELL_KNOWLEDGE_GRAPH, {
        variables: { hostId },
        skip: !hostId,
        fetchPolicy: "cache-and-network",
        pollInterval: 3000,
    });

    useEffect(() => {
        if (knowledgeData) {
            setLastKnowledgeSyncAt(Date.now());
        }
    }, [knowledgeData]);

    useEffect(() => {
        if (knowledgeOpen && hostId) {
            refetchKnowledgeGraph();
        }
    }, [knowledgeOpen, hostId, refetchKnowledgeGraph]);

    // Fetch portal status
    const { data: portalData } = useQuery(GET_PORTAL_STATUS, {
        variables: { id: portalId },
        skip: !portalId,
        pollInterval: 2000
    });

    // Fetch active users
    const { data: activeUsersData } = useQuery(GET_SHELL_ACTIVE_USERS, {
        variables: { id: shellId },
        skip: !shellId,
        pollInterval: 10000
    });

    // Fetch Eldritch tomes for the Composer
    const { data: tomesData, refetch: refetchTomes } = useQuery(GET_SHELL_TOMES, {
        skip: !shellId,
    });

    // Reset portalId when switching shells
    useEffect(() => {
        setPortalId(null);
    }, [shellId]);

    // Initialize portalId from shell data or history data without clobbering
    // a live WebSocket PORTAL_UPGRADE when shell.portals is empty or not yet synced
    useEffect(() => {
        const shellPortals = shellData?.node?.portals || [];
        const historyPortals = historyData?.node?.portals || [];
        const portals = historyPortals.length > 0 ? historyPortals : shellPortals;
        if (!Array.isArray(portals) || portals.length === 0) {
            return;
        }
        const openPortalIds = portals
            .filter((p: any) => p && !p.closedAt)
            .map((p: any) => parseInt(p.id, 10))
            .filter((id: number) => !Number.isNaN(id));
        if (openPortalIds.length > 0) {
            const newestPortalId = Math.max(...openPortalIds);
            setPortalId(newestPortalId);
        } else {
            setPortalId((prev) => {
                if (
                    prev !== null &&
                    portals.some((p: any) => parseInt(p.id, 10) === prev && Boolean(p.closedAt))
                ) {
                    return null;
                }
                return prev;
            });
        }
    }, [shellData, historyData]);

    // Check if portal closed
    useEffect(() => {
        if (portalData?.node?.closedAt) {
            setPortalId(null);
        }
    }, [portalData]);

    const activeUsers = activeUsersData?.node?.activeUsers?.edges?.map((edge: any) => edge.node) || [];

    const shellTasks: ShellTaskItem[] = useMemo(() => {
        const edges = historyData?.node?.shellTasks?.edges || [];
        const currentShellIdStr = String(historyData?.node?.id || shellId || "");
        const currentBeaconName = shellData?.node?.beacon?.name;
        const currentHostName = shellData?.node?.beacon?.host?.name;
        return edges
            .map((e: any) => e.node)
            .filter(Boolean)
            .map((t: any) => ({
                ...t,
                shellId: t.shellId || currentShellIdStr,
                beaconName: t.beaconName || currentBeaconName,
                hostName: t.hostName || currentHostName,
            }));
    }, [historyData, shellId, shellData]);

    const allShellTasks: ShellTaskItem[] = useMemo(() => {
        const byId = new Map<string, ShellTaskItem>();
        for (const t of shellTasks) {
            byId.set(String(t.id), t);
        }
        const shellEdges = crossSessionHistoryData?.crossSessionShells?.edges || [];
        for (const se of shellEdges) {
            const sNode = se?.node;
            if (!sNode) continue;
            const sId = String(sNode.id || "");
            const bName = sNode.beacon?.name;
            const hName = sNode.beacon?.host?.name;
            const tEdges = sNode.shellTasks?.edges || [];
            for (const te of tEdges) {
                const tNode = te?.node;
                if (!tNode?.id) continue;
                const key = String(tNode.id);
                if (!byId.has(key)) {
                    byId.set(key, {
                        ...tNode,
                        shellId: sId,
                        beaconName: bName,
                        hostName: hName,
                    });
                }
            }
        }
        return Array.from(byId.values()).sort(
            (a, b) => new Date(a.createdAt || 0).getTime() - new Date(b.createdAt || 0).getTime()
        );
    }, [shellTasks, crossSessionHistoryData]);

    const totalTaskCount: number = historyData?.node?.shellTasks?.totalCount ?? shellTasks.length;

    // Immediately refresh Knowledge Graph whenever shell tasks report new credentials, files, or processes
    // (skipping the initial hostId load since useQuery(GET_SHELL_KNOWLEDGE_GRAPH) already executes on mount)
    const reportedArtifactSignature = useMemo(() => {
        return shellTasks
            .map(
                (t) =>
                    `${t.id}:${t.execFinishedAt || ""}:${t.reportedCredentials?.length || 0}:${t.reportedFiles?.length || 0}:${t.reportedProcesses?.length || 0}`
            )
            .join("|");
    }, [shellTasks]);

    const prevArtifactSyncRef = useRef<{ hostId?: string; sig: string }>({
        hostId: undefined,
        sig: "",
    });
    useEffect(() => {
        if (!hostId) return;
        const hostIdStr = String(hostId);
        const prev = prevArtifactSyncRef.current;
        prevArtifactSyncRef.current = { hostId: hostIdStr, sig: reportedArtifactSignature };
        if (prev.hostId === hostIdStr && prev.sig !== reportedArtifactSignature && reportedArtifactSignature) {
            refetchKnowledgeGraph();
            setLastKnowledgeSyncAt(Date.now());
        }
    }, [hostId, reportedArtifactSignature, refetchKnowledgeGraph]);

    const queuedTasks = useMemo(
        () =>
            shellTasks.filter(
                (t) => !t.claimedAt && !t.execFinishedAt && !t.output && !t.error
            ),
        [shellTasks]
    );

    const runningTasks = useMemo(
        () =>
            shellTasks.filter(
                (t) => !!t.claimedAt && !t.execFinishedAt && !t.output && !t.error
            ),
        [shellTasks]
    );

    // Tick every second only when there are running tasks so stuckTask elapsed time updates smoothly
    const hasRunningTasks = runningTasks.length > 0;
    useEffect(() => {
        if (!hasRunningTasks) return;
        setNowTick(Date.now());
        const timer = setInterval(() => {
            setNowTick(Date.now());
        }, 1000);
        return () => clearInterval(timer);
    }, [hasRunningTasks]);

    const stuckTaskInfo = useMemo(() => {
        for (const task of runningTasks) {
            const startTimeStr = task.execStartedAt || task.claimedAt;
            if (!startTimeStr) continue;
            const startedMs = new Date(startTimeStr).getTime();
            if (!isNaN(startedMs) && nowTick - startedMs >= STUCK_TASK_THRESHOLD_MS) {
                return {
                    task,
                    elapsedSeconds: Math.max(1, Math.floor((nowTick - startedMs) / 1000)),
                };
            }
        }
        return null;
    }, [runningTasks, nowTick]);

    const stuckTask: ShellTaskItem | null = stuckTaskInfo?.task ?? null;
    const stuckElapsedSeconds: number = stuckTaskInfo?.elapsedSeconds ?? 0;

    const tomes: ShellTomeItem[] = useMemo(() => {
        const edges = tomesData?.tomes?.edges || [];
        return edges
            .map((e: any) => e.node)
            .filter((t: any) => t && t.eldritch);
    }, [tomesData]);

    // Knowledge Graph & Pivots unified derived state
    const currentHostNode = knowledgeData?.currentHost || shellData?.node?.beacon?.host;
    const currentHostTags: string[] = useMemo(() => {
        const edges = shellData?.node?.beacon?.host?.tags?.edges || [];
        return edges.map((e: any) => e.node?.name).filter(Boolean);
    }, [shellData]);

    const currentShellPivots: ShellPivotItem[] = useMemo(() => {
        const edges = historyData?.node?.pivots?.edges || shellData?.node?.pivots?.edges || [];
        return edges.map((e: any) => e.node).filter(Boolean);
    }, [historyData, shellData]);

    const unifiedGraph = useMemo(
        () =>
            buildUnifiedGraphData({
                currentHostNode,
                hostId: hostId ? String(hostId) : undefined,
                currentShellId: shellId ? String(shellId) : undefined,
                currentHostTags,
                currentShellPivots,
                knowledgeData,
            }),
        [currentHostNode, hostId, shellId, currentHostTags, currentShellPivots, knowledgeData]
    );

    const pivots = unifiedGraph.nearbyPivots;
    const hostCredentials = unifiedGraph.hostCredentials;
    const allCredentials = unifiedGraph.allCredentials;
    const networkHosts = unifiedGraph.networkHosts;
    const hostFiles = unifiedGraph.hostFiles;
    const hostShells = unifiedGraph.hostShells;
    const hostProcessCount: number = currentHostNode?.processes?.totalCount ?? 0;

    return {
        loading,
        error,
        shellData,
        beaconData,
        portalData,
        portalId,
        setPortalId,
        activeUsers,
        shellTasks,
        allShellTasks,
        totalTaskCount,
        queuedTasks,
        runningTasks,
        stuckTask,
        stuckElapsedSeconds,
        pivots,
        tomes,
        refetchTomes,
        refetchHistory,
        hostCredentials,
        allCredentials,
        networkHosts,
        hostFiles,
        hostProcessCount,
        hostShells,
        lastKnowledgeSyncAt,
        refetchKnowledgeGraph,
        refetchShell,
    };
};
