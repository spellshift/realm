import {
    KnowledgeCredentialItem,
    KnowledgeFileItem,
    KnowledgeHostItem,
    KnowledgeShellItem,
    ShellPivotItem,
} from "../hooks/useShellData";

export const getIpv4SubnetPrefix = (ipOrCidr?: string | null): string | null => {
    if (!ipOrCidr) {
        return null;
    }
    const baseIp = ipOrCidr.trim().split("/")[0];
    const parts = baseIp.split(".");
    if (parts.length !== 4) {
        return null;
    }
    return `${parts[0]}.${parts[1]}.${parts[2]}.`;
};

export const formatSubnetCidr = (ipOrSubnet?: string | null): string => {
    if (!ipOrSubnet) {
        return "";
    }
    const trimmed = ipOrSubnet.trim();
    if (trimmed.includes("/")) {
        return trimmed;
    }
    const prefix = getIpv4SubnetPrefix(trimmed);
    if (!prefix) {
        return "";
    }
    return `${prefix}0/24`;
};

export const extractPivotTargetHost = (destination?: string | null): string => {
    if (!destination) {
        return "";
    }
    const trimmed = destination.trim();
    const afterAt = trimmed.includes("@") ? trimmed.split("@").pop() || trimmed : trimmed;
    if (afterAt.startsWith("[")) {
        const bracketEnd = afterAt.indexOf("]");
        if (bracketEnd > 1) {
            return afterAt.slice(1, bracketEnd);
        }
    }
    const lastColon = afterAt.lastIndexOf(":");
    if (lastColon > 0 && afterAt.indexOf(":") === lastColon) {
        return afterAt.slice(0, lastColon);
    }
    return afterAt;
};

