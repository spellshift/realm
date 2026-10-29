import { useQuery } from "@apollo/client";
import { useEffect, useMemo, useState } from "react";
import {
    GET_BEACON_STATUS,
    GET_PORTAL_STATUS,
    GET_SHELL,
    GET_SHELL_ACTIVE_USERS,
    GET_SHELL_HISTORY,
    GET_SHELL_KNOWLEDGE_GRAPH,
    GET_SHELL_TOMES,
} from "../graphql";

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
    port?: number;
    closedAt?: string | null;
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
}

export interface KnowledgeShellItem {
    id: string;
    createdAt: string;
    closedAt?: string | null;
    beaconId: string;
    beaconName: string;
    beaconPrincipal?: string;
    ownerName?: string;
}

export interface KnowledgeHostItem {
    id: string;
    name: string;
    primaryIP?: string;
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
}

export const STUCK_TASK_THRESHOLD_MS = 15_000;

const getIpv4SubnetPrefix = (ip?: string | null): string | null => {
    if (!ip) return null;
    const parts = ip.trim().split(".");
    if (parts.length !== 4) return null;
    return `${parts[0]}.${parts[1]}.${parts[2]}.`;
};

export const useShellData = (shellId: string | undefined) => {
    const [portalId, setPortalId] = useState<number | null>(null);
    const [nowTick, setNowTick] = useState<number>(() => Date.now());

    // Fetch shell details first
    const { loading, error, data: shellData } = useQuery(GET_SHELL, {
        variables: { id: shellId },
        skip: !shellId,
        fetchPolicy: "network-only"
    });

    // Fetch shell history (tasks & pivots)
    const { data: historyData, refetch: refetchHistory } = useQuery(GET_SHELL_HISTORY, {
        variables: { id: shellId },
        skip: !shellId,
        fetchPolicy: "cache-and-network",
        pollInterval: 3000,
    });

    // Fetch beacon status
    const beaconId = shellData?.node?.beacon?.id;
    const { data: beaconData } = useQuery(GET_BEACON_STATUS, {
        variables: { id: beaconId },
        skip: !beaconId,
        pollInterval: 2000
    });

    // Fetch knowledge graph context for the current host & network with fast real-time polling
    const hostId = shellData?.node?.beacon?.host?.id;
    const [lastKnowledgeSyncAt, setLastKnowledgeSyncAt] = useState<number>(() => Date.now());
    const { data: knowledgeData, refetch: refetchKnowledgeGraph } = useQuery(GET_SHELL_KNOWLEDGE_GRAPH, {
        variables: { hostId },
        skip: !hostId,
        pollInterval: 3000,
        onCompleted: () => setLastKnowledgeSyncAt(Date.now()),
    });

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
    const { data: tomesData } = useQuery(GET_SHELL_TOMES, {
        skip: !shellId,
    });

    // Tick every second when there are running tasks so stuckTask elapsed time updates smoothly
    useEffect(() => {
        const timer = setInterval(() => {
            setNowTick(Date.now());
        }, 1000);
        return () => clearInterval(timer);
    }, []);

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
        const activePortal = portals.find((p: any) => !p.closedAt);
        if (activePortal) {
            const parsedId = parseInt(activePortal.id, 10);
            if (!Number.isNaN(parsedId)) {
                setPortalId(parsedId);
            }
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
        const shellEdges = historyData?.crossSessionShells?.edges || [];
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
    }, [shellTasks, historyData]);

    const totalTaskCount: number = historyData?.node?.shellTasks?.totalCount ?? shellTasks.length;

    // Immediately refresh Knowledge Graph whenever shell tasks report new credentials, files, or processes
    const reportedArtifactSignature = useMemo(() => {
        return shellTasks
            .map(
                (t) =>
                    `${t.id}:${t.execFinishedAt || ""}:${t.reportedCredentials?.length || 0}:${t.reportedFiles?.length || 0}:${t.reportedProcesses?.length || 0}`
            )
            .join("|");
    }, [shellTasks]);

    useEffect(() => {
        if (hostId && reportedArtifactSignature) {
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

    const pivots: ShellPivotItem[] = useMemo(() => {
        const edges = historyData?.node?.pivots?.edges || shellData?.node?.pivots?.edges || [];
        return edges.map((e: any) => e.node).filter(Boolean);
    }, [historyData, shellData]);

    const tomes: ShellTomeItem[] = useMemo(() => {
        const edges = tomesData?.tomes?.edges || [];
        return edges
            .map((e: any) => e.node)
            .filter((t: any) => t && t.eldritch);
    }, [tomesData]);

    // Knowledge Graph derived state
    const currentHostNode = knowledgeData?.currentHost || shellData?.node?.beacon?.host;
    const currentHostTags: string[] = useMemo(() => {
        const edges = shellData?.node?.beacon?.host?.tags?.edges || [];
        return edges.map((e: any) => e.node?.name).filter(Boolean);
    }, [shellData]);
    const currentSubnetPrefix = useMemo(
        () => getIpv4SubnetPrefix(shellData?.node?.beacon?.host?.primaryIP),
        [shellData]
    );

    const hostCredentials: KnowledgeCredentialItem[] = useMemo(() => {
        const edges = currentHostNode?.credentials?.edges || [];
        const hId = String(currentHostNode?.id || hostId || "");
        const hName = currentHostNode?.name || "Current Host";
        const hIp = currentHostNode?.primaryIP;
        return edges
            .map((e: any) => e.node)
            .filter(Boolean)
            .map((c: any) => ({
                id: String(c.id),
                principal: c.principal,
                kind: c.kind,
                secret: c.secret,
                createdAt: c.createdAt,
                hostId: hId,
                hostName: hName,
                hostPrimaryIP: hIp,
                isCurrentHost: true,
            }));
    }, [currentHostNode, hostId]);

    const networkHosts: KnowledgeHostItem[] = useMemo(() => {
        const edges = knowledgeData?.networkHosts?.edges || [];
        const currentIdStr = String(hostId || "");
        const items: KnowledgeHostItem[] = edges
            .map((e: any) => e.node)
            .filter(Boolean)
            .map((h: any) => {
                const hId = String(h.id);
                const isCurrentHost = hId === currentIdStr;
                const tags = (h.tags?.edges || []).map((te: any) => te.node).filter(Boolean);
                const sharedTags = tags
                    .map((t: any) => t.name)
                    .filter((name: string) => currentHostTags.includes(name));
                const subnet = getIpv4SubnetPrefix(h.primaryIP);
                const sameSubnet = !!(currentSubnetPrefix && subnet && currentSubnetPrefix === subnet);
                const creds: KnowledgeCredentialItem[] = (h.credentials?.edges || [])
                    .map((ce: any) => ce.node)
                    .filter(Boolean)
                    .map((c: any) => ({
                        id: String(c.id),
                        principal: c.principal,
                        kind: c.kind,
                        secret: c.secret,
                        createdAt: c.createdAt,
                        hostId: hId,
                        hostName: h.name,
                        hostPrimaryIP: h.primaryIP,
                        isCurrentHost,
                    }));

                return {
                    id: hId,
                    name: h.name,
                    primaryIP: h.primaryIP,
                    externalIP: h.externalIP,
                    platform: h.platform,
                    lastSeenAt: h.lastSeenAt,
                    tags,
                    beaconCount: h.beacons?.totalCount ?? 0,
                    credentialCount: h.credentials?.totalCount ?? creds.length,
                    credentials: creds,
                    isCurrentHost,
                    sharedTags,
                    sameSubnet,
                };
            });

        // Sort: non-current hosts with shared tags or same subnet first, then credential count, then name
        return items.sort((a, b) => {
            if (a.isCurrentHost !== b.isCurrentHost) return a.isCurrentHost ? 1 : -1;
            const aRel = (a.sharedTags.length > 0 ? 2 : 0) + (a.sameSubnet ? 1 : 0);
            const bRel = (b.sharedTags.length > 0 ? 2 : 0) + (b.sameSubnet ? 1 : 0);
            if (aRel !== bRel) return bRel - aRel;
            if (a.credentialCount !== b.credentialCount) return b.credentialCount - a.credentialCount;
            return a.name.localeCompare(b.name);
        });
    }, [knowledgeData, hostId, currentHostTags, currentSubnetPrefix]);

    const allCredentials: KnowledgeCredentialItem[] = useMemo(() => {
        const seen = new Set<string>();
        const list: KnowledgeCredentialItem[] = [];
        for (const cred of hostCredentials) {
            seen.add(cred.id);
            list.push(cred);
        }
        for (const h of networkHosts) {
            for (const cred of h.credentials) {
                if (!seen.has(cred.id)) {
                    seen.add(cred.id);
                    list.push(cred);
                }
            }
        }
        return list;
    }, [hostCredentials, networkHosts]);

    const hostFiles: KnowledgeFileItem[] = useMemo(() => {
        const edges = currentHostNode?.files?.edges || [];
        return edges.map((e: any) => e.node).filter(Boolean);
    }, [currentHostNode]);

    const hostProcessCount: number = currentHostNode?.processes?.totalCount ?? 0;

    const hostShells: KnowledgeShellItem[] = useMemo(() => {
        const beaconEdges = currentHostNode?.beacons?.edges || [];
        const shells: KnowledgeShellItem[] = [];
        beaconEdges.forEach((be: any) => {
            const b = be.node;
            if (!b) return;
            const sEdges = b.shells?.edges || [];
            sEdges.forEach((se: any) => {
                const s = se.node;
                if (!s) return;
                shells.push({
                    id: String(s.id),
                    createdAt: s.createdAt,
                    closedAt: s.closedAt,
                    beaconId: String(b.id),
                    beaconName: b.name,
                    beaconPrincipal: b.principal,
                    ownerName: s.owner?.name,
                });
            });
        });
        return shells.sort(
            (a, b) => new Date(b.createdAt).getTime() - new Date(a.createdAt).getTime()
        );
    }, [currentHostNode]);

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
        refetchHistory,
        hostCredentials,
        allCredentials,
        networkHosts,
        hostFiles,
        hostProcessCount,
        hostShells,
        lastKnowledgeSyncAt,
        refetchKnowledgeGraph,
    };
};
