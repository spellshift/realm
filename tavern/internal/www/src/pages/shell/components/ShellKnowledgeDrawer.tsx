import React, { useCallback, useMemo, useState } from "react";
import { MagnifyingGlassIcon, XMarkIcon } from "@heroicons/react/24/outline";
import { Shell, Unplug } from "lucide-react";
import { GraphStar } from "./ShellHeader";
import {
    KnowledgeCredentialItem,
    KnowledgeFileItem,
    KnowledgeHostItem,
    KnowledgeShellItem,
    ShellPivotItem,
} from "../hooks/useShellData";
import { formatSubnetCidr } from "../utils/networkGraphUtils";
import GraphPivotsSection from "./graph/GraphPivotsSection";
import GraphHostsSection from "./graph/GraphHostsSection";
import GraphShellsSection from "./graph/GraphShellsSection";
import GraphCredentialsSection from "./graph/GraphCredentialsSection";
import GraphFilesSection from "./graph/GraphFilesSection";

export interface ShellKnowledgeDrawerProps {
    isOpen: boolean;
    onClose: () => void;
    currentShellId?: string;
    currentHostId?: string;
    currentHostName?: string;
    currentHostIp?: string;
    portalId: number | null;
    pivots?: ShellPivotItem[];
    hostCredentials?: KnowledgeCredentialItem[];
    allCredentials: KnowledgeCredentialItem[];
    networkHosts: KnowledgeHostItem[];
    hostFiles: KnowledgeFileItem[];
    hostProcessCount?: number;
    hostShells: KnowledgeShellItem[];
    lastKnowledgeSyncAt?: number;
    onRefetchKnowledgeGraph: () => void;
    onNewPortal?: () => void;
    onClosePortal?: () => void;
    onOpenPty?: () => void;
    onOpenSshModal?: () => void;
    onOpenPivotTab?: (type: "ssh" | "pty", target: string, pivotId: string) => void;
    onOpenSshWithTarget: (target: string, credentialId?: string) => void;
    onRenamePivot?: (pivotId: string, newTitle: string) => void;
    onResetPivotName?: (pivotId: string) => void;
    onLoadIntoComposer?: (code: string) => void;
    onStageToPrompt?: (code: string) => void;
}

type GraphSectionFilter = "pivots" | "shells" | "hosts" | "creds" | "files";

const SECTION_TABS: { key: GraphSectionFilter; label: string }[] = [
    { key: "pivots", label: "Pivots" },
    { key: "shells", label: "Shells" },
    { key: "hosts", label: "Hosts" },
    { key: "creds", label: "Creds" },
    { key: "files", label: "Files" },
];

const HeaderPortalButton: React.FC<{
    portalId: number | null;
    onNewPortal?: () => void;
    onClosePortal?: () => void;
}> = ({ portalId, onNewPortal, onClosePortal }) => {
    if (portalId) {
        return (
            <button
                type="button"
                data-tour="shell-new-portal"
                onClick={onClosePortal}
                title={`Close portal #${portalId}`}
                aria-label="Close portal"
                className="p-1 rounded-md text-red-400 hover:text-red-300 hover:bg-gray-800 transition-colors"
            >
                <Unplug className="w-4 h-4" />
            </button>
        );
    }

    return (
        <button
            type="button"
            data-tour="shell-new-portal"
            onClick={onNewPortal}
            title="Open portal"
            aria-label="Open portal"
            className="p-1 rounded-md text-gray-400 hover:text-white hover:bg-gray-800 transition-colors"
        >
            <Shell className="w-4 h-4" />
        </button>
    );
};

