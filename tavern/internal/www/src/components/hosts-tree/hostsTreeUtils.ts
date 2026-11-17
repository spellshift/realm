import { selectBestBeaconId } from "../create-shell-button/CreateShellButton";
import {
    KnowledgeBeaconItem,
    KnowledgeHostItem,
    KnowledgeShellItem,
    ShellPivotItem,
    ShellTaskItem,
} from "../../pages/shell/hooks/useShellData";
import {
    extractPivotTargetHost,
    formatPivotTabLabel,
    formatSubnetCidr,
    hasPivotActivity,
    stripAnsi,
} from "../../pages/shell/utils/networkGraphUtils";
import { EMPTY_HOSTS_FILTERS, HostsFilterState } from "./HostsFilterPopover";

export interface HostBeaconNode {
    id: string;
    name: string;
    principal: string;
    transport?: string;
    interval?: number | null;
    lastSeenAt?: string;
    nextSeenAt?: string;
    isOnline: boolean;
    activeShells: KnowledgeShellItem[];
    closedShells: KnowledgeShellItem[];
    activePivots: ShellPivotItem[];
    closedPivots: ShellPivotItem[];
    hasMatchingClosed?: boolean;
}

export interface HostTreeNode {
    key: string;
    hostId?: string;
    name: string;
    ip: string;
    subtitle: string;
    isCurrentHost: boolean;
    isOnline: boolean;
    isConnected: boolean;
    beacons: HostBeaconNode[];
    activeShells: KnowledgeShellItem[];
    activePivots: ShellPivotItem[];
    closedShells: KnowledgeShellItem[];
    closedPivots: ShellPivotItem[];
    outgoingActivePivots: ShellPivotItem[];
    outgoingClosedPivots: ShellPivotItem[];
    hasMatchingClosed?: boolean;
    sourceHost?: KnowledgeHostItem;
}

export interface HostsTreeSections {
    currentHostNode: HostTreeNode | null;
    activePivotHosts: HostTreeNode[];
    hiddenOtherHosts: HostTreeNode[];
    availablePivotHosts: HostTreeNode[];
    totalHostCount: number;
    visibleHostCount: number;
    activePivotCount: number;
}

export interface BuildHostsTreeParams {
    hosts: KnowledgeHostItem[];
    shells: KnowledgeShellItem[];
    pivots: ShellPivotItem[];
    currentHostBeacons?: KnowledgeBeaconItem[];
    currentHostId?: string;
    currentHostName?: string;
    currentHostIp?: string;
    currentShellId?: string;
}

export const isShellLive = (shell: KnowledgeShellItem): boolean => {
    if (shell.closedAt) {
        return false;
    }
    return shell.isLive ?? (shell.beaconOnline !== undefined ? shell.beaconOnline : true);
};

export const pickBestBeaconIdForBeacons = (beacons: HostBeaconNode[]): string | undefined => {
    if (beacons.length === 0) {
        return undefined;
    }
    const onlineBeacons = beacons.filter((beacon) => beacon.isOnline);
    const pool = onlineBeacons.length > 0 ? onlineBeacons : [];
    if (pool.length === 0) {
        return undefined;
    }
    const bestId = selectBestBeaconId(
        pool.map((beacon) => ({
            id: beacon.id,
            principal: beacon.principal || "",
            interval:
                beacon.interval !== undefined && beacon.interval !== null
                    ? Number(beacon.interval)
                    : Number.MAX_SAFE_INTEGER,
            lastSeenAt: beacon.lastSeenAt || new Date(0).toISOString(),
            nextSeenAt: beacon.nextSeenAt,
            transport: beacon.transport,
        }))
    );
    return bestId ? String(bestId) : undefined;
};

export const dedupePivotsById = (items: ShellPivotItem[]): ShellPivotItem[] => {
    const seen = new Set<string>();
    const result: ShellPivotItem[] = [];
    for (const item of items) {
        const key = String(item.id);
        if (seen.has(key)) {
            continue;
        }
        seen.add(key);
        result.push(item);
    }
    return result;
};

export const groupPivotsByShellId = (
    shells: KnowledgeShellItem[],
    pivots: ShellPivotItem[]
): {
    pivotsByShellId: Map<string, ShellPivotItem[]>;
    unmatchedActivePivots: ShellPivotItem[];
} => {
    const activeShellIds = new Set(shells.map((s) => String(s.id)));
    const pivotsByShellId = new Map<string, ShellPivotItem[]>();
    const unmatchedActivePivots: ShellPivotItem[] = [];

    for (const pivot of pivots) {
        const sid = pivot.shellId ? String(pivot.shellId) : "";
        if (!sid || !activeShellIds.has(sid)) {
            unmatchedActivePivots.push(pivot);
            continue;
        }
        const list = pivotsByShellId.get(sid) || [];
        list.push(pivot);
        pivotsByShellId.set(sid, list);
    }

    return { pivotsByShellId, unmatchedActivePivots };
};

const isUserCurrentOperator = (
    user: { id?: string; name?: string },
    currentUserId?: string,
    currentUserName?: string
): boolean => {
    if (currentUserId && user.id) {
        return String(user.id) === String(currentUserId);
    }
    if (currentUserName && user.name) {
        return user.name.trim().toLowerCase() === currentUserName.trim().toLowerCase();
    }
    return false;
};

export const isShellOwnedByOperator = (
    shell: KnowledgeShellItem,
    currentUserId?: string,
    currentUserName?: string
): boolean =>
    isUserCurrentOperator(
        { id: shell.ownerId, name: shell.ownerName },
        currentUserId,
        currentUserName
    );

