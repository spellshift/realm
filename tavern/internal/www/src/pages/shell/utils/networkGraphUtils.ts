import {
    KnowledgeBeaconItem,
    KnowledgeCredentialItem,
    KnowledgeFileItem,
    KnowledgeHostItem,
    KnowledgeShellItem,
    ShellPivotItem,
} from "../hooks/useShellData";
import { selectBestBeaconId } from "../../../components/create-shell-button/CreateShellButton";

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

export const isMatchingPivotTarget = (
    type: string,
    pendingTarget?: string,
    incomingTarget?: string
): boolean => {
    if (!pendingTarget || !incomingTarget) {
        return true;
    }
    if (type.toLowerCase() === "pty") {
        return true;
    }
    if (pendingTarget === incomingTarget) {
        return true;
    }

    const stripUser = (s: string) => (s.includes("@") ? s.split("@").pop() || s : s).trim();
    const cleanPending = stripUser(pendingTarget);
    const cleanIncoming = stripUser(incomingTarget);
    if (cleanPending === cleanIncoming) {
        return true;
    }

    const stripPort22 = (s: string) => s.replace(/:22$/, "");
    if (stripPort22(cleanPending) === stripPort22(cleanIncoming)) {
        return true;
    }

    const hostPending = extractPivotTargetHost(pendingTarget);
    const hostIncoming = extractPivotTargetHost(incomingTarget);
    if (hostPending && hostIncoming && hostPending.toLowerCase() === hostIncoming.toLowerCase()) {
        return true;
    }

    return false;
};

export function formatPivotTabLabel(tab: {
    type: string;
    target: string;
    customTitle?: string;
}): string {
    if (tab.customTitle && tab.customTitle.trim()) {
        return tab.customTitle.trim();
    }
    if (tab.type.toLowerCase() === "ssh") {
        const raw = (tab.target || "").trim();
        if (!raw) return "SSH";
        const afterAt = raw.includes("@") ? raw.slice(raw.lastIndexOf("@") + 1) : raw;
        const hostOnly = afterAt.replace(/:\d+$/, "");
        return hostOnly || afterAt || raw;
    }
    return "PTY";
}

const ESC_CHAR = String.fromCharCode(27);
const ANSI_ESCAPE_REGEX = new RegExp(`${ESC_CHAR}\\[[0-9;?]*[ -/]*[@-~]`, "g");

export const stripAnsi = (value?: string | null): string =>
    String(value || "").replace(ANSI_ESCAPE_REGEX, "");

export const hasPivotActivity = (
    pivot?: {
        closedAt?: string | null;
        data?: string | null;
    } | null
): boolean => {
    if (!pivot) {
        return false;
    }
    if (!pivot.closedAt) {
        return true;
    }
    if (pivot.data === undefined) {
        return true;
    }
    const cleaned = stripAnsi(pivot.data).trim();
    if (!cleaned) {
        return false;
    }
    const nonEmptyLines = cleaned
        .split(/\r?\n/)
        .map((line) => line.trim())
        .filter(Boolean);
    if (nonEmptyLines.length === 0) {
        return false;
    }
    if (nonEmptyLines.every((line) => line.startsWith("[-]"))) {
        return false;
    }
    return true;
};

export const isBeaconOnline = (beacon: any, nowMs: number = Date.now()): boolean => {
    if (!beacon) {
        return false;
    }
    const intervalSec = Number(beacon.interval);
    const intervalMs =
        Number.isFinite(intervalSec) && intervalSec > 0 ? intervalSec * 1000 : 60_000;
    if (beacon.nextSeenAt) {
        const nextMs = new Date(beacon.nextSeenAt).getTime();
        if (!Number.isFinite(nextMs)) {
            return false;
        }
        const graceMs = Math.max(intervalMs, 30_000);
        return nowMs - nextMs <= graceMs;
    }
    if (beacon.lastSeenAt) {
        const lastMs = new Date(beacon.lastSeenAt).getTime();
        if (!Number.isFinite(lastMs)) {
            return false;
        }
        const maxStaleMs = Math.max(intervalMs * 3, 120_000);
        return nowMs - lastMs <= maxStaleMs;
    }
    if ("lastSeenAt" in beacon || "nextSeenAt" in beacon) {
        return false;
    }
    return true;
};