export const ShellKnowledgeDrawer: React.FC<ShellKnowledgeDrawerProps> = ({
    isOpen,
    onClose,
    currentShellId,
    currentHostId,
    currentHostName,
    currentHostIp,
    portalId,
    pivots = [],
    allCredentials,
    networkHosts,
    hostFiles,
    hostShells,
    onRefetchKnowledgeGraph,
    onNewPortal,
    onClosePortal,
    onOpenPty,
    onOpenSshModal,
    onOpenPivotTab,
    onOpenSshWithTarget,
    onRenamePivot,
    onResetPivotName,
}) => {
    const [activeSection, setActiveSection] = useState<GraphSectionFilter>("pivots");
    const [searchQuery, setSearchQuery] = useState("");
    const [savedHostsById, setSavedHostsById] = useState<Record<string, KnowledgeHostItem>>({});
    const [localCredentials, setLocalCredentials] = useState<KnowledgeCredentialItem[]>([]);

    const normalizedQuery = searchQuery.trim().toLowerCase();

    const mergedHosts = useMemo(() => {
        const result: KnowledgeHostItem[] = [];
        const seenIds = new Set<string>();

        for (const h of networkHosts) {
            seenIds.add(h.id);
            if (h.networkHostId) {
                seenIds.add(`nh-${h.networkHostId}`);
            }
            const override =
                savedHostsById[h.id] ||
                (h.networkHostId ? savedHostsById[`nh-${h.networkHostId}`] : undefined);
            if (override) {
                const credById = new Map<string, KnowledgeCredentialItem>();
                for (const c of h.credentials || []) {
                    credById.set(c.id, c);
                }
                for (const c of override.credentials || []) {
                    credById.set(c.id, c);
                }
                const mergedCreds = Array.from(credById.values());
                result.push({
                    ...h,
                    ...override,
                    id: h.id,
                    hostId: h.hostId || override.hostId,
                    networkHostId: h.networkHostId || override.networkHostId,
                    credentials: mergedCreds,
                    credentialCount: mergedCreds.length,
                });
            } else {
                result.push(h);
            }
        }

        for (const [id, savedHost] of Object.entries(savedHostsById)) {
            const nhKey = savedHost.networkHostId ? `nh-${savedHost.networkHostId}` : undefined;
            if (!seenIds.has(id) && (!nhKey || !seenIds.has(nhKey))) {
                seenIds.add(id);
                if (nhKey) {
                    seenIds.add(nhKey);
                }
                result.push(savedHost);
            }
        }

        return result;
    }, [networkHosts, savedHostsById]);

    const mergedCredentials = useMemo(() => {
        const byId = new Map<string, KnowledgeCredentialItem>();
        for (const c of allCredentials) {
            byId.set(c.id, c);
        }
        for (const savedHost of Object.values(savedHostsById)) {
            for (const c of savedHost.credentials || []) {
                byId.set(c.id, c);
            }
        }
        for (const c of localCredentials) {
            byId.set(c.id, c);
        }
        return Array.from(byId.values());
    }, [allCredentials, savedHostsById, localCredentials]);

    const handleHostSaved = useCallback(
        (savedHost: KnowledgeHostItem) => {
            setSavedHostsById((prev) => ({
                ...prev,
                [savedHost.id]: savedHost,
            }));
            onRefetchKnowledgeGraph();
        },
        [onRefetchKnowledgeGraph]
    );

    const handleCredentialCreated = useCallback(
        (createdCred: KnowledgeCredentialItem, targetHostId?: string) => {
            setLocalCredentials((prev) => [
                ...prev.filter((c) => c.id !== createdCred.id),
                createdCred,
            ]);
            if (targetHostId) {
                setSavedHostsById((prev) => {
                    const existing =
                        prev[targetHostId] ||
                        networkHosts.find(
                            (h) =>
                                h.id === targetHostId ||
                                (h.networkHostId && `nh-${h.networkHostId}` === targetHostId) ||
                                (h.networkHostId && h.networkHostId === targetHostId)
                        );
                    if (!existing) {
                        return prev;
                    }
                    const nextCreds = [
                        ...(existing.credentials || []).filter((c) => c.id !== createdCred.id),
                        createdCred,
                    ];
                    return {
                        ...prev,
                        [existing.id]: {
                            ...existing,
                            credentials: nextCreds,
                            credentialCount: nextCreds.length,
                        },
                    };
                });
            }
            onRefetchKnowledgeGraph();
        },
        [networkHosts, onRefetchKnowledgeGraph]
    );

    const filteredPivots = useMemo(() => {
        if (!normalizedQuery) {
            return pivots;
        }
        return pivots.filter(
            (p) =>
                p.destination?.toLowerCase().includes(normalizedQuery) ||
                p.activeTarget?.toLowerCase().includes(normalizedQuery) ||
                p.kind?.toLowerCase().includes(normalizedQuery) ||
                p.hostName?.toLowerCase().includes(normalizedQuery)
        );
    }, [pivots, normalizedQuery]);

    const filteredHosts = useMemo(() => {
        if (!normalizedQuery) {
            return mergedHosts;
        }
        return mergedHosts.filter(
            (h) =>
                h.name?.toLowerCase().includes(normalizedQuery) ||
                h.hostname?.toLowerCase().includes(normalizedQuery) ||
                h.ipAddress?.toLowerCase().includes(normalizedQuery) ||
                h.primaryIP?.toLowerCase().includes(normalizedQuery) ||
                h.subnet?.toLowerCase().includes(normalizedQuery) ||
                (h.openPorts || []).some((port) => String(port).includes(normalizedQuery)) ||
                (h.credentials || []).some((c) =>
                    c.principal?.toLowerCase().includes(normalizedQuery)
                )
        );
    }, [mergedHosts, normalizedQuery]);

    const filteredShells = useMemo(() => {
        if (!normalizedQuery) {
            return hostShells;
        }
        return hostShells.filter(
            (s) =>
                s.id.toLowerCase().includes(normalizedQuery) ||
                s.beaconName?.toLowerCase().includes(normalizedQuery) ||
                s.beaconPrincipal?.toLowerCase().includes(normalizedQuery) ||
                s.hostName?.toLowerCase().includes(normalizedQuery)
        );
    }, [hostShells, normalizedQuery]);

    const filteredCredentials = useMemo(() => {
        if (!normalizedQuery) {
            return mergedCredentials;
        }
        return mergedCredentials.filter(
            (c) =>
                c.principal?.toLowerCase().includes(normalizedQuery) ||
                c.hostName?.toLowerCase().includes(normalizedQuery) ||
                c.hostPrimaryIP?.toLowerCase().includes(normalizedQuery)
        );
    }, [mergedCredentials, normalizedQuery]);

    const filteredFiles = useMemo(() => {
        if (!normalizedQuery) {
            return hostFiles;
        }
        return hostFiles.filter(
            (f) =>
                f.path?.toLowerCase().includes(normalizedQuery) ||
                f.hostName?.toLowerCase().includes(normalizedQuery)
        );
    }, [hostFiles, normalizedQuery]);

    if (!isOpen) {
        return null;
    }

    const currentSubnet = formatSubnetCidr(currentHostIp);
    const showPivots = activeSection === "pivots";
    const showShells = activeSection === "shells";
    const showHosts = activeSection === "hosts";
    const showCreds = activeSection === "creds";
    const showFiles = activeSection === "files";

    return (
        <aside
            className="w-full md:w-96 max-w-full bg-gray-900 border-l border-gray-800 flex flex-col h-full select-none z-20 shrink-0"
            aria-label="Knowledge graph and pivots drawer"
            data-testid="shell-knowledge-drawer"
        >
            {/* Header */}
            <div className="px-3 py-2 border-b border-gray-800 flex items-center justify-between bg-gray-900">
                <div className="min-w-0">
                    <h2 className="text-xs font-semibold uppercase tracking-wider text-gray-200 flex items-center gap-1.5">
                        <GraphStar className="w-4 h-4 text-gray-400" />
                        <span>Context</span>
                    </h2>
                    <div className="text-xs text-gray-400 font-mono truncate mt-0.5">
                        {currentHostName || "Current Host"}
                        {currentHostIp ? ` • ${currentHostIp}` : ""}
                        {currentSubnet ? ` (${currentSubnet})` : ""}
                    </div>
                </div>
                <div className="flex items-center gap-1">
                    <HeaderPortalButton
                        portalId={portalId}
                        onNewPortal={onNewPortal}
                        onClosePortal={onClosePortal}
                    />
                    <button
                        type="button"
                        onClick={onClose}
                        aria-label="Close knowledge drawer"
                        className="p-1 rounded-md text-gray-400 hover:text-white hover:bg-gray-800 transition-colors"
                    >
                        <XMarkIcon className="w-4 h-4" />
                    </button>
                </div>
            </div>

            {/* Filter Input & Section Tabs */}
            <div className="px-3 pt-2 border-b border-gray-800 bg-gray-900">
                <div className="relative mb-2">
                    <MagnifyingGlassIcon className="w-4 h-4 text-gray-400 absolute left-2.5 top-1/2 -translate-y-1/2 pointer-events-none" />
                    <input
                        type="text"
                        value={searchQuery}
                        onChange={(e) => setSearchQuery(e.target.value)}
                        placeholder="Filter hosts, pivots, creds, files..."
                        aria-label="Filter graph items"
                        className="w-full bg-gray-800 border border-gray-700 rounded-md pl-8 pr-2.5 py-1.5 text-xs text-gray-200 placeholder-gray-400 focus:outline-none focus:border-purple-500"
                    />
                </div>
                <div className="flex items-center gap-3 overflow-x-auto no-scrollbar">
                    {SECTION_TABS.map((tab) => {
                        const isActive = activeSection === tab.key;
                        const tabClass = isActive
                            ? "border-purple-500 text-white"
                            : "border-transparent text-gray-400 hover:text-gray-200";
                        return (
                            <button
                                key={tab.key}
                                type="button"
                                onClick={() => setActiveSection(tab.key)}
                                className={`pb-2 text-xs font-semibold border-b-2 transition-colors whitespace-nowrap ${tabClass}`}
                            >
                                {tab.label}
                            </button>
                        );
                    })}
                </div>
            </div>

            {/* Unified Minimalist Feed */}
            <div className="flex-1 overflow-y-auto select-text">
                {showPivots && (
                    <GraphPivotsSection
                        portalId={portalId}
                        pivots={filteredPivots}
                        onNewPortal={onNewPortal}
                        onClosePortal={onClosePortal}
                        onOpenPty={onOpenPty}
                        onOpenSshModal={onOpenSshModal}
                        onOpenSshWithTarget={onOpenSshWithTarget}
                        onOpenPivotTab={onOpenPivotTab}
                        onRenamePivot={onRenamePivot}
                        onResetPivotName={onResetPivotName}
                    />
                )}
                {showShells && (
                    <GraphShellsSection
                        shells={filteredShells}
                        currentShellId={currentShellId}
                    />
                )}
                {showHosts && (
                    <GraphHostsSection
                        hosts={filteredHosts}
                        availableCredentials={mergedCredentials}
                        onOpenSshWithTarget={onOpenSshWithTarget}
                        onHostSaved={handleHostSaved}
                    />
                )}
                {showCreds && (
                    <GraphCredentialsSection
                        credentials={filteredCredentials}
                        currentHostId={currentHostId}
                        currentHostName={currentHostName}
                        currentHostIp={currentHostIp}
                        hosts={mergedHosts}
                        onOpenSshWithTarget={onOpenSshWithTarget}
                        onCredentialCreated={handleCredentialCreated}
                    />
                )}
                {showFiles && <GraphFilesSection files={filteredFiles} />}
            </div>
        </aside>
    );
};

export default ShellKnowledgeDrawer;