export const pickPreferredShell = (
    candidateShells: KnowledgeShellItem[],
    currentUserId?: string,
    currentUserName?: string
): KnowledgeShellItem | undefined => {
    if (candidateShells.length === 0) {
        return undefined;
    }
    if (currentUserId || currentUserName) {
        const owned = candidateShells.find((shell) =>
            isShellOwnedByOperator(shell, currentUserId, currentUserName)
        );
        if (owned) {
            return owned;
        }
    }
    return candidateShells[0];
};

export const hasOtherActiveUsers = (
    node: HostTreeNode,
    currentUserId?: string,
    currentUserName?: string
): boolean => {
    const hasIdentity = Boolean(currentUserId || currentUserName);
    return node.activeShells.some((shell) =>
        (shell.activeUsers || []).some((user) =>
            hasIdentity ? !isUserCurrentOperator(user, currentUserId, currentUserName) : true
        )
    );
};

export const isPrivilegedPrincipal = (principal?: string | null): boolean => {
    const leaf = String(principal || "")
        .trim()
        .split(/[\\/]/)
        .pop()
        ?.toLowerCase();
    return leaf === "root" || leaf === "system" || leaf === "administrator";
};

export const hasPrivilegedHostAccess = (node: HostTreeNode): boolean =>
    node.activeShells.some((shell) => isPrivilegedPrincipal(shell.beaconPrincipal));

export const compareHostTreeNodes = (
    a: HostTreeNode,
    b: HostTreeNode,
    currentUserId?: string,
    currentUserName?: string
): number => {
    const getPriority = (node: HostTreeNode): number => {
        if (node.activePivots.length > 0) {
            return 0;
        }
        if (hasOtherActiveUsers(node, currentUserId, currentUserName)) {
            return 1;
        }
        if (
            node.activeShells.length > 0 ||
            Boolean(node.sourceHost?.hasOnlineBeacon) ||
            node.beacons.some((beacon) => beacon.isOnline) ||
            node.isOnline
        ) {
            return 2;
        }
        return 3;
    };
    const priorityDiff = getPriority(a) - getPriority(b);
    if (priorityDiff !== 0) {
        return priorityDiff;
    }
    return a.name.localeCompare(b.name);
};