export const extractBeaconsFromHost = (host: any): KnowledgeBeaconItem[] => {
    const hId = host?.id ? String(host.id) : undefined;
    const hName = host?.name || host?.primaryIP || hId;
    return (host?.beacons?.edges || [])
        .map((e: any) => e?.node)
        .filter((b: any) => Boolean(b?.id))
        .map((b: any) => ({
            id: String(b.id),
            name: b.name || `Beacon #${b.id}`,
            principal: b.principal,
            transport: b.transport,
            interval: b.interval,
            lastSeenAt: b.lastSeenAt,
            nextSeenAt: b.nextSeenAt,
            isOnline: isBeaconOnline(b),
            hostId: hId,
            hostName: hName,
        }));
};

export const selectPrimaryBeaconId = (host: any): string | undefined => {
    const beacons = extractBeaconsFromHost(host);
    if (beacons.length === 0) {
        return undefined;
    }
    const onlineBeacons = beacons.filter((b) => b.isOnline);
    if (onlineBeacons.length === 0) {
        return undefined;
    }
    const bestId = selectBestBeaconId(
        onlineBeacons.map((b) => ({
            id: b.id,
            principal: b.principal || "",
            interval: Number(b.interval) || Number.MAX_SAFE_INTEGER,
            lastSeenAt: b.lastSeenAt || new Date(0).toISOString(),
            nextSeenAt: b.nextSeenAt,
            transport: b.transport,
        }))
    );
    return bestId ? String(bestId) : undefined;
};

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
    currentHostBeacons: KnowledgeBeaconItem[];
}