export function formatPivotTabLabel(tab: { type: string; target: string }): string {
    if (tab.type.toLowerCase() === "ssh") {
        const raw = (tab.target || "").trim();
        if (!raw) return "SSH";
        const afterAt = raw.includes("@") ? raw.slice(raw.lastIndexOf("@") + 1) : raw;
        const hostOnly = afterAt.replace(/:\d+$/, "");
        return hostOnly || afterAt || raw;
    }
    const cleaned = (tab.target || "PTY").replace(/^PTY\s*#\d+$/i, "PTY").trim();
    if (!cleaned || cleaned.toLowerCase() === "exit" || cleaned.toLowerCase() === "pty") {
        return "PTY";
    }
    return cleaned.length > 24 ? `${cleaned.slice(0, 24)}...` : cleaned;
}

export const formatFileSize = (bytes?: number | null): string => {
    if (bytes === undefined || bytes === null || Number.isNaN(bytes)) {
        return "0 B";
    }
    if (bytes < 1024) {
        return `${bytes} B`;
    }
    if (bytes < 1024 * 1024) {
        return `${(bytes / 1024).toFixed(1)} KB`;
    }
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
};

export const parsePortList = (raw?: string | null): number[] => {
    if (!raw) {
        return [];
    }
    const tokens = raw.split(/[\s,]+/);
    const seen = new Set<number>();
    for (const token of tokens) {
        const cleaned = token.replace(/^:/, "").trim();
        if (!cleaned) {
            continue;
        }
        const num = Number(cleaned);
        if (Number.isInteger(num) && num >= 1 && num <= 65535) {
            seen.add(num);
        }
    }
    return Array.from(seen).sort((a, b) => a - b);
};

export const formatPortListInput = (ports?: number[] | null): string => {
    if (!Array.isArray(ports) || ports.length === 0) {
        return "";
    }
    return ports.join(", ");
};

export const togglePortInList = (openPorts: number[], port: number): number[] => {
    if (openPorts.includes(port)) {
        return openPorts.filter((p) => p !== port);
    }
    return [...openPorts, port].sort((a, b) => a - b);
};

export interface BuildGraphDataParams {
    currentHostNode: any;
    hostId?: string;
    currentShellId?: string;
    currentHostTags: string[];
    currentShellPivots: ShellPivotItem[];
    knowledgeData: any;
}

export interface UnifiedGraphDataResult {
    hostCredentials: KnowledgeCredentialItem[];
    allCredentials: KnowledgeCredentialItem[];
    networkHosts: KnowledgeHostItem[];
    hostFiles: KnowledgeFileItem[];
    hostShells: KnowledgeShellItem[];
    nearbyPivots: ShellPivotItem[];
}

export const buildUnifiedGraphData = ({
    currentHostNode,
    hostId,
    currentHostTags,
    currentShellPivots,
    knowledgeData,
}: BuildGraphDataParams): UnifiedGraphDataResult => {
    const currentIdStr = String(currentHostNode?.id || hostId || "");
    const currentHostName = currentHostNode?.name || "Current Host";
    const currentPrimaryIp = currentHostNode?.primaryIP || "";
    const currentSubnetPrefix = getIpv4SubnetPrefix(currentPrimaryIp);

    // 1. Collect raw Host nodes and NetworkHost nodes from knowledgeData
    const rawHostEdges =
        knowledgeData?.hosts?.edges ||
        (knowledgeData?.networkHosts?.edges?.[0]?.node?.primaryIP !== undefined
            ? knowledgeData?.networkHosts?.edges
            : []) ||
        [];
    const rawNetworkHostEdges =
        knowledgeData?.networkHosts?.edges?.[0]?.node?.ipAddress !== undefined ||
        knowledgeData?.networkHosts?.edges?.[0]?.node?.openPorts !== undefined
            ? knowledgeData?.networkHosts?.edges || []
            : [];

    const rawHosts = rawHostEdges.map((e: any) => e?.node).filter(Boolean);
    const rawNetworkHosts = rawNetworkHostEdges.map((e: any) => e?.node).filter(Boolean);

    // 2. Determine reachable subnets and networks from current host & non-PTY pivots
    const reachableSubnets = new Set<string>();
    const reachableNetworkIds = new Set<string>();
    const pivotedTargets = new Set<string>();

    if (currentSubnetPrefix) {
        reachableSubnets.add(currentSubnetPrefix);
    }

    for (const p of currentShellPivots) {
        if (String(p.kind || "").toLowerCase() === "pty") {
            continue;
        }
        const targetHost = extractPivotTargetHost(p.destination);
        if (!targetHost) {
            continue;
        }
        pivotedTargets.add(targetHost.toLowerCase());
        const targetPrefix = getIpv4SubnetPrefix(targetHost);
        if (targetPrefix) {
            reachableSubnets.add(targetPrefix);
        }
    }

    for (const nh of rawNetworkHosts) {
        const linkedHostId = nh.host?.id ? String(nh.host.id) : "";
        const nhIp = nh.ipAddress || nh.host?.primaryIP || "";
        const isCurrentNh =
            (currentIdStr && linkedHostId === currentIdStr) ||
            (currentPrimaryIp && nhIp === currentPrimaryIp);
        if (!isCurrentNh) {
            continue;
        }
        const nhPrefix = getIpv4SubnetPrefix(nh.subnet || nhIp);
        if (nhPrefix) {
            reachableSubnets.add(nhPrefix);
        }
        for (const ne of nh.networks?.edges || []) {
            const netNode = ne?.node;
            if (!netNode) {
                continue;
            }
            reachableNetworkIds.add(String(netNode.id));
            const netPrefix = getIpv4SubnetPrefix(netNode.subnet);
            if (netPrefix) {
                reachableSubnets.add(netPrefix);
            }
        }
    }

    // 3. Map Host entities by ID, IP, and Name so NetworkHost records can merge cleanly
    const hostById = new Map<string, any>();
    const hostByIp = new Map<string, any>();
    const hostByName = new Map<string, any>();

    if (currentHostNode?.id) {
        hostById.set(String(currentHostNode.id), currentHostNode);
        if (currentHostNode.primaryIP) {
            hostByIp.set(currentHostNode.primaryIP, currentHostNode);
        }
        if (currentHostNode.name) {
            hostByName.set(String(currentHostNode.name).toLowerCase(), currentHostNode);
        }
    }

    for (const h of rawHosts) {
        const hId = String(h.id);
        hostById.set(hId, h);
        if (h.primaryIP) {
            hostByIp.set(h.primaryIP, h);
        }
        if (h.name) {
            hostByName.set(String(h.name).toLowerCase(), h);
        }
    }

    const mergedHostIds = new Set<string>();
    const unifiedHosts: KnowledgeHostItem[] = [];

    const extractCredsFromHost = (h: any, isCurr: boolean): KnowledgeCredentialItem[] => {
        if (!h) {
            return [];
        }
        const hId = String(h.id);
        return (h.credentials?.edges || [])
            .map((ce: any) => ce?.node)
            .filter(Boolean)
            .map((c: any) => ({
                id: String(c.id),
                principal: c.principal,
                kind: c.kind,
                secret: c.secret,
                createdAt: c.createdAt,
                hostId: hId,
                hostName: h.name || h.primaryIP || hId,
                hostPrimaryIP: h.primaryIP,
                isCurrentHost: isCurr,
            }));
    };

    const extractCredsFromNetworkHost = (
        nhNode: any,
        fallbackHost: any,
        isCurr: boolean
    ): KnowledgeCredentialItem[] => {
        const byId = new Map<string, KnowledgeCredentialItem>();
        const nhHostId = fallbackHost?.id ? String(fallbackHost.id) : `nh-${nhNode.id}`;
        const nhName =
            nhNode.name || fallbackHost?.name || nhNode.hostname || nhNode.ipAddress || nhHostId;
        const nhIp = nhNode.ipAddress || fallbackHost?.primaryIP;

        for (const ce of nhNode?.credentials?.edges || []) {
            const c = ce?.node;
            if (!c?.id) {
                continue;
            }
            const cId = String(c.id);
            byId.set(cId, {
                id: cId,
                principal: c.principal,
                kind: c.kind,
                secret: c.secret,
                createdAt: c.createdAt,
                hostId: nhHostId,
                hostName: nhName,
                hostPrimaryIP: nhIp,
                isCurrentHost: isCurr,
            });
        }

        for (const c of extractCredsFromHost(fallbackHost, isCurr)) {
            if (!byId.has(c.id)) {
                byId.set(c.id, c);
            }
        }

        return Array.from(byId.values());
    };

    // 4. Process NetworkHost records first and link to matching Host if present
    for (const nh of rawNetworkHosts) {
        const linkedHost =
            (nh.host?.id && hostById.get(String(nh.host.id))) ||
            nh.host ||
            (nh.ipAddress && hostByIp.get(nh.ipAddress)) ||
            (nh.hostname && hostByName.get(String(nh.hostname).toLowerCase())) ||
            (nh.name && hostByName.get(String(nh.name).toLowerCase())) ||
            null;

        const linkedHostId = linkedHost?.id ? String(linkedHost.id) : undefined;
        if (linkedHostId) {
            mergedHostIds.add(linkedHostId);
        }

        const ipAddr = nh.ipAddress || linkedHost?.primaryIP || "";
        const subnet = nh.subnet || formatSubnetCidr(ipAddr);
        const prefix = getIpv4SubnetPrefix(subnet || ipAddr);
        const isCurrentHost =
            Boolean(currentIdStr && linkedHostId === currentIdStr) ||
            Boolean(currentPrimaryIp && ipAddr && ipAddr === currentPrimaryIp);
        const sameSubnet = Boolean(currentSubnetPrefix && prefix && currentSubnetPrefix === prefix);
        const inReachableSubnet = Boolean(prefix && reachableSubnets.has(prefix));

        const networks = (nh.networks?.edges || [])
            .map((ne: any) => ne?.node)
            .filter(Boolean)
            .map((n: any) => ({
                id: String(n.id),
                name: n.name,
                subnet: n.subnet,
                description: n.description,
            }));

        const sharesNetwork = networks.some(
            (n: { id: string; subnet: string }) =>
                reachableNetworkIds.has(n.id) ||
                Boolean(getIpv4SubnetPrefix(n.subnet) && reachableSubnets.has(getIpv4SubnetPrefix(n.subnet)!))
        );
        const isPivotedTarget =
            (ipAddr && pivotedTargets.has(ipAddr.toLowerCase())) ||
            (nh.hostname && pivotedTargets.has(String(nh.hostname).toLowerCase())) ||
            (nh.name && pivotedTargets.has(String(nh.name).toLowerCase()));

        const tags = (linkedHost?.tags?.edges || []).map((te: any) => te?.node).filter(Boolean);
        const sharedTags = tags
            .map((t: any) => t.name)
            .filter((name: string) => currentHostTags.includes(name));
        const creds = extractCredsFromNetworkHost(nh, linkedHost, isCurrentHost);
        const openPorts = Array.isArray(nh.openPorts)
            ? nh.openPorts.map((p: any) => Number(p)).filter((p: number) => !Number.isNaN(p) && p > 0)
            : [];

        unifiedHosts.push({
            id: linkedHostId || `nh-${nh.id}`,
            hostId: linkedHostId,
            networkHostId: String(nh.id),
            name: nh.name || linkedHost?.name || nh.hostname || ipAddr || `Host #${nh.id}`,
            hostname: nh.hostname || linkedHost?.name || "",
            primaryIP: ipAddr,
            ipAddress: ipAddr,
            subnet,
            openPorts,
            networks,
            externalIP: linkedHost?.externalIP,
            platform: linkedHost?.platform,
            lastSeenAt: linkedHost?.lastSeenAt,
            tags,
            beaconCount: linkedHost?.beacons?.totalCount ?? 0,
            credentialCount: Math.max(linkedHost?.credentials?.totalCount ?? 0, creds.length),
            credentials: creds,
            isCurrentHost,
            sharedTags,
            sameSubnet,
            isReachable:
                isCurrentHost ||
                sameSubnet ||
                inReachableSubnet ||
                sharesNetwork ||
                Boolean(isPivotedTarget) ||
                sharedTags.length > 0 ||
                !currentSubnetPrefix,
        });
    }

    // 5. Add remaining Host records not already represented by a NetworkHost
    for (const h of rawHosts) {
        const hId = String(h.id);
        if (mergedHostIds.has(hId)) {
            continue;
        }
        const isCurrentHost = hId === currentIdStr;
        const ipAddr = h.primaryIP || "";
        const subnet = formatSubnetCidr(ipAddr);
        const prefix = getIpv4SubnetPrefix(ipAddr);
        const sameSubnet = Boolean(currentSubnetPrefix && prefix && currentSubnetPrefix === prefix);
        const inReachableSubnet = Boolean(prefix && reachableSubnets.has(prefix));
        const isPivotedTarget =
            (ipAddr && pivotedTargets.has(ipAddr.toLowerCase())) ||
            (h.name && pivotedTargets.has(String(h.name).toLowerCase()));

        const tags = (h.tags?.edges || []).map((te: any) => te?.node).filter(Boolean);
        const sharedTags = tags
            .map((t: any) => t.name)
            .filter((name: string) => currentHostTags.includes(name));
        const creds = extractCredsFromHost(h, isCurrentHost);

        unifiedHosts.push({
            id: hId,
            hostId: hId,
            name: h.name || ipAddr || hId,
            hostname: h.name || "",
            primaryIP: ipAddr,
            ipAddress: ipAddr,
            subnet,
            openPorts: [],
            networks: [],
            externalIP: h.externalIP,
            platform: h.platform,
            lastSeenAt: h.lastSeenAt,
            tags,
            beaconCount: h.beacons?.totalCount ?? (h.beacons?.edges?.length || 0),
            credentialCount: h.credentials?.totalCount ?? creds.length,
            credentials: creds,
            isCurrentHost,
            sharedTags,
            sameSubnet,
            isReachable:
                isCurrentHost ||
                sameSubnet ||
                inReachableSubnet ||
                Boolean(isPivotedTarget) ||
                sharedTags.length > 0 ||
                !currentSubnetPrefix,
        });
    }

    // Sort hosts prioritizing adjacent/reachable hosts and same-subnet hosts with open ports
    const sortedHosts = [...unifiedHosts].sort((a, b) => {
        if (a.isCurrentHost !== b.isCurrentHost) {
            return a.isCurrentHost ? 1 : -1;
        }
        if (Boolean(a.isReachable) !== Boolean(b.isReachable)) {
            return a.isReachable ? -1 : 1;
        }
        const aScore = (a.sameSubnet ? 2 : 0) + ((a.openPorts?.length || 0) > 0 ? 1 : 0);
        const bScore = (b.sameSubnet ? 2 : 0) + ((b.openPorts?.length || 0) > 0 ? 1 : 0);
        if (aScore !== bScore) {
            return bScore - aScore;
        }
        return a.name.localeCompare(b.name);
    });

    const adjacentHostIds = new Set<string>();
    if (currentIdStr) {
        adjacentHostIds.add(currentIdStr);
    }
    for (const uh of sortedHosts) {
        if (uh.hostId) {
            adjacentHostIds.add(uh.hostId);
        }
    }

    // 6. Credentials from current host and nearby hosts
    // Credentials explicitly linked to an adjacent NetworkHost take attribution precedence
    // over a fallback currentHost association.
    const adjacentNetworkHostCredIds = new Set<string>();
    for (const h of sortedHosts) {
        if (h.isCurrentHost) {
            continue;
        }
        for (const c of h.credentials) {
            adjacentNetworkHostCredIds.add(c.id);
        }
    }

    const hostCredentials = extractCredsFromHost(currentHostNode, true).filter(
        (c) => !adjacentNetworkHostCredIds.has(c.id)
    );
    const seenCredIds = new Set<string>();
    const allCredentials: KnowledgeCredentialItem[] = [];
    for (const c of hostCredentials) {
        seenCredIds.add(c.id);
        allCredentials.push(c);
    }
    for (const h of sortedHosts) {
        for (const c of h.credentials) {
            if (!seenCredIds.has(c.id)) {
                seenCredIds.add(c.id);
                allCredentials.push(c);
            }
        }
    }
    for (const h of rawHosts) {
        const isCurr = String(h.id) === currentIdStr;
        for (const c of extractCredsFromHost(h, isCurr)) {
            if (!seenCredIds.has(c.id)) {
                seenCredIds.add(c.id);
                allCredentials.push(c);
            }
        }
    }

    // 7. Reported files from current host and adjacent hosts
    const seenFileIds = new Set<string>();
    const hostFiles: KnowledgeFileItem[] = [];
    const collectFilesFromHost = (h: any, isCurr: boolean) => {
        if (!h) {
            return;
        }
        const hId = String(h.id || "");
        const hName = h.name || h.primaryIP || hId;
        for (const fe of h.files?.edges || []) {
            const f = fe?.node;
            if (!f?.id) {
                continue;
            }
            const fId = String(f.id);
            if (seenFileIds.has(fId)) {
                continue;
            }
            seenFileIds.add(fId);
            hostFiles.push({
                id: fId,
                path: f.path,
                size: f.size,
                owner: f.owner,
                permissions: f.permissions,
                createdAt: f.createdAt,
                hostId: hId,
                hostName: hName,
                isCurrentHost: isCurr,
            });
        }
    };

    collectFilesFromHost(currentHostNode, true);
    for (const h of rawHosts) {
        const hId = String(h.id);
        if (adjacentHostIds.has(hId)) {
            collectFilesFromHost(h, hId === currentIdStr);
        }
    }

    // 8. Shells and Pivot sessions from current host and adjacent hosts
    const seenShellIds = new Set<string>();
    const hostShells: KnowledgeShellItem[] = [];
    const seenPivotIds = new Set<string>();
    const nearbyPivots: ShellPivotItem[] = [];

    for (const p of currentShellPivots) {
        if (!p?.id) {
            continue;
        }
        const pId = String(p.id);
        seenPivotIds.add(pId);
        nearbyPivots.push({
            ...p,
            id: pId,
            hostId: currentIdStr,
            hostName: currentHostName,
            isCurrentHost: true,
        });
    }

    const collectShellsAndPivotsFromHost = (h: any, isCurr: boolean) => {
        if (!h) {
            return;
        }
        const hId = String(h.id || "");
        const hName = h.name || h.primaryIP || hId;
        for (const be of h.beacons?.edges || []) {
            const b = be?.node;
            if (!b) {
                continue;
            }
            for (const se of b.shells?.edges || []) {
                const s = se?.node;
                if (!s?.id) {
                    continue;
                }
                const sId = String(s.id);
                if (!seenShellIds.has(sId)) {
                    seenShellIds.add(sId);
                    hostShells.push({
                        id: sId,
                        createdAt: s.createdAt,
                        closedAt: s.closedAt,
                        beaconId: String(b.id),
                        beaconName: b.name,
                        beaconPrincipal: b.principal,
                        ownerName: s.owner?.name,
                        hostId: hId,
                        hostName: hName,
                        isCurrentHost: isCurr,
                    });
                }
                for (const pe of s.pivots?.edges || []) {
                    const p = pe?.node;
                    if (!p?.id) {
                        continue;
                    }
                    const pId = String(p.id);
                    if (!seenPivotIds.has(pId)) {
                        seenPivotIds.add(pId);
                        nearbyPivots.push({
                            id: pId,
                            createdAt: p.createdAt,
                            kind: p.kind,
                            destination: p.destination,
                            port: p.port,
                            closedAt: p.closedAt,
                            shellId: sId,
                            hostId: hId,
                            hostName: hName,
                            isCurrentHost: isCurr,
                        });
                    }
                }
            }
        }
    };

    collectShellsAndPivotsFromHost(currentHostNode, true);
    for (const h of rawHosts) {
        const hId = String(h.id);
        if (adjacentHostIds.has(hId)) {
            collectShellsAndPivotsFromHost(h, hId === currentIdStr);
        }
    }

    hostShells.sort((a, b) => {
        const aOpen = !a.closedAt;
        const bOpen = !b.closedAt;
        if (aOpen !== bOpen) {
            return aOpen ? -1 : 1;
        }
        if (Boolean(a.isCurrentHost) !== Boolean(b.isCurrentHost)) {
            return a.isCurrentHost ? -1 : 1;
        }
        return new Date(b.createdAt || 0).getTime() - new Date(a.createdAt || 0).getTime();
    });

    nearbyPivots.sort((a, b) => {
        const aOpen = !a.closedAt;
        const bOpen = !b.closedAt;
        if (aOpen !== bOpen) {
            return aOpen ? -1 : 1;
        }
        return new Date(b.createdAt || 0).getTime() - new Date(a.createdAt || 0).getTime();
    });

    return {
        hostCredentials,
        allCredentials,
        networkHosts: sortedHosts,
        hostFiles,
        hostShells,
        nearbyPivots,
    };
};