export const buildHostsTreeSections = ({
    hosts,
    shells,
    pivots,
    currentHostBeacons = [],
    currentHostId,
    currentHostName,
    currentHostIp,
    currentShellId,
}: BuildHostsTreeParams): HostsTreeSections => {
    const activeShells = shells.filter(isShellLive);
    const closedShells = shells.filter((s) => !isShellLive(s));
    const openPivots = pivots.filter((p) => !p.closedAt && hasPivotActivity(p));
    const closedPivots = pivots.filter((p) => Boolean(p.closedAt) && hasPivotActivity(p));

    const nodeByKey = new Map<string, HostTreeNode>();
    const keyByHostId = new Map<string, string>();
    const keyByIp = new Map<string, string>();
    const keyByName = new Map<string, string>();
    const keyByShortName = new Map<string, string>();

    const extractShortHostname = (value?: string | null): string => {
        const lower = String(value || "").trim().toLowerCase();
        if (!lower || !lower.includes(".") || /^\d{1,3}(\.\d{1,3}){3}$/.test(lower)) {
            return "";
        }
        return lower.split(".")[0];
    };

    const indexNodeAliases = (
        key: string,
        base: {
            hostId?: string;
            name?: string;
            ip?: string;
            sourceHost?: KnowledgeHostItem;
        }
    ) => {
        const hostId = base.hostId || base.sourceHost?.hostId;
        if (hostId && !keyByHostId.has(String(hostId))) {
            keyByHostId.set(String(hostId), key);
        }
        const ipCandidates = [
            base.ip,
            base.sourceHost?.primaryIP,
            base.sourceHost?.ipAddress,
            base.sourceHost?.externalIP,
        ];
        for (const candidate of ipCandidates) {
            const trimmedIp = String(candidate || "").trim();
            if (trimmedIp && !keyByIp.has(trimmedIp)) {
                keyByIp.set(trimmedIp, key);
            }
        }
        const nameCandidates = [base.name, base.sourceHost?.name, base.sourceHost?.hostname];
        for (const candidate of nameCandidates) {
            const lower = String(candidate || "").trim().toLowerCase();
            if (!lower) {
                continue;
            }
            if (!keyByName.has(lower)) {
                keyByName.set(lower, key);
            }
            const shortName = extractShortHostname(lower);
            if (shortName && !keyByShortName.has(shortName)) {
                keyByShortName.set(shortName, key);
            }
        }
    };

    const upsertHostBeacon = (
        hostNode: HostTreeNode,
        incoming: Partial<HostBeaconNode> & { id: string }
    ): HostBeaconNode => {
        const existing = hostNode.beacons.find((b) => String(b.id) === String(incoming.id));
        if (existing) {
            if (incoming.name && (!existing.name || existing.name.startsWith("Beacon #"))) {
                existing.name = incoming.name;
            }
            if (incoming.principal && !existing.principal) {
                existing.principal = incoming.principal;
            }
            if (incoming.transport && !existing.transport) {
                existing.transport = incoming.transport;
            }
            if (incoming.interval !== undefined && existing.interval === undefined) {
                existing.interval = incoming.interval;
            }
            if (incoming.lastSeenAt && !existing.lastSeenAt) {
                existing.lastSeenAt = incoming.lastSeenAt;
            }
            if (incoming.nextSeenAt && !existing.nextSeenAt) {
                existing.nextSeenAt = incoming.nextSeenAt;
            }
            if (incoming.isOnline) {
                existing.isOnline = true;
            }
            return existing;
        }
        const created: HostBeaconNode = {
            id: String(incoming.id),
            name: incoming.name || `Beacon #${incoming.id}`,
            principal: incoming.principal || "",
            transport: incoming.transport,
            interval: incoming.interval,
            lastSeenAt: incoming.lastSeenAt,
            nextSeenAt: incoming.nextSeenAt,
            isOnline: Boolean(incoming.isOnline),
            activeShells: [],
            closedShells: [],
            activePivots: [],
            closedPivots: [],
        };
        hostNode.beacons.push(created);
        return created;
    };

    const registerNode = (
        base: {
            hostId?: string;
            name: string;
            ip: string;
            subtitle: string;
            isCurrentHost: boolean;
            sourceHost?: KnowledgeHostItem;
        },
        fallbackKey: string
    ): HostTreeNode => {
        const normName = base.name.trim().toLowerCase();
        const normIp = base.ip.trim();
        const existingKey =
            (base.hostId && keyByHostId.get(String(base.hostId))) ||
            (normIp && keyByIp.get(normIp)) ||
            (normName && keyByName.get(normName));

        if (existingKey && nodeByKey.has(existingKey)) {
            const existing = nodeByKey.get(existingKey)!;
            if (base.isCurrentHost) existing.isCurrentHost = true;
            if (!existing.hostId && base.hostId) existing.hostId = base.hostId;
            if (!existing.ip && normIp) existing.ip = normIp;
            if (!existing.sourceHost && base.sourceHost) existing.sourceHost = base.sourceHost;
            for (const beacon of base.sourceHost?.beacons || []) {
                upsertHostBeacon(existing, beacon);
            }
            indexNodeAliases(existingKey, base);
            return existing;
        }

        const key = existingKey || fallbackKey;
        const created: HostTreeNode = {
            key,
            hostId: base.hostId,
            name: base.name || normIp || "Unknown Host",
            ip: normIp,
            subtitle: base.subtitle,
            isCurrentHost: base.isCurrentHost,
            isOnline: false,
            isConnected: false,
            beacons: [],
            activeShells: [],
            activePivots: [],
            closedShells: [],
            closedPivots: [],
            outgoingActivePivots: [],
            outgoingClosedPivots: [],
            sourceHost: base.sourceHost,
        };
        for (const beacon of base.sourceHost?.beacons || []) {
            upsertHostBeacon(created, beacon);
        }
        nodeByKey.set(key, created);
        indexNodeAliases(key, base);
        return created;
    };

    if (currentHostId || currentHostName || currentHostIp) {
        const currIp = currentHostIp || "";
        const currNode = registerNode(
            {
                hostId: currentHostId ? String(currentHostId) : undefined,
                name: currentHostName || currIp || "Current Host",
                ip: currIp,
                subtitle: currIp ? `${currIp} · current host` : "current host",
                isCurrentHost: true,
            },
            `curr-${currentHostId || currentHostName || currIp}`
        );
        for (const beacon of currentHostBeacons) {
            upsertHostBeacon(currNode, beacon);
        }
    }

    for (const h of hosts) {
        const ip = h.ipAddress || h.primaryIP || "";
        const isCurr =
            h.isCurrentHost ||
            Boolean(currentHostId && h.hostId && String(h.hostId) === String(currentHostId));
        const subnet = h.subnet || formatSubnetCidr(ip);
        const subtitle = isCurr
            ? ip
                ? `${ip} · current host`
                : "current host"
            : [ip, subnet].filter(Boolean).join(" · ") || "known host";

        const hostNode = registerNode(
            {
                hostId: h.hostId || (!h.networkHostId ? h.id : undefined),
                name: h.name || ip || h.id,
                ip,
                subtitle,
                isCurrentHost: isCurr,
                sourceHost: h,
            },
            `host-${h.id}`
        );
        if (isCurr) {
            for (const beacon of currentHostBeacons) {
                upsertHostBeacon(hostNode, beacon);
            }
        }
    }

    const resolveNodeForShell = (s: KnowledgeShellItem): HostTreeNode | null => {
        if (s.hostId && keyByHostId.has(String(s.hostId))) {
            return nodeByKey.get(keyByHostId.get(String(s.hostId))!) || null;
        }
        if (s.hostName && keyByName.has(s.hostName.trim().toLowerCase())) {
            return nodeByKey.get(keyByName.get(s.hostName.trim().toLowerCase())!) || null;
        }
        if (s.isCurrentHost) {
            return Array.from(nodeByKey.values()).find((n) => n.isCurrentHost) || null;
        }
        if (!s.hostId && !s.hostName) {
            return null;
        }
        return registerNode(
            {
                hostId: s.hostId,
                name: s.hostName || s.beaconName || "Host",
                ip: "",
                subtitle: s.isCurrentHost ? "current host" : "shell host",
                isCurrentHost: Boolean(s.isCurrentHost),
            },
            `shell-host-${s.hostId || s.hostName}`
        );
    };

    const ensureShellBeacon = (targetNode: HostTreeNode, shell: KnowledgeShellItem) => {
        if (!shell.beaconId) {
            return null;
        }
        const beaconIsOnline =
            shell.beaconOnline !== undefined ? Boolean(shell.beaconOnline) : isShellLive(shell);
        return upsertHostBeacon(targetNode, {
            id: String(shell.beaconId),
            name: shell.beaconName || `Beacon #${shell.beaconId}`,
            principal: shell.beaconPrincipal || "",
            interval: shell.beaconInterval,
            lastSeenAt: shell.beaconLastSeenAt,
            nextSeenAt: shell.beaconNextSeenAt,
            isOnline: beaconIsOnline,
        });
    };

    for (const shell of activeShells) {
        const targetNode = resolveNodeForShell(shell);
        if (!targetNode) continue;
        if (!targetNode.activeShells.some((existing) => existing.id === shell.id)) {
            targetNode.activeShells.push(shell);
        }
        const beaconNode = ensureShellBeacon(targetNode, shell);
        if (beaconNode && !beaconNode.activeShells.some((existing) => existing.id === shell.id)) {
            beaconNode.activeShells.push(shell);
        }
        targetNode.isOnline = true;
    }

    for (const shell of closedShells) {
        const targetNode = resolveNodeForShell(shell);
        if (!targetNode) continue;
        if (!targetNode.closedShells.some((existing) => existing.id === shell.id)) {
            targetNode.closedShells.push(shell);
        }
        const beaconNode = ensureShellBeacon(targetNode, shell);
        if (beaconNode && !beaconNode.closedShells.some((existing) => existing.id === shell.id)) {
            beaconNode.closedShells.push(shell);
        }
    }

    const shellById = new Map<string, KnowledgeShellItem>();
    for (const s of shells) {
        shellById.set(String(s.id), s);
    }

    const attachPivotToBeaconIfOwnedByHost = (
        hostNode: HostTreeNode,
        pivot: ShellPivotItem,
        ownerShell: KnowledgeShellItem | undefined,
        isClosed: boolean
    ) => {
        if (!ownerShell?.beaconId) {
            return;
        }
        const beaconNode = hostNode.beacons.find(
            (b) => String(b.id) === String(ownerShell.beaconId)
        );
        if (!beaconNode) {
            return;
        }
        const list = isClosed ? beaconNode.closedPivots : beaconNode.activePivots;
        if (!list.some((p) => String(p.id) === String(pivot.id))) {
            list.push(pivot);
        }
    };

    const currentHostNodeRef = Array.from(nodeByKey.values()).find((n) => n.isCurrentHost) || null;
    const currentHostShellIds = new Set<string>(
        currentHostNodeRef
            ? [...currentHostNodeRef.activeShells, ...currentHostNodeRef.closedShells].map((s) =>
                  String(s.id)
              )
            : []
    );

    const attachPivot = (pivot: ShellPivotItem, isClosed: boolean) => {
        const isPty = String(pivot.kind || "").toLowerCase() === "pty";
        const ownerShell = pivot.shellId ? shellById.get(String(pivot.shellId)) : undefined;

        if (isPty) {
            let hostNode: HostTreeNode | null = null;
            if (pivot.hostId && keyByHostId.has(String(pivot.hostId))) {
                hostNode = nodeByKey.get(keyByHostId.get(String(pivot.hostId))!) || null;
            } else if (ownerShell) {
                hostNode = resolveNodeForShell(ownerShell);
            } else {
                hostNode = currentHostNodeRef;
            }
            if (hostNode) {
                const enrichedPivot: ShellPivotItem = {
                    ...pivot,
                    shellId: pivot.shellId || ownerShell?.id || currentShellId,
                };
                if (isClosed) {
                    if (!hostNode.closedPivots.some((p) => String(p.id) === String(pivot.id))) {
                        hostNode.closedPivots.push(enrichedPivot);
                    }
                } else {
                    if (!hostNode.activePivots.some((p) => String(p.id) === String(pivot.id))) {
                        hostNode.activePivots.push(enrichedPivot);
                    }
                    hostNode.isOnline = true;
                }
                attachPivotToBeaconIfOwnedByHost(hostNode, enrichedPivot, ownerShell, isClosed);
            }
            return;
        }

        const candidateTargets = [
            extractPivotTargetHost(pivot.activeTarget),
            extractPivotTargetHost(pivot.destination),
        ].filter(Boolean);
        let destKey: string | undefined;
        for (const candidate of candidateTargets) {
            const lower = candidate.toLowerCase();
            const shortCandidate = extractShortHostname(lower);
            destKey =
                keyByIp.get(candidate) ||
                keyByName.get(lower) ||
                (shortCandidate
                    ? keyByName.get(shortCandidate) || keyByShortName.get(shortCandidate)
                    : undefined);
            if (destKey) {
                break;
            }
        }
        const destHost = candidateTargets[0] || "";
        let targetNode = destKey ? nodeByKey.get(destKey) || null : null;

        if (!targetNode && destHost) {
            const subnet = formatSubnetCidr(destHost);
            targetNode = registerNode(
                {
                    name: destHost,
                    ip: destHost,
                    subtitle: [destHost, subnet].filter(Boolean).join(" · ") || "pivot target",
                    isCurrentHost: false,
                },
                `pivot-target-${destHost}`
            );
        } else if (!targetNode) {
            targetNode = currentHostNodeRef;
        }

        const enrichedPivot: ShellPivotItem = {
            ...pivot,
            shellId: pivot.shellId || ownerShell?.id || currentShellId,
        };

        if (targetNode) {
            if (isClosed) {
                if (!targetNode.closedPivots.some((p) => String(p.id) === String(enrichedPivot.id))) {
                    targetNode.closedPivots.push(enrichedPivot);
                }
            } else {
                if (!targetNode.activePivots.some((p) => String(p.id) === String(enrichedPivot.id))) {
                    targetNode.activePivots.push(enrichedPivot);
                }
                targetNode.isConnected = true;
            }
            attachPivotToBeaconIfOwnedByHost(targetNode, enrichedPivot, ownerShell, isClosed);
        }

        // Track outgoing pivots created from the current host's shells for Specific Host View
        const originatesOnCurrentHost =
            currentHostNodeRef &&
            targetNode &&
            !targetNode.isCurrentHost &&
            ((enrichedPivot.shellId && currentHostShellIds.has(String(enrichedPivot.shellId))) ||
                (pivot.hostId &&
                    currentHostNodeRef.hostId &&
                    String(pivot.hostId) === String(currentHostNodeRef.hostId)));

        if (currentHostNodeRef && originatesOnCurrentHost) {
            const outgoingList = isClosed
                ? currentHostNodeRef.outgoingClosedPivots
                : currentHostNodeRef.outgoingActivePivots;
            if (!outgoingList.some((p) => String(p.id) === String(enrichedPivot.id))) {
                outgoingList.push(enrichedPivot);
            }
        }
    };

    for (const pivot of openPivots) {
        attachPivot(pivot, false);
    }

    for (const pivot of closedPivots) {
        attachPivot(pivot, true);
    }

    const allNodes = Array.from(nodeByKey.values());
    const currentHostNode = allNodes.find((n) => n.isCurrentHost) || null;

    if (currentHostNode) {
        currentHostNode.isOnline =
            currentHostNode.activeShells.length > 0 ||
            currentHostNode.activePivots.length > 0 ||
            currentHostNode.beacons.some((b) => b.isOnline) ||
            Boolean(currentHostId);
    }

    for (const node of allNodes) {
        if (
            !node.isOnline &&
            (node.sourceHost?.hasOnlineBeacon || node.beacons.some((b) => b.isOnline))
        ) {
            node.isOnline = true;
        }
        // Order beacons so the primary/best online beacon sits at the top of the host branch
        const bestBeaconId = pickBestBeaconIdForBeacons(node.beacons);
        node.beacons.sort((a, b) => {
            if (bestBeaconId) {
                if (a.id === bestBeaconId && b.id !== bestBeaconId) return -1;
                if (b.id === bestBeaconId && a.id !== bestBeaconId) return 1;
            }
            if (a.isOnline !== b.isOnline) {
                return a.isOnline ? -1 : 1;
            }
            return a.name.localeCompare(b.name);
        });
    }

    const activePivotHostIds = new Set<string>();
    const activePivotTargetTokens = new Set<string>();
    const addTargetToken = (value?: string | null) => {
        const lower = String(value || "").trim().toLowerCase();
        if (!lower) {
            return;
        }
        activePivotTargetTokens.add(lower);
        const shortName = extractShortHostname(lower);
        if (shortName) {
            activePivotTargetTokens.add(shortName);
        }
    };

    for (const pivot of openPivots) {
        if (String(pivot.kind || "").toLowerCase() === "pty") {
            continue;
        }
        addTargetToken(extractPivotTargetHost(pivot.activeTarget));
        addTargetToken(extractPivotTargetHost(pivot.destination));
    }

    for (const node of allNodes) {
        const hasNonPtyActivePivot = node.activePivots.some(
            (p) => String(p.kind || "").toLowerCase() !== "pty"
        );
        if (!hasNonPtyActivePivot) {
            continue;
        }
        const resolvedHostId = node.hostId || node.sourceHost?.hostId;
        if (resolvedHostId) {
            activePivotHostIds.add(String(resolvedHostId));
        }
        addTargetToken(node.ip);
        addTargetToken(node.name);
        addTargetToken(node.sourceHost?.primaryIP);
        addTargetToken(node.sourceHost?.ipAddress);
        addTargetToken(node.sourceHost?.externalIP);
        addTargetToken(node.sourceHost?.name);
        addTargetToken(node.sourceHost?.hostname);
    }

    const hasActivePivotToNode = (node: HostTreeNode): boolean => {
        if (node.activePivots.length > 0) {
            return true;
        }
        const resolvedHostId = node.hostId || node.sourceHost?.hostId;
        if (resolvedHostId && activePivotHostIds.has(String(resolvedHostId))) {
            return true;
        }
        const candidates = [
            node.ip,
            node.name,
            node.sourceHost?.primaryIP,
            node.sourceHost?.ipAddress,
            node.sourceHost?.externalIP,
            node.sourceHost?.name,
            node.sourceHost?.hostname,
        ];
        return candidates.some((candidate) => {
            const lower = String(candidate || "").trim().toLowerCase();
            if (!lower) {
                return false;
            }
            if (activePivotTargetTokens.has(lower)) {
                return true;
            }
            const shortName = extractShortHostname(lower);
            return Boolean(shortName && activePivotTargetTokens.has(shortName));
        });
    };

    const otherNodes = allNodes.filter((n) => !n.isCurrentHost);

    const availablePivotHosts = otherNodes.filter(
        (n) =>
            !hasActivePivotToNode(n) &&
            n.activeShells.length === 0 &&
            n.closedShells.length === 0 &&
            n.closedPivots.length === 0 &&
            n.beacons.length === 0 &&
            !n.sourceHost?.hasOnlineBeacon &&
            n.sourceHost?.isReachable !== false
    );
    const availableHostKeys = new Set(availablePivotHosts.map((n) => n.key));

    const activePivotHosts = otherNodes
        .filter((n) => n.activePivots.length > 0)
        .sort(compareHostTreeNodes);

    const hiddenOtherHosts = otherNodes
        .filter(
            (n) =>
                n.activePivots.length === 0 &&
                !availableHostKeys.has(n.key) &&
                (n.activeShells.length > 0 ||
                    n.closedShells.length > 0 ||
                    n.closedPivots.length > 0 ||
                    n.beacons.length > 0 ||
                    Boolean(n.sourceHost?.hasOnlineBeacon) ||
                    Boolean(n.hostId))
        )
        .sort(compareHostTreeNodes);

    const visibleHostCount =
        (currentHostNode ? 1 : 0) +
        activePivotHosts.length +
        (currentHostNode ? availablePivotHosts.length : 0);

    return {
        currentHostNode,
        activePivotHosts,
        hiddenOtherHosts,
        availablePivotHosts,
        totalHostCount: allNodes.length,
        visibleHostCount,
        activePivotCount: openPivots.length,
    };
};