export const buildUnifiedGraphData = ({
    currentHostNode,
    hostId,
    currentShellId,
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
        if (!hasPivotActivity(p)) {
            continue;
        }
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

    // 3. Map Host entities by ID, IP (primary & external), and normalized Name so NetworkHost records merge cleanly
    const normalizeHostNameKey = (rawName?: string | null): string =>
        String(rawName || "").trim().toLowerCase();
    const shortHostNameKey = (rawName?: string | null): string => {
        const normalized = normalizeHostNameKey(rawName);
        if (!normalized) return "";
        return normalized.split(".")[0];
    };

    const hostById = new Map<string, any>();
    const hostByIp = new Map<string, any>();
    const hostByName = new Map<string, any>();

    const registerHostLookup = (h: any) => {
        if (!h) return;
        if (h.id) {
            hostById.set(String(h.id), h);
        }
        if (h.primaryIP) {
            hostByIp.set(String(h.primaryIP).trim(), h);
        }
        if (h.externalIP) {
            hostByIp.set(String(h.externalIP).trim(), h);
        }
        const fullName = normalizeHostNameKey(h.name);
        if (fullName) {
            hostByName.set(fullName, h);
            const shortName = shortHostNameKey(h.name);
            if (shortName && !hostByName.has(shortName)) {
                hostByName.set(shortName, h);
            }
        }
    };

    registerHostLookup(currentHostNode);
    for (const h of rawHosts) {
        registerHostLookup(h);
    }

    const mergedHostIds = new Set<string>();
    const mergedIps = new Set<string>();
    const mergedNames = new Set<string>();
    const unifiedHosts: KnowledgeHostItem[] = [];
    const unifiedHostIndexByKey = new Map<string, number>();

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

    const getCanonicalKeysForEntry = (entry: {
        hostId?: string;
        ip?: string;
        externalIp?: string;
        name?: string;
        hostname?: string;
    }): string[] => {
        const keys: string[] = [];
        if (entry.hostId) {
            keys.push(`id:${entry.hostId}`);
        }
        if (entry.ip) {
            keys.push(`ip:${entry.ip.trim()}`);
        }
        if (entry.externalIp) {
            keys.push(`ip:${entry.externalIp.trim()}`);
        }
        const n1 = normalizeHostNameKey(entry.name);
        if (n1) {
            keys.push(`name:${n1}`);
        }
        const n2 = normalizeHostNameKey(entry.hostname);
        if (n2) {
            keys.push(`name:${n2}`);
        }
        return keys;
    };

    const upsertUnifiedHost = (candidate: KnowledgeHostItem) => {
        const keys = getCanonicalKeysForEntry({
            hostId: candidate.hostId,
            ip: candidate.ipAddress || candidate.primaryIP,
            externalIp: candidate.externalIP,
            name: candidate.name,
            hostname: candidate.hostname,
        });

        let existingIdx: number | undefined;
        for (const key of keys) {
            const found = unifiedHostIndexByKey.get(key);
            if (found !== undefined) {
                existingIdx = found;
                break;
            }
        }

        if (candidate.hostId) mergedHostIds.add(candidate.hostId);
        if (candidate.primaryIP) mergedIps.add(candidate.primaryIP.trim());
        if (candidate.ipAddress) mergedIps.add(candidate.ipAddress.trim());
        if (candidate.externalIP) mergedIps.add(candidate.externalIP.trim());
        if (normalizeHostNameKey(candidate.name)) mergedNames.add(normalizeHostNameKey(candidate.name));
        if (normalizeHostNameKey(candidate.hostname)) {
            mergedNames.add(normalizeHostNameKey(candidate.hostname));
        }

        if (existingIdx === undefined) {
            const newIndex = unifiedHosts.length;
            unifiedHosts.push(candidate);
            for (const key of keys) {
                unifiedHostIndexByKey.set(key, newIndex);
            }
            return;
        }

        const existing = unifiedHosts[existingIdx];
        const credById = new Map<string, KnowledgeCredentialItem>();
        for (const c of existing.credentials || []) {
            credById.set(c.id, c);
        }
        for (const c of candidate.credentials || []) {
            credById.set(c.id, c);
        }
        const mergedCreds = Array.from(credById.values());

        const portSet = new Set<number>([...(existing.openPorts || []), ...(candidate.openPorts || [])]);
        const mergedPorts = Array.from(portSet).sort((a, b) => a - b);

        const netById = new Map<string, { id: string; name: string; subnet: string; description?: string }>();
        for (const n of existing.networks || []) {
            netById.set(n.id, n);
        }
        for (const n of candidate.networks || []) {
            netById.set(n.id, n);
        }

        const tagById = new Map<string, { id: string; name: string; kind: string }>();
        for (const t of existing.tags || []) {
            tagById.set(t.id, t);
        }
        for (const t of candidate.tags || []) {
            tagById.set(t.id, t);
        }

        const sharedTagSet = new Set<string>([
            ...(existing.sharedTags || []),
            ...(candidate.sharedTags || []),
        ]);

        const beaconById = new Map<string, KnowledgeBeaconItem>();
        for (const b of existing.beacons || []) {
            beaconById.set(b.id, b);
        }
        for (const b of candidate.beacons || []) {
            beaconById.set(b.id, b);
        }
        const mergedBeacons = Array.from(beaconById.values());

        const mergedItem: KnowledgeHostItem = {
            ...existing,
            id: existing.hostId || candidate.hostId || existing.id,
            hostId: existing.hostId || candidate.hostId,
            networkHostId: existing.networkHostId || candidate.networkHostId,
            name:
                existing.name && existing.name !== existing.primaryIP
                    ? existing.name
                    : candidate.name || existing.name,
            hostname: existing.hostname || candidate.hostname,
            primaryIP: existing.primaryIP || candidate.primaryIP,
            ipAddress: existing.ipAddress || candidate.ipAddress,
            subnet: existing.subnet || candidate.subnet,
            externalIP: existing.externalIP || candidate.externalIP,
            platform: existing.platform || candidate.platform,
            lastSeenAt: existing.lastSeenAt || candidate.lastSeenAt,
            openPorts: mergedPorts,
            networks: Array.from(netById.values()),
            tags: Array.from(tagById.values()),
            sharedTags: Array.from(sharedTagSet),
            beaconCount: Math.max(
                existing.beaconCount || 0,
                candidate.beaconCount || 0,
                mergedBeacons.length
            ),
            beacons: mergedBeacons,
            primaryBeaconId: existing.primaryBeaconId || candidate.primaryBeaconId,
            hasOnlineBeacon: Boolean(existing.hasOnlineBeacon || candidate.hasOnlineBeacon),
            credentials: mergedCreds,
            credentialCount: Math.max(
                existing.credentialCount || 0,
                candidate.credentialCount || 0,
                mergedCreds.length
            ),
            isCurrentHost: existing.isCurrentHost || candidate.isCurrentHost,
            sameSubnet: existing.sameSubnet || candidate.sameSubnet,
            isReachable: existing.isReachable || candidate.isReachable,
        };
        unifiedHosts[existingIdx] = mergedItem;
        for (const key of keys) {
            unifiedHostIndexByKey.set(key, existingIdx);
        }
    };

    // 4. Process NetworkHost records first and link to matching Host if present
    for (const nh of rawNetworkHosts) {
        const ipCandidate = String(nh.ipAddress || nh.host?.primaryIP || "").trim();
        const linkedHost =
            (nh.host?.id && hostById.get(String(nh.host.id))) ||
            nh.host ||
            (ipCandidate && hostByIp.get(ipCandidate)) ||
            (nh.hostname && hostByName.get(normalizeHostNameKey(nh.hostname))) ||
            (nh.name && hostByName.get(normalizeHostNameKey(nh.name))) ||
            (nh.hostname && hostByName.get(shortHostNameKey(nh.hostname))) ||
            (nh.name && hostByName.get(shortHostNameKey(nh.name))) ||
            null;

        const linkedHostId = linkedHost?.id ? String(linkedHost.id) : undefined;
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
        const beacons = extractBeaconsFromHost(linkedHost);
        const primaryBeaconId = selectPrimaryBeaconId(linkedHost);

        upsertUnifiedHost({
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
            beaconCount: linkedHost?.beacons?.totalCount ?? beacons.length,
            beacons,
            primaryBeaconId,
            hasOnlineBeacon: Boolean(primaryBeaconId),
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
        const ipAddr = String(h.primaryIP || "").trim();
        const extIp = String(h.externalIP || "").trim();
        const normName = normalizeHostNameKey(h.name);
        if (
            mergedHostIds.has(hId) ||
            (ipAddr && mergedIps.has(ipAddr)) ||
            (extIp && mergedIps.has(extIp)) ||
            (normName && mergedNames.has(normName))
        ) {
            continue;
        }
        const isCurrentHost = hId === currentIdStr;
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
        const beacons = extractBeaconsFromHost(h);
        const primaryBeaconId = selectPrimaryBeaconId(h);

        upsertUnifiedHost({
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
            beaconCount: h.beacons?.totalCount ?? beacons.length,
            beacons,
            primaryBeaconId,
            hasOnlineBeacon: Boolean(primaryBeaconId),
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
        if (!p?.id || !hasPivotActivity(p)) {
            continue;
        }
        const pId = String(p.id);
        seenPivotIds.add(pId);
        nearbyPivots.push({
            ...p,
            id: pId,
            shellId: p.shellId || currentShellId,
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
            const beaconOnline = isBeaconOnline(b);
            for (const se of b.shells?.edges || []) {
                const s = se?.node;
                if (!s?.id) {
                    continue;
                }
                const sId = String(s.id);
                const shellActiveUsers = (s.activeUsers?.edges || [])
                    .map((ue: any) => ue?.node)
                    .filter(Boolean)
                    .map((u: any) => ({
                        id: String(u.id),
                        name: u.name || "Operator",
                        photoURL: u.photoURL,
                    }));
                if (!seenShellIds.has(sId)) {
                    seenShellIds.add(sId);
                    const tasks = (s.shellTasks?.edges || [])
                        .map((te: any) => te?.node)
                        .filter(Boolean)
                        .map((t: any) => ({
                            id: t.id ? String(t.id) : undefined,
                            input: t.input,
                            output: t.output,
                            error: t.error,
                        }));
                    hostShells.push({
                        id: sId,
                        createdAt: s.createdAt,
                        closedAt: s.closedAt,
                        beaconId: String(b.id),
                        beaconName: b.name,
                        beaconPrincipal: b.principal,
                        beaconLastSeenAt: b.lastSeenAt,
                        beaconNextSeenAt: b.nextSeenAt,
                        beaconInterval: b.interval,
                        beaconOnline,
                        isLive: !s.closedAt && beaconOnline,
                        activeUsers: shellActiveUsers,
                        ownerId: s.owner?.id ? String(s.owner.id) : undefined,
                        ownerName: s.owner?.name,
                        hostId: hId,
                        hostName: hName,
                        isCurrentHost: isCurr,
                        tasks,
                    });
                }
                for (const pe of s.pivots?.edges || []) {
                    const p = pe?.node;
                    if (!p?.id || !hasPivotActivity(p)) {
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
                            data: p.data,
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
        const aLive = a.isLive ?? (!a.closedAt && (a.beaconOnline ?? true));
        const bLive = b.isLive ?? (!b.closedAt && (b.beaconOnline ?? true));
        if (aLive !== bLive) {
            return aLive ? -1 : 1;
        }
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
        currentHostBeacons: extractBeaconsFromHost(currentHostNode),
    };
};