const matchesQueryValue = (value: string | null | undefined, query: string): boolean => {
    if (!value || !query) {
        return false;
    }
    return value.toLowerCase().includes(query);
};

interface NormalizedHostFilters {
    name: string;
    ip: string;
    input: string;
    output: string;
    hasAny: boolean;
    activeFields: Array<keyof HostsFilterState>;
}

const normalizeHostsFilters = (
    rawFilter: string | Partial<HostsFilterState>
): NormalizedHostFilters => {
    if (typeof rawFilter === "string") {
        const q = rawFilter.trim().toLowerCase();
        if (!q) {
            return { ...EMPTY_HOSTS_FILTERS, hasAny: false, activeFields: [] };
        }
        return {
            name: q,
            ip: q,
            input: q,
            output: q,
            hasAny: true,
            activeFields: ["name"],
        };
    }

    const name = String(rawFilter.name || "").trim().toLowerCase();
    const ip = String(rawFilter.ip || "").trim().toLowerCase();
    const input = String(rawFilter.input || "").trim().toLowerCase();
    const output = String(rawFilter.output || "").trim().toLowerCase();
    const activeFields: Array<keyof HostsFilterState> = [];
    if (name) activeFields.push("name");
    if (ip) activeFields.push("ip");
    if (input) activeFields.push("input");
    if (output) activeFields.push("output");

    return {
        name,
        ip,
        input,
        output,
        hasAny: activeFields.length > 0,
        activeFields,
    };
};

const getHostFieldMatches = (node: HostTreeNode, query: string): Record<keyof HostsFilterState, boolean> => {
    const nameCandidates: Array<string | undefined | null> = [
        node.name,
        node.sourceHost?.hostname,
        node.sourceHost?.platform,
        ...(node.sourceHost?.tags || []).map((tag) => tag.name),
    ];
    const ipCandidates: Array<string | undefined | null> = [
        node.ip,
        node.sourceHost?.primaryIP,
        node.sourceHost?.ipAddress,
        node.sourceHost?.externalIP,
        node.sourceHost?.subnet || formatSubnetCidr(node.ip),
        ...(node.sourceHost?.openPorts || []).map((port) => `:${port}`),
    ];
    return {
        name: nameCandidates.some((c) => matchesQueryValue(c, query)),
        ip: ipCandidates.some((c) => matchesQueryValue(c, query)),
        input: false,
        output: false,
    };
};

const doesHostMatchFilters = (
    node: HostTreeNode,
    filters: NormalizedHostFilters,
    isLegacyStringMode: boolean
): boolean => {
    if (!filters.hasAny) {
        return true;
    }
    if (isLegacyStringMode) {
        const q = filters.name;
        const fields = getHostFieldMatches(node, q);
        return fields.name || fields.ip;
    }
    return filters.activeFields.every((field) => {
        const q = filters[field];
        if (field === "input" || field === "output") {
            return false;
        }
        return getHostFieldMatches(node, q)[field];
    });
};

const doesBeaconMatchDirect = (
    beacon: HostBeaconNode,
    field: keyof HostsFilterState,
    query: string
): boolean => {
    if (field !== "name") {
        return false;
    }
    return [beacon.name, beacon.principal, beacon.id, `#${beacon.id}`].some((c) =>
        matchesQueryValue(c, query)
    );
};

const doesShellMatchDirectField = (
    shell: KnowledgeShellItem,
    field: keyof HostsFilterState,
    query: string,
    tasksByShellId: Map<string, ShellTaskItem[]>
): boolean => {
    const inlineTasks = shell.tasks || [];
    const externalTasks = tasksByShellId.get(String(shell.id)) || [];
    const allTasks = [...inlineTasks, ...externalTasks];

    if (field === "name") {
        const meta: Array<string | undefined | null> = [
            "shell",
            shell.id,
            `#${shell.id}`,
            shell.beaconName,
            shell.beaconPrincipal,
            shell.ownerName,
            shell.hostName,
            ...(shell.activeUsers || []).map((u) => u.name),
        ];
        return meta.some((c) => matchesQueryValue(c, query));
    }
    if (field === "ip") {
        return false;
    }
    if (field === "input") {
        return allTasks.some((t) => matchesQueryValue(stripAnsi(t.input), query));
    }
    return allTasks.some(
        (t) =>
            matchesQueryValue(stripAnsi(t.output), query) ||
            matchesQueryValue(stripAnsi(t.error), query)
    );
};

const doesPivotMatchDirectField = (
    pivot: ShellPivotItem,
    field: keyof HostsFilterState,
    query: string
): boolean => {
    const isPty = String(pivot.kind || "").toLowerCase() === "pty";
    const tabLabel = formatPivotTabLabel({
        type: pivot.kind || "ssh",
        target: pivot.activeTarget || pivot.destination,
        customTitle: pivot.customTitle,
    });
    if (field === "name") {
        const candidates: Array<string | undefined | null> = [
            isPty ? "pty" : "pivot",
            pivot.kind,
            pivot.id,
            `#${pivot.id}`,
            pivot.customTitle,
            pivot.destination,
            pivot.activeTarget,
            tabLabel,
            pivot.hostName,
        ];
        return candidates.some((c) => matchesQueryValue(c, query));
    }
    if (field === "ip") {
        const candidates: Array<string | undefined | null> = [
            pivot.destination,
            pivot.activeTarget,
            extractPivotTargetHost(pivot.activeTarget),
            extractPivotTargetHost(pivot.destination),
        ];
        return candidates.some((c) => matchesQueryValue(c, query));
    }
    // Terminal PTY/SSH sessions store combined I/O stream in pivot.data
    return matchesQueryValue(stripAnsi(pivot.data), query);
};

const evaluateChildWithAncestorContext = (
    filters: NormalizedHostFilters,
    isLegacyStringMode: boolean,
    directMatcher: (field: keyof HostsFilterState, query: string) => boolean,
    ancestorMatcher: (field: keyof HostsFilterState, query: string) => boolean
): { contextualMatch: boolean; hasDirectMatch: boolean } => {
    if (isLegacyStringMode) {
        const q = filters.name;
        const fields: Array<keyof HostsFilterState> = ["name", "ip", "input", "output"];
        const matchedDirect = fields.some((f) => directMatcher(f, q));
        return {
            contextualMatch: matchedDirect,
            hasDirectMatch: matchedDirect,
        };
    }

    let hasDirectMatch = false;
    const contextualMatch = filters.activeFields.every((field) => {
        const query = filters[field];
        const direct = directMatcher(field, query);
        if (direct) {
            hasDirectMatch = true;
            return true;
        }
        if (field === "name" || field === "ip") {
            return ancestorMatcher(field, query);
        }
        return false;
    });

    return {
        contextualMatch,
        hasDirectMatch: contextualMatch && hasDirectMatch,
    };
};

const filterHostNode = (
    node: HostTreeNode,
    filters: NormalizedHostFilters,
    isLegacyStringMode: boolean,
    tasksByShellId: Map<string, ShellTaskItem[]>
): HostTreeNode | null => {
    const hostMatchesDirect = doesHostMatchFilters(node, filters, isLegacyStringMode);
    const hostFieldMatch = (field: keyof HostsFilterState, q: string) =>
        getHostFieldMatches(node, q)[field];

    const beaconByShellId = new Map<string, HostBeaconNode>();
    for (const beacon of node.beacons) {
        for (const shell of [...beacon.activeShells, ...beacon.closedShells]) {
            beaconByShellId.set(String(shell.id), beacon);
        }
    }

    const shellMatches = (shell: KnowledgeShellItem, ancestorBeacon?: HostBeaconNode) => {
        const beacon = ancestorBeacon || beaconByShellId.get(String(shell.id));
        return evaluateChildWithAncestorContext(
            filters,
            isLegacyStringMode,
            (field, q) => doesShellMatchDirectField(shell, field, q, tasksByShellId),
            (field, q) =>
                hostFieldMatch(field, q) ||
                Boolean(beacon && doesBeaconMatchDirect(beacon, field, q))
        );
    };

    const pivotMatches = (pivot: ShellPivotItem, ancestorBeacon?: HostBeaconNode) => {
        const beacon =
            ancestorBeacon || (pivot.shellId ? beaconByShellId.get(String(pivot.shellId)) : undefined);
        return evaluateChildWithAncestorContext(
            filters,
            isLegacyStringMode,
            (field, q) => doesPivotMatchDirectField(pivot, field, q),
            (field, q) =>
                hostFieldMatch(field, q) ||
                Boolean(beacon && doesBeaconMatchDirect(beacon, field, q))
        );
    };

    const matchedActiveShells = node.activeShells.filter((shell) => {
        const res = shellMatches(shell);
        return res.hasDirectMatch || (res.contextualMatch && !hostMatchesDirect);
    });
    const matchedActivePivots = node.activePivots.filter((pivot) => {
        const res = pivotMatches(pivot);
        return res.hasDirectMatch || (res.contextualMatch && !hostMatchesDirect);
    });
    const matchedClosedShells = node.closedShells.filter(
        (shell) => shellMatches(shell).hasDirectMatch
    );
    const matchedClosedPivots = node.closedPivots.filter(
        (pivot) => pivotMatches(pivot).hasDirectMatch
    );
    const matchedOutgoingActive = node.outgoingActivePivots.filter((pivot) => {
        const res = pivotMatches(pivot);
        return res.hasDirectMatch || (res.contextualMatch && !hostMatchesDirect);
    });
    const matchedOutgoingClosed = node.outgoingClosedPivots.filter(
        (pivot) => pivotMatches(pivot).hasDirectMatch
    );

    const matchedBeacons: HostBeaconNode[] = [];
    for (const beacon of node.beacons) {
        const beaconEval = evaluateChildWithAncestorContext(
            filters,
            isLegacyStringMode,
            (field, q) => doesBeaconMatchDirect(beacon, field, q),
            hostFieldMatch
        );

        const bActiveShells = beacon.activeShells.filter((s) => {
            const res = shellMatches(s, beacon);
            return res.hasDirectMatch || (beaconEval.hasDirectMatch && res.contextualMatch);
        });
        const bActivePivots = beacon.activePivots.filter((p) => {
            const res = pivotMatches(p, beacon);
            return res.hasDirectMatch || (beaconEval.hasDirectMatch && res.contextualMatch);
        });
        const bClosedShells = beacon.closedShells.filter(
            (s) => shellMatches(s, beacon).hasDirectMatch
        );
        const bClosedPivots = beacon.closedPivots.filter(
            (p) => pivotMatches(p, beacon).hasDirectMatch
        );

        const beaconHasDirect =
            beaconEval.hasDirectMatch ||
            bActiveShells.length > 0 ||
            bActivePivots.length > 0 ||
            (node.isCurrentHost && bClosedShells.length > 0) ||
            bClosedPivots.length > 0;

        if (!beaconHasDirect && !hostMatchesDirect) {
            continue;
        }

        if (beaconHasDirect) {
            matchedBeacons.push({
                ...beacon,
                activeShells: beaconEval.hasDirectMatch && bActiveShells.length === 0 ? beacon.activeShells : bActiveShells,
                activePivots: beaconEval.hasDirectMatch && bActivePivots.length === 0 ? beacon.activePivots : bActivePivots,
                closedShells: node.isCurrentHost ? bClosedShells : [],
                closedPivots: bClosedPivots,
                hasMatchingClosed: node.isCurrentHost
                    ? bClosedShells.length > 0 || bClosedPivots.length > 0
                    : bClosedPivots.length > 0,
            });
        } else if (hostMatchesDirect && beacon.isOnline) {
            matchedBeacons.push({
                ...beacon,
                closedShells: node.isCurrentHost ? beacon.closedShells : [],
                closedPivots: [],
                hasMatchingClosed: false,
            });
        }
    }

    const hasAnyChildMatch =
        matchedActiveShells.length > 0 ||
        matchedActivePivots.length > 0 ||
        (node.isCurrentHost && matchedClosedShells.length > 0) ||
        matchedClosedPivots.length > 0 ||
        matchedOutgoingActive.length > 0 ||
        (node.isCurrentHost && matchedOutgoingClosed.length > 0) ||
        matchedBeacons.some(
            (b) =>
                doesBeaconMatchDirect(b, "name", filters.name) ||
                b.activeShells.length > 0 ||
                b.activePivots.length > 0 ||
                Boolean(b.hasMatchingClosed)
        );

    if (!hostMatchesDirect && !hasAnyChildMatch) {
        return null;
    }

    const hasMatchingClosed = node.isCurrentHost
        ? matchedClosedShells.length > 0 ||
          matchedClosedPivots.length > 0 ||
          matchedOutgoingClosed.length > 0
        : matchedClosedPivots.length > 0;

    if (hostMatchesDirect && !hasAnyChildMatch) {
        return {
            ...node,
            beacons: matchedBeacons,
            hasMatchingClosed: false,
        };
    }

    return {
        ...node,
        beacons: matchedBeacons,
        activeShells: hostMatchesDirect && matchedActiveShells.length === 0 ? node.activeShells : matchedActiveShells,
        activePivots: hostMatchesDirect && matchedActivePivots.length === 0 ? node.activePivots : matchedActivePivots,
        outgoingActivePivots:
            hostMatchesDirect && matchedOutgoingActive.length === 0
                ? node.outgoingActivePivots
                : matchedOutgoingActive,
        closedShells: node.isCurrentHost
            ? matchedClosedShells
            : [],
        closedPivots: matchedClosedPivots,
        outgoingClosedPivots: node.isCurrentHost ? matchedOutgoingClosed : [],
        hasMatchingClosed,
    };
};

export const filterHostsTreeSections = (
    sections: HostsTreeSections,
    rawFilter: string | Partial<HostsFilterState>,
    shellTasks: ShellTaskItem[] = []
): HostsTreeSections => {
    const isLegacyStringMode = typeof rawFilter === "string";
    const filters = normalizeHostsFilters(rawFilter);
    if (!filters.hasAny) {
        return sections;
    }

    const tasksByShellId = new Map<string, ShellTaskItem[]>();
    for (const task of shellTasks) {
        if (!task.shellId) {
            continue;
        }
        const key = String(task.shellId);
        const list = tasksByShellId.get(key) || [];
        list.push(task);
        tasksByShellId.set(key, list);
    }

    const matchedAvailableHosts = sections.availablePivotHosts.filter((node) =>
        doesHostMatchFilters(node, filters, isLegacyStringMode)
    );

    const currentHostNode = sections.currentHostNode
        ? filterHostNode(sections.currentHostNode, filters, isLegacyStringMode, tasksByShellId)
        : null;

    const matchedActivePivotHosts = sections.activePivotHosts
        .map((node) => filterHostNode(node, filters, isLegacyStringMode, tasksByShellId))
        .filter((node): node is HostTreeNode => node !== null);

    const matchedHiddenOtherHosts = (sections.hiddenOtherHosts || [])
        .map((node) => filterHostNode(node, filters, isLegacyStringMode, tasksByShellId))
        .filter((node): node is HostTreeNode => node !== null);

    const activePivotHosts = [...matchedActivePivotHosts, ...matchedHiddenOtherHosts];

    const visibleHostCount =
        (currentHostNode ? 1 : 0) +
        activePivotHosts.length +
        matchedAvailableHosts.length;

    const activePivotCount =
        (currentHostNode?.activePivots.length || 0) +
        activePivotHosts.reduce((sum, node) => sum + node.activePivots.length, 0);

    return {
        ...sections,
        currentHostNode,
        activePivotHosts,
        hiddenOtherHosts: [],
        availablePivotHosts: matchedAvailableHosts,
        visibleHostCount,
        activePivotCount,
    };
};
