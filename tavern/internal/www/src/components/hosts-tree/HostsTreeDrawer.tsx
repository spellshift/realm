import React, { useEffect, useMemo, useState } from "react";
import { Tooltip } from "@chakra-ui/react";
import {
    ChevronDown,
    ChevronUp,
    CornerDownRight,
    KeyRound,
    Monitor,
    Network,
    Plus,
    SquareTerminal,
    Terminal,
    User,
    X,
    Zap,
} from "lucide-react";
import PlaceholderUser from "../../assets/PlaceholderUser.png";
import CredentialModal from "../../pages/shell/components/graph/CredentialModal";
import HostModal from "../../pages/shell/components/graph/HostModal";
import {
    KnowledgeBeaconItem,
    KnowledgeCredentialItem,
    KnowledgeHostItem,
    KnowledgeShellItem,
    ShellPivotItem,
    ShellTaskItem,
} from "../../pages/shell/hooks/useShellData";
import { formatPivotTabLabel } from "../../pages/shell/utils/networkGraphUtils";
import {
    EMPTY_HOSTS_FILTERS,
    HostsFilterPopover,
    HostsFilterState,
    hasActiveHostFilters,
} from "./HostsFilterPopover";
import {
    BuildHostsTreeParams,
    HostBeaconNode,
    HostTreeNode,
    HostsTreeSections,
    buildHostsTreeSections,
    compareHostTreeNodes,
    dedupePivotsById,
    filterHostsTreeSections,
    groupPivotsByShellId,
    hasPrivilegedHostAccess,
    pickBestBeaconIdForBeacons,
    pickPreferredShell,
} from "./hostsTreeUtils";

export type {
    BuildHostsTreeParams,
    HostBeaconNode,
    HostTreeNode,
    HostsTreeSections,
    HostsFilterState,
};
export { EMPTY_HOSTS_FILTERS, buildHostsTreeSections, filterHostsTreeSections };

export type HostsTreeTheme = "dark" | "light";
export type HostsTreeViewMode = "host" | "overview";

interface ThemePalette {
    container: string;
    headerBorder: string;
    headerTitle: string;
    breadcrumbLink: string;
    breadcrumbActive: string;
    breadcrumbSeparator: string;
    headerIcon: string;
    closeBtn: string;
    addBadgeBg: string;
    privilegedIcon: string;
    emptyText: string;
    clearFilterBtn: string;
    rowHover: string;
    rowTitle: string;
    rowSubtitle: string;
    hostIcon: string;
    currentSelectorBar: string;
    branchLine: string;
    shellTag: string;
    ptyTag: string;
    pivotTag: string;
    itemLabel: string;
    itemMeta: string;
    mutedItem: string;
    protocolBadge: string;
    transcriptTag: string;
    expandButton: string;
    avatarRing: string;
    statusOfflineText: string;
    statusOfflineDot: string;
    ptyButtonEnabled: string;
    ptyButtonDisabled: string;
    divider: string;
    footer: string;
}

const THEME_PALETTES: Record<HostsTreeTheme, ThemePalette> = {
    dark: {
        container: "bg-gray-900 border-gray-800 text-gray-200",
        headerBorder: "border-gray-800",
        headerTitle: "text-gray-200",
        breadcrumbLink: "text-purple-400 hover:text-purple-300",
        breadcrumbActive: "text-gray-100",
        breadcrumbSeparator: "text-gray-600",
        headerIcon: "text-gray-400",
        closeBtn: "text-gray-400 hover:text-white hover:bg-gray-800",
        addBadgeBg: "bg-gray-900",
        privilegedIcon: "text-amber-400 fill-amber-400",
        emptyText: "text-gray-400",
        clearFilterBtn: "text-purple-400 hover:text-purple-300",
        rowHover: "hover:bg-gray-800/60",
        rowTitle: "text-gray-100",
        rowSubtitle: "text-gray-400",
        hostIcon: "text-gray-400",
        currentSelectorBar: "bg-purple-400",
        branchLine: "border-gray-700/80",
        shellTag: "text-purple-400",
        ptyTag: "text-purple-300",
        pivotTag: "text-purple-400",
        itemLabel: "text-gray-200",
        itemMeta: "text-gray-500",
        mutedItem: "text-gray-400",
        protocolBadge: "bg-gray-800 text-gray-300 border border-gray-700/80",
        transcriptTag: "bg-purple-500/15 text-purple-300",
        expandButton: "text-gray-400 hover:text-gray-200 hover:bg-gray-800/50",
        avatarRing: "ring-gray-900",
        statusOfflineText: "text-red-400",
        statusOfflineDot: "bg-red-500",
        ptyButtonEnabled: "text-gray-300 hover:text-white hover:bg-gray-800",
        ptyButtonDisabled: "text-gray-600 opacity-50 cursor-not-allowed",
        divider: "border-gray-800/80",
        footer: "text-gray-500 border-gray-800",
    },
    light: {
        container: "bg-white border-gray-200 text-gray-900",
        headerBorder: "border-gray-200",
        headerTitle: "text-gray-900",
        breadcrumbLink: "text-purple-700 hover:text-purple-800",
        breadcrumbActive: "text-gray-900",
        breadcrumbSeparator: "text-gray-400",
        headerIcon: "text-gray-600",
        closeBtn: "text-gray-500 hover:text-gray-900 hover:bg-gray-100",
        addBadgeBg: "bg-white",
        privilegedIcon: "text-amber-500 fill-amber-500",
        emptyText: "text-gray-500",
        clearFilterBtn: "text-purple-700 hover:text-purple-800",
        rowHover: "hover:bg-gray-100",
        rowTitle: "text-gray-900",
        rowSubtitle: "text-gray-500",
        hostIcon: "text-gray-500",
        currentSelectorBar: "bg-purple-600",
        branchLine: "border-gray-300",
        shellTag: "text-purple-700",
        ptyTag: "text-purple-700",
        pivotTag: "text-purple-700",
        itemLabel: "text-gray-800",
        itemMeta: "text-gray-500",
        mutedItem: "text-gray-500",
        protocolBadge: "bg-gray-100 text-gray-700 border border-gray-200",
        transcriptTag: "bg-purple-100 text-purple-700",
        expandButton: "text-gray-500 hover:text-gray-800 hover:bg-gray-100",
        avatarRing: "ring-white",
        statusOfflineText: "text-red-600",
        statusOfflineDot: "bg-red-500",
        ptyButtonEnabled: "text-gray-600 hover:text-gray-900 hover:bg-gray-200/70",
        ptyButtonDisabled: "text-gray-400 opacity-50 cursor-not-allowed",
        divider: "border-gray-200",
        footer: "text-gray-500 border-gray-200",
    },
};

const ActiveUsersAvatars: React.FC<{
    users?: { id: string; name: string; photoURL?: string }[];
    ringClass: string;
}> = ({ users = [], ringClass }) => {
    if (users.length === 0) {
        return null;
    }

    return (
        <div
            className="flex -space-x-1 overflow-hidden shrink-0"
            data-testid="hosts-tree-shell-avatars"
        >
            {users.slice(0, 3).map((user) => (
                <Tooltip key={user.id} label={user.name} hasArrow>
                    <img
                        src={user.photoURL || PlaceholderUser}
                        alt={user.name}
                        onError={(e) => {
                            (e.target as HTMLImageElement).src = PlaceholderUser;
                        }}
                        className={`w-4 h-4 rounded-full ring-1 ${ringClass} bg-gray-200 object-cover`}
                    />
                </Tooltip>
            ))}
        </div>
    );
};

const HostStatusBadge: React.FC<{
    node: HostTreeNode;
    palette: ThemePalette;
}> = ({ node, palette }) => {
    if (!node.isCurrentHost || node.isOnline) {
        return null;
    }
    return (
        <span
            className={`inline-flex items-center gap-1.5 text-[11px] font-medium shrink-0 ${palette.statusOfflineText}`}
        >
            <span className={`w-1.5 h-1.5 rounded-full ${palette.statusOfflineDot}`} />
            <span>Offline</span>
        </span>
    );
};

const HostRowOpenPtyButton: React.FC<{
    portalOpen: boolean;
    palette: ThemePalette;
    onOpenPty?: () => void;
}> = ({ portalOpen, palette, onOpenPty }) => {
    const tooltipLabel = portalOpen
        ? "Open PTY"
        : "PTY unavailable in async mode, please create a portal first.";
    const buttonClasses = portalOpen ? palette.ptyButtonEnabled : palette.ptyButtonDisabled;

    return (
        <Tooltip label={tooltipLabel} hasArrow>
            <span className="inline-flex shrink-0">
                <button
                    type="button"
                    data-testid="hosts-tree-open-pty-button"
                    aria-label="Open PTY"
                    disabled={!portalOpen}
                    onClick={() => {
                        if (!portalOpen) {
                            return;
                        }
                        onOpenPty?.();
                    }}
                    className={`p-1 rounded-md transition-colors ${buttonClasses}`}
                >
                    <Terminal className="w-3.5 h-3.5" />
                </button>
            </span>
        </Tooltip>
    );
};

const PivotTreeItemRow: React.FC<{
    pivot: ShellPivotItem;
    closed?: boolean;
    isClosing?: boolean;
    palette: ThemePalette;
    onClick: () => void;
    onClosePivot?: () => void;
}> = ({ pivot, closed = false, isClosing = false, palette, onClick, onClosePivot }) => {
    const isPty = String(pivot.kind || "").toLowerCase() === "pty";
    const protocolLabel = String(pivot.kind || "SSH").trim().toUpperCase() || "SSH";
    const label = formatPivotTabLabel({
        type: pivot.kind || "ssh",
        target: pivot.activeTarget || pivot.destination,
        customTitle: pivot.customTitle,
    });
    const customPtyTitle = pivot.customTitle?.trim();
    const displayTarget = isPty
        ? customPtyTitle || "PTY"
        : pivot.destination || pivot.activeTarget || label;
    const showProtocolBadge =
        !isPty || (Boolean(customPtyTitle) && customPtyTitle!.toUpperCase() !== protocolLabel);

    const icon = isPty ? (
        <SquareTerminal
            className={`w-3.5 h-3.5 shrink-0 ${closed ? `opacity-60 ${palette.mutedItem}` : palette.ptyTag}`}
        />
    ) : (
        <CornerDownRight
            className={`w-3.5 h-3.5 shrink-0 ${closed ? `opacity-60 ${palette.mutedItem}` : palette.pivotTag}`}
        />
    );

    const targetClass = closed
        ? `truncate font-mono ${palette.mutedItem}`
        : isPty && !customPtyTitle
          ? `truncate font-semibold ${palette.ptyTag}`
          : `truncate font-mono ${palette.itemLabel}`;

    const content = (
        <span className="flex items-center gap-1.5 min-w-0">
            {icon}
            <span className={targetClass}>{displayTarget}</span>
            {showProtocolBadge && (
                <span
                    data-testid={`hosts-tree-pivot-protocol-${pivot.id}`}
                    className={`inline-flex items-center justify-center leading-none pt-[2.5px] pb-[1.5px] px-1.5 text-[10px] font-medium rounded uppercase shrink-0 ${palette.protocolBadge} ${
                        closed ? "opacity-75" : ""
                    }`}
                >
                    {protocolLabel}
                </span>
            )}
        </span>
    );

    if (!closed && onClosePivot) {
        return (
            <div
                className={`w-full flex items-center justify-between gap-1.5 px-1.5 py-0.5 rounded text-xs transition-colors ${palette.rowHover}`}
            >
                <button
                    type="button"
                    data-testid={`hosts-tree-pivot-${pivot.id}`}
                    onClick={onClick}
                    className="flex items-center gap-1.5 min-w-0 flex-1 text-left"
                >
                    {content}
                </button>
                <button
                    type="button"
                    data-testid={`hosts-tree-close-pivot-${pivot.id}`}
                    aria-label={isPty ? "Close PTY" : `Close pivot ${displayTarget}`}
                    title={isPty ? "Close PTY" : "Close pivot"}
                    disabled={isClosing}
                    onClick={(e) => {
                        e.stopPropagation();
                        onClosePivot();
                    }}
                    className={`p-0.5 rounded transition-colors shrink-0 ${palette.closeBtn} ${
                        isClosing ? "opacity-50 cursor-not-allowed" : ""
                    }`}
                >
                    <X className="w-3.5 h-3.5" />
                </button>
            </div>
        );
    }

    return (
        <button
            type="button"
            data-testid={closed ? `hosts-tree-closed-pivot-${pivot.id}` : `hosts-tree-pivot-${pivot.id}`}
            onClick={onClick}
            className={`w-full flex items-center justify-between gap-2 px-1.5 py-0.5 rounded text-left text-xs transition-colors ${palette.rowHover}`}
        >
            {content}
            {closed && (
                <span className={`inline-flex items-center justify-center leading-none pt-[2px] pb-[1.5px] px-1.5 text-[10px] font-medium rounded shrink-0 ${palette.transcriptTag}`}>
                    Transcript
                </span>
            )}
        </button>
    );
};

const ShellTreeItemRow: React.FC<{
    shell: KnowledgeShellItem;
    closed?: boolean;
    isCurrent?: boolean;
    showPrincipal?: boolean;
    palette: ThemePalette;
    onSelectShell: (shell: KnowledgeShellItem) => void;
}> = ({ shell, closed = false, isCurrent = false, showPrincipal = false, palette, onSelectShell }) => {
    const hasBeaconName = Boolean(shell.beaconName);
    const hasPrincipal = Boolean(shell.beaconPrincipal);
    const primaryLabel = hasBeaconName
        ? shell.beaconName
        : showPrincipal && hasPrincipal
          ? null
          : shell.beaconPrincipal || (closed ? "Closed" : "Active");

    return (
        <button
            type="button"
            data-testid={closed ? `hosts-tree-closed-shell-${shell.id}` : `hosts-tree-shell-${shell.id}`}
            onClick={() => onSelectShell(shell)}
            className={`w-full flex items-center justify-between gap-2 px-1.5 py-0.5 rounded text-left text-xs transition-colors ${palette.rowHover}`}
        >
            <span className="flex items-center gap-1.5 min-w-0">
                <Terminal
                    className={`w-3.5 h-3.5 shrink-0 ${closed ? `opacity-60 ${palette.mutedItem}` : palette.shellTag}`}
                />
                {primaryLabel && (
                    <span className={`truncate font-mono ${closed ? palette.mutedItem : palette.itemLabel}`}>
                        {primaryLabel}
                    </span>
                )}
                {showPrincipal && hasPrincipal && (
                    <span
                        data-testid={`hosts-tree-shell-principal-${shell.id}`}
                        className={`inline-flex items-center gap-1 min-w-0 truncate ${
                            closed ? palette.mutedItem : palette.rowSubtitle
                        }`}
                    >
                        <User className="w-3 h-3 shrink-0" />
                        <span className="truncate">{shell.beaconPrincipal}</span>
                    </span>
                )}
            </span>
            <span className="flex items-center gap-2 shrink-0">
                <ActiveUsersAvatars users={shell.activeUsers} ringClass={palette.avatarRing} />
                {!closed && isCurrent && (
                    <span className={`text-[11px] ${palette.itemMeta}`}>active</span>
                )}
                {closed && (
                    <span className={`inline-flex items-center justify-center leading-none pt-[2px] pb-[1.5px] px-1.5 text-[10px] font-medium rounded ${palette.transcriptTag}`}>
                        Transcript
                    </span>
                )}
            </span>
        </button>
    );
};

const AvailableHostRow: React.FC<{
    availHost: HostTreeNode;
    palette: ThemePalette;
    onSelectAvailableHost?: (hostNode: HostTreeNode) => void;
}> = ({ availHost, palette, onSelectAvailableHost }) => {
    const ports = availHost.sourceHost?.openPorts || [];
    const primaryPort = ports.includes(22) ? 22 : ports[0];
    const showIp = Boolean(availHost.ip && availHost.ip !== availHost.name);

    return (
        <button
            type="button"
            data-testid={`hosts-tree-available-host-${availHost.key}`}
            onClick={() => onSelectAvailableHost?.(availHost)}
            className={`w-full flex items-center justify-between gap-2 px-1.5 py-0.5 rounded text-left text-xs transition-colors ${palette.rowHover}`}
        >
            <span className="flex items-center gap-1.5 min-w-0">
                <Monitor className={`w-3.5 h-3.5 shrink-0 ${palette.hostIcon}`} />
                <span className={`truncate font-medium ${palette.rowTitle}`}>{availHost.name}</span>
            </span>
            {(showIp || primaryPort !== undefined) && (
                <span className="flex items-center shrink-0 font-mono text-[11px]">
                    {showIp && (
                        <span
                            data-testid={`hosts-tree-host-ip-${availHost.key}`}
                            className={palette.rowSubtitle}
                        >
                            {availHost.ip}
                        </span>
                    )}
                    {primaryPort !== undefined && (
                        <span className={palette.itemMeta}>:{primaryPort}</span>
                    )}
                </span>
            )}
        </button>
    );
};

const ClosedSectionToggle: React.FC<{
    testId: string;
    labelSubject: string;
    count: number;
    isExpanded: boolean;
    onToggle: () => void;
    palette: ThemePalette;
    children: React.ReactNode;
}> = ({ testId, labelSubject, count, isExpanded, onToggle, palette, children }) => {
    if (count === 0) {
        return null;
    }
    return (
        <div className="space-y-0.5">
            <button
                type="button"
                data-testid={testId}
                aria-expanded={isExpanded}
                aria-label={
                    isExpanded
                        ? `Collapse closed sessions for ${labelSubject}`
                        : `Show ${count} closed sessions for ${labelSubject}`
                }
                onClick={(e) => {
                    e.stopPropagation();
                    onToggle();
                }}
                className={`w-full flex items-center gap-1.5 px-1.5 py-0.5 rounded text-[11px] font-medium transition-colors ${palette.expandButton}`}
            >
                {isExpanded ? (
                    <ChevronUp className="w-3.5 h-3.5 shrink-0" />
                ) : (
                    <ChevronDown className="w-3.5 h-3.5 shrink-0" />
                )}
                <span>{isExpanded ? `Hide closed (${count})` : `${count} closed`}</span>
            </button>

            {isExpanded && <div className="space-y-0.5">{children}</div>}
        </div>
    );
};

const useSearchableClosedToggle = (
    hasActiveFilter: boolean,
    hasMatchingClosed: boolean,
    filterResetKey: string
) => {
    const [isExpanded, setIsExpanded] = useState(false);
    const [searchOverride, setSearchOverride] = useState<boolean | null>(null);

    useEffect(() => {
        setSearchOverride(null);
    }, [filterResetKey]);

    const effectiveExpanded =
        hasActiveFilter && hasMatchingClosed ? (searchOverride ?? true) : isExpanded;

    const toggle = () => {
        if (hasActiveFilter && hasMatchingClosed) {
            setSearchOverride((prev) => !(prev ?? true));
            return;
        }
        setIsExpanded((prev) => !prev);
    };

    return { effectiveExpanded, toggle };
};

const HostDivider: React.FC<{
    testId: string;
    label: string;
    palette: ThemePalette;
}> = ({ testId, label, palette }) => (
    <div data-testid={testId} className="flex items-center gap-2 pt-1 pb-0.5">
        <div className={`flex-1 border-t ${palette.divider}`} />
        <span className={`text-[10px] font-semibold uppercase tracking-wider shrink-0 ${palette.itemMeta}`}>
            {label}
        </span>
        <div className={`flex-1 border-t ${palette.divider}`} />
    </div>
);

const SpecificHostSection: React.FC<{
    node: HostTreeNode;
    availableHosts: HostTreeNode[];
    currentShellId?: string;
    filters: HostsFilterState;
    hasActiveFilter: boolean;
    filterResetKey: string;
    showOpenPtyButton: boolean;
    portalOpen: boolean;
    closingPivotIds?: string[];
    palette: ThemePalette;
    onOpenPty?: () => void;
    onSelectShell: (shell: KnowledgeShellItem) => void;
    onSelectPivot: (pivot: ShellPivotItem, hostNode: HostTreeNode) => void;
    onClosePivot?: (pivot: ShellPivotItem) => void;
    onSelectAvailableHost: (hostNode: HostTreeNode) => void;
}> = ({
    node,
    availableHosts,
    currentShellId,
    filters,
    hasActiveFilter,
    filterResetKey,
    showOpenPtyButton,
    portalOpen,
    closingPivotIds = [],
    palette,
    onOpenPty,
    onSelectShell,
    onSelectPivot,
    onClosePivot,
    onSelectAvailableHost,
}) => {
    const relevantActivePivots = useMemo(
        () => dedupePivotsById([...node.activePivots, ...node.outgoingActivePivots]),
        [node.activePivots, node.outgoingActivePivots]
    );
    const relevantClosedPivots = useMemo(
        () => dedupePivotsById([...node.closedPivots, ...node.outgoingClosedPivots]),
        [node.closedPivots, node.outgoingClosedPivots]
    );

    const hasIoFilter = Boolean(filters.input.trim() || filters.output.trim());
    const showMatchedShells =
        hasActiveFilter &&
        (hasIoFilter ||
            (node.activeShells.length > 0 &&
                relevantActivePivots.length === 0 &&
                availableHosts.length === 0));
    const visibleActiveShells = showMatchedShells ? node.activeShells : [];
    const visibleClosedShells = hasActiveFilter ? node.closedShells : [];

    const closedCount = visibleClosedShells.length + relevantClosedPivots.length;
    const { effectiveExpanded, toggle } = useSearchableClosedToggle(
        hasActiveFilter,
        Boolean(node.hasMatchingClosed),
        filterResetKey
    );

    return (
        <div data-testid="hosts-tree-current-section" className="space-y-1">
            <div
                data-testid={`hosts-tree-host-${node.key}`}
                className="flex items-center justify-between gap-2 px-1 py-0.5"
            >
                <div className="flex items-center gap-1.5 min-w-0">
                    <span className="inline-flex items-center gap-1 shrink-0">
                        <span
                            data-testid="hosts-tree-current-indicator"
                            aria-hidden="true"
                            className={`w-0.5 h-3.5 rounded-full ${palette.currentSelectorBar}`}
                        />
                        <Monitor
                            data-testid="hosts-tree-current-monitor-icon"
                            className={`w-4 h-4 ${palette.hostIcon}`}
                        />
                    </span>
                    <span className={`text-xs font-semibold truncate ${palette.rowTitle}`}>
                        {node.name}
                    </span>
                </div>
                <div className="flex items-center gap-1.5 shrink-0">
                    {node.ip && node.ip !== node.name && (
                        <span
                            data-testid={`hosts-tree-host-ip-${node.key}`}
                            className={`text-[11px] font-mono ${palette.rowSubtitle}`}
                        >
                            {node.ip}
                        </span>
                    )}
                    <HostStatusBadge node={node} palette={palette} />
                    {showOpenPtyButton && (
                        <HostRowOpenPtyButton
                            portalOpen={portalOpen}
                            palette={palette}
                            onOpenPty={onOpenPty}
                        />
                    )}
                </div>
            </div>

            <div className="space-y-0.5">
                {visibleActiveShells.map((shell) => (
                    <ShellTreeItemRow
                        key={`shell-${shell.id}`}
                        shell={shell}
                        isCurrent={Boolean(currentShellId && String(shell.id) === String(currentShellId))}
                        palette={palette}
                        onSelectShell={onSelectShell}
                    />
                ))}

                {relevantActivePivots.map((pivot) => (
                    <PivotTreeItemRow
                        key={`pivot-${pivot.id}`}
                        pivot={pivot}
                        isClosing={closingPivotIds.includes(String(pivot.id))}
                        palette={palette}
                        onClick={() => onSelectPivot(pivot, node)}
                        onClosePivot={onClosePivot ? () => onClosePivot(pivot) : undefined}
                    />
                ))}

                {availableHosts.map((availHost) => (
                    <AvailableHostRow
                        key={`available-host-${availHost.key}`}
                        availHost={availHost}
                        palette={palette}
                        onSelectAvailableHost={onSelectAvailableHost}
                    />
                ))}

                <ClosedSectionToggle
                    testId={`hosts-tree-toggle-closed-${node.key}`}
                    labelSubject={node.name}
                    count={closedCount}
                    isExpanded={effectiveExpanded}
                    onToggle={toggle}
                    palette={palette}
                >
                    {visibleClosedShells.map((shell) => (
                        <ShellTreeItemRow
                            key={`closed-shell-${shell.id}`}
                            shell={shell}
                            closed
                            palette={palette}
                            onSelectShell={onSelectShell}
                        />
                    ))}
                    {relevantClosedPivots.map((pivot) => (
                        <PivotTreeItemRow
                            key={`closed-pivot-${pivot.id}`}
                            pivot={pivot}
                            closed
                            palette={palette}
                            onClick={() => onSelectPivot(pivot, node)}
                        />
                    ))}
                </ClosedSectionToggle>
            </div>
        </div>
    );
};

const OtherShellsHostRow: React.FC<{
    node: HostTreeNode;
    filters: HostsFilterState;
    hasActiveFilter: boolean;
    filterResetKey: string;
    palette: ThemePalette;
    onSelectHost: (node: HostTreeNode, toggleClosed: () => void) => void;
    onSelectShell: (shell: KnowledgeShellItem) => void;
    onSelectPivot: (pivot: ShellPivotItem, hostNode: HostTreeNode) => void;
}> = ({
    node,
    filters,
    hasActiveFilter,
    filterResetKey,
    palette,
    onSelectHost,
    onSelectShell,
    onSelectPivot,
}) => {
    const { effectiveExpanded, toggle } = useSearchableClosedToggle(
        hasActiveFilter,
        Boolean(node.hasMatchingClosed),
        filterResetKey
    );

    const hasIoFilter = Boolean(filters.input.trim() || filters.output.trim());
    const visibleMatchedShells = hasActiveFilter && hasIoFilter ? node.activeShells : [];
    const showClosedSection =
        node.closedPivots.length > 0 && hasActiveFilter && Boolean(node.hasMatchingClosed);
    const hasChildren =
        visibleMatchedShells.length > 0 || node.activePivots.length > 0 || showClosedSection;

    return (
        <div data-testid={`hosts-tree-host-${node.key}`}>
            <button
                type="button"
                onClick={() => onSelectHost(node, toggle)}
                className={`w-full flex items-center justify-between gap-2 px-1.5 py-0.5 rounded text-left transition-colors ${palette.rowHover}`}
            >
                <span className="flex items-center gap-1.5 min-w-0">
                    <Monitor className={`w-4 h-4 shrink-0 ${palette.hostIcon}`} />
                    <span className={`text-xs font-medium truncate ${palette.rowTitle}`}>
                        {node.name}
                    </span>
                </span>
                {node.ip && node.ip !== node.name && (
                    <span
                        data-testid={`hosts-tree-host-ip-${node.key}`}
                        className={`text-[11px] font-mono shrink-0 ${palette.rowSubtitle}`}
                    >
                        {node.ip}
                    </span>
                )}
            </button>

            {hasChildren && (
                <div className={`ml-3.5 mt-0.5 pl-2.5 border-l ${palette.branchLine} space-y-0.5`}>
                    {visibleMatchedShells.map((shell) => (
                        <ShellTreeItemRow
                            key={`shell-${shell.id}`}
                            shell={shell}
                            palette={palette}
                            onSelectShell={onSelectShell}
                        />
                    ))}

                    {node.activePivots.map((pivot) => (
                        <PivotTreeItemRow
                            key={`pivot-${pivot.id}`}
                            pivot={pivot}
                            palette={palette}
                            onClick={() => onSelectPivot(pivot, node)}
                        />
                    ))}

                    {showClosedSection && (
                        <ClosedSectionToggle
                            testId={`hosts-tree-toggle-closed-${node.key}`}
                            labelSubject={node.name}
                            count={node.closedPivots.length}
                            isExpanded={effectiveExpanded}
                            onToggle={toggle}
                            palette={palette}
                        >
                            {node.closedPivots.map((pivot) => (
                                <PivotTreeItemRow
                                    key={`closed-pivot-${pivot.id}`}
                                    pivot={pivot}
                                    closed
                                    palette={palette}
                                    onClick={() => onSelectPivot(pivot, node)}
                                />
                            ))}
                        </ClosedSectionToggle>
                    )}
                </div>
            )}
        </div>
    );
};

const HostOverviewBranch: React.FC<{
    node: HostTreeNode;
    currentShellId?: string;
    hasActiveFilter: boolean;
    filterResetKey: string;
    showOpenPtyButton?: boolean;
    portalOpen?: boolean;
    palette: ThemePalette;
    onOpenPty?: () => void;
    onSelectHost: (node: HostTreeNode) => void;
    onSelectShell: (shell: KnowledgeShellItem) => void;
    onSelectPivot: (pivot: ShellPivotItem, hostNode: HostTreeNode, fallbackShellId?: string) => void;
}> = ({
    node,
    currentShellId,
    hasActiveFilter,
    filterResetKey,
    showOpenPtyButton = false,
    portalOpen = false,
    palette,
    onOpenPty,
    onSelectHost,
    onSelectShell,
    onSelectPivot,
}) => {
    const { effectiveExpanded, toggle } = useSearchableClosedToggle(
        hasActiveFilter,
        Boolean(node.hasMatchingClosed),
        filterResetKey
    );

    const { pivotsByShellId, unmatchedActivePivots } = useMemo(
        () => groupPivotsByShellId(node.activeShells, node.activePivots),
        [node.activeShells, node.activePivots]
    );

    const visibleClosedShells = hasActiveFilter ? node.closedShells : [];
    const hostClosedPivots = useMemo(
        () =>
            dedupePivotsById([
                ...node.closedPivots,
                ...node.beacons.flatMap((beacon) => beacon.closedPivots),
            ]),
        [node.closedPivots, node.beacons]
    );
    const closedCount = visibleClosedShells.length + hostClosedPivots.length;
    const showClosedSection =
        closedCount > 0 &&
        (node.isCurrentHost || (hasActiveFilter && Boolean(node.hasMatchingClosed)));

    const hasPrivilegedAccess = useMemo(() => hasPrivilegedHostAccess(node), [node]);

    const hasChildren =
        node.activeShells.length > 0 ||
        node.activePivots.length > 0 ||
        showClosedSection;

    const monitorWithBadge = (
        <span className="relative inline-flex items-center justify-center shrink-0">
            <Monitor
                data-testid={node.isCurrentHost ? "hosts-tree-current-monitor-icon" : undefined}
                className={`w-4 h-4 ${palette.hostIcon}`}
            />
            {hasPrivilegedAccess && (
                <Zap
                    data-testid={`hosts-tree-privileged-icon-${node.key}`}
                    aria-label="Privileged access"
                    className={`w-2.5 h-2.5 absolute -top-1 -left-1 ${palette.addBadgeBg} ${palette.privilegedIcon} rounded-full stroke-[2.25]`}
                />
            )}
        </span>
    );

    return (
        <div data-testid={`hosts-tree-host-${node.key}`}>
            <div
                className={`w-full flex items-center justify-between gap-2 px-1.5 py-0.5 rounded transition-colors ${palette.rowHover}`}
            >
                <button
                    type="button"
                    onClick={() => onSelectHost(node)}
                    className="flex items-center gap-1.5 min-w-0 flex-1 text-left"
                >
                    {node.isCurrentHost ? (
                        <span className="inline-flex items-center gap-1.5 shrink-0">
                            <span
                                data-testid="hosts-tree-current-indicator"
                                aria-hidden="true"
                                className={`w-0.5 h-3.5 rounded-full ${palette.currentSelectorBar}`}
                            />
                            {monitorWithBadge}
                        </span>
                    ) : (
                        monitorWithBadge
                    )}
                    <span className={`text-xs font-medium truncate ${palette.rowTitle}`}>
                        {node.name}
                    </span>
                </button>
                {(showOpenPtyButton || (node.isCurrentHost && !node.isOnline)) && (
                    <div className="flex items-center gap-1.5 shrink-0">
                        <HostStatusBadge node={node} palette={palette} />
                        {showOpenPtyButton && (
                            <HostRowOpenPtyButton
                                portalOpen={portalOpen}
                                palette={palette}
                                onOpenPty={onOpenPty}
                            />
                        )}
                    </div>
                )}
            </div>

            {hasChildren && (
                <div
                    className={`${node.isCurrentHost ? "ml-4" : "ml-3.5"} mt-0.5 pl-2.5 border-l ${palette.branchLine} space-y-0.5`}
                >
                    {node.activeShells.map((shell) => {
                        const shellPivots = pivotsByShellId.get(String(shell.id)) || [];
                        return (
                            <div key={`shell-${shell.id}`} className="space-y-0.5">
                                <ShellTreeItemRow
                                    shell={shell}
                                    isCurrent={Boolean(
                                        currentShellId && String(shell.id) === String(currentShellId)
                                    )}
                                    showPrincipal={true}
                                    palette={palette}
                                    onSelectShell={onSelectShell}
                                />
                                {shellPivots.length > 0 && (
                                    <div
                                        data-testid={`hosts-tree-shell-pivots-${shell.id}`}
                                        className={`ml-3.5 mt-0.5 pl-2.5 border-l ${palette.branchLine} space-y-0.5`}
                                    >
                                        {shellPivots.map((pivot) => (
                                            <PivotTreeItemRow
                                                key={`pivot-${pivot.id}`}
                                                pivot={pivot}
                                                palette={palette}
                                                onClick={() =>
                                                    onSelectPivot(pivot, node, String(shell.id))
                                                }
                                            />
                                        ))}
                                    </div>
                                )}
                            </div>
                        );
                    })}

                    {unmatchedActivePivots.map((pivot) => (
                        <PivotTreeItemRow
                            key={`pivot-${pivot.id}`}
                            pivot={pivot}
                            palette={palette}
                            onClick={() => onSelectPivot(pivot, node)}
                        />
                    ))}

                    {showClosedSection && (
                        <ClosedSectionToggle
                            testId={`hosts-tree-toggle-closed-${node.key}`}
                            labelSubject={node.name}
                            count={closedCount}
                            isExpanded={effectiveExpanded}
                            onToggle={toggle}
                            palette={palette}
                        >
                            {visibleClosedShells.map((shell) => (
                                <ShellTreeItemRow
                                    key={`closed-shell-${shell.id}`}
                                    shell={shell}
                                    closed
                                    showPrincipal={true}
                                    palette={palette}
                                    onSelectShell={onSelectShell}
                                />
                            ))}
                            {hostClosedPivots.map((pivot) => (
                                <PivotTreeItemRow
                                    key={`closed-pivot-${pivot.id}`}
                                    pivot={pivot}
                                    closed
                                    palette={palette}
                                    onClick={() => onSelectPivot(pivot, node)}
                                />
                            ))}
                        </ClosedSectionToggle>
                    )}
                </div>
            )}
        </div>
    );
};

const HostsTreeHeader: React.FC<{
    viewMode: HostsTreeViewMode;
    activeHostTitle: string;
    palette: ThemePalette;
    onSelectOverview: () => void;
    onAddHost?: () => void;
    onAddCredential?: () => void;
    onClose?: () => void;
}> = ({
    viewMode,
    activeHostTitle,
    palette,
    onSelectOverview,
    onAddHost,
    onAddCredential,
    onClose,
}) => {
    const breadcrumbClass =
        viewMode === "overview" ? palette.breadcrumbActive : palette.breadcrumbLink;

    return (
        <div
            className={`px-3 py-2 border-b ${palette.headerBorder} flex items-center justify-between gap-2`}
        >
            <nav aria-label="Access breadcrumb" className="flex items-center gap-1 min-w-0 flex-1">
                <button
                    type="button"
                    data-testid="hosts-tree-breadcrumb-hosts"
                    onClick={onSelectOverview}
                    className={`inline-flex items-center gap-1.5 text-xs font-semibold uppercase tracking-wider shrink-0 transition-colors ${breadcrumbClass}`}
                >
                    <Network className={`w-4 h-4 ${palette.headerIcon}`} />
                    <span className={breadcrumbClass}>Access</span>
                </button>
                {viewMode === "host" && (
                    <>
                        <span
                            aria-hidden="true"
                            className={`text-xs shrink-0 ${palette.breadcrumbSeparator}`}
                        >
                            /
                        </span>
                        <span
                            data-testid="hosts-tree-breadcrumb-active-host"
                            title={activeHostTitle}
                            className={`text-xs font-semibold truncate ${palette.breadcrumbActive}`}
                        >
                            {activeHostTitle}
                        </span>
                    </>
                )}
            </nav>
            <div className="flex items-center gap-1.5 shrink-0">
                {onAddHost && (
                    <button
                        type="button"
                        onClick={onAddHost}
                        data-testid="hosts-tree-add-button"
                        aria-label="Add Host"
                        title="Add Host"
                        className={`inline-flex items-center justify-center p-1 rounded-md transition-colors ${palette.closeBtn}`}
                    >
                        <span className="relative inline-flex items-center justify-center">
                            <Monitor className="w-4 h-4" />
                            <Plus
                                data-testid="hosts-tree-add-plus-icon"
                                className={`w-2.5 h-2.5 absolute -top-1 -right-1 ${palette.addBadgeBg} rounded-full stroke-[2.5]`}
                            />
                        </span>
                    </button>
                )}
                {onAddCredential && (
                    <button
                        type="button"
                        onClick={onAddCredential}
                        data-testid="hosts-tree-add-credential-button"
                        aria-label="Add Credential"
                        title="Add Credential"
                        className={`inline-flex items-center justify-center p-1 rounded-md transition-colors ${palette.closeBtn}`}
                    >
                        <span className="relative inline-flex items-center justify-center">
                            <KeyRound className="w-4 h-4" />
                            <Plus
                                data-testid="hosts-tree-add-credential-plus-icon"
                                className={`w-2.5 h-2.5 absolute -top-1 -right-1 ${palette.addBadgeBg} rounded-full stroke-[2.5]`}
                            />
                        </span>
                    </button>
                )}
                {onClose && (
                    <button
                        type="button"
                        onClick={onClose}
                        aria-label="Close access drawer"
                        className={`p-1 rounded-md transition-colors ${palette.closeBtn}`}
                    >
                        <X className="w-4 h-4" />
                    </button>
                )}
            </div>
        </div>
    );
};

const SpecificHostViewContent: React.FC<{
    filteredSections: HostsTreeSections;
    currentShellId?: string;
    filters: HostsFilterState;
    hasActiveFilter: boolean;
    filterResetKey: string;
    showOpenPtyButton: boolean;
    portalOpen: boolean;
    closingPivotIds: string[];
    palette: ThemePalette;
    onOpenPty?: () => void;
    onSelectShell: (shell: KnowledgeShellItem) => void;
    onSelectPivot: (pivot: ShellPivotItem, hostNode: HostTreeNode) => void;
    onClosePivot?: (pivot: ShellPivotItem) => void;
    onSelectAvailableHost: (hostNode: HostTreeNode) => void;
    onSelectOtherShellsHost: (node: HostTreeNode, toggleClosed: () => void) => void;
}> = ({
    filteredSections,
    currentShellId,
    filters,
    hasActiveFilter,
    filterResetKey,
    showOpenPtyButton,
    portalOpen,
    closingPivotIds,
    palette,
    onOpenPty,
    onSelectShell,
    onSelectPivot,
    onClosePivot,
    onSelectAvailableHost,
    onSelectOtherShellsHost,
}) => (
    <>
        {filteredSections.currentHostNode && (
            <SpecificHostSection
                node={filteredSections.currentHostNode}
                availableHosts={filteredSections.availablePivotHosts}
                currentShellId={currentShellId}
                filters={filters}
                hasActiveFilter={hasActiveFilter}
                filterResetKey={filterResetKey}
                showOpenPtyButton={showOpenPtyButton}
                portalOpen={portalOpen}
                closingPivotIds={closingPivotIds}
                palette={palette}
                onOpenPty={onOpenPty}
                onSelectShell={onSelectShell}
                onSelectPivot={onSelectPivot}
                onClosePivot={onClosePivot}
                onSelectAvailableHost={onSelectAvailableHost}
            />
        )}

        {hasActiveFilter && filteredSections.activePivotHosts.length > 0 && (
            <div data-testid="hosts-tree-active-pivots-section" className="space-y-0.5">
                <HostDivider
                    testId="hosts-tree-other-shells-divider"
                    label="Other Shells"
                    palette={palette}
                />
                {filteredSections.activePivotHosts.map((node) => (
                    <OtherShellsHostRow
                        key={node.key}
                        node={node}
                        filters={filters}
                        hasActiveFilter={hasActiveFilter}
                        filterResetKey={filterResetKey}
                        palette={palette}
                        onSelectHost={onSelectOtherShellsHost}
                        onSelectShell={onSelectShell}
                        onSelectPivot={onSelectPivot}
                    />
                ))}
            </div>
        )}
    </>
);

const OverviewViewContent: React.FC<{
    currentHostNode: HostTreeNode | null;
    overviewOtherHosts: HostTreeNode[];
    currentShellId?: string;
    hasActiveFilter: boolean;
    filterResetKey: string;
    showOpenPtyButton: boolean;
    portalOpen: boolean;
    palette: ThemePalette;
    onOpenPty?: () => void;
    onSelectHost: (node: HostTreeNode) => void;
    onSelectShell: (shell: KnowledgeShellItem) => void;
    onSelectPivot: (pivot: ShellPivotItem, hostNode: HostTreeNode, fallbackShellId?: string) => void;
}> = ({
    currentHostNode,
    overviewOtherHosts,
    currentShellId,
    hasActiveFilter,
    filterResetKey,
    showOpenPtyButton,
    portalOpen,
    palette,
    onOpenPty,
    onSelectHost,
    onSelectShell,
    onSelectPivot,
}) => (
    <>
        {currentHostNode && (
            <div data-testid="hosts-tree-current-section">
                <HostOverviewBranch
                    node={currentHostNode}
                    currentShellId={currentShellId}
                    hasActiveFilter={hasActiveFilter}
                    filterResetKey={filterResetKey}
                    showOpenPtyButton={showOpenPtyButton}
                    portalOpen={portalOpen}
                    palette={palette}
                    onOpenPty={onOpenPty}
                    onSelectHost={onSelectHost}
                    onSelectShell={onSelectShell}
                    onSelectPivot={onSelectPivot}
                />
            </div>
        )}

        {overviewOtherHosts.length > 0 && (
            <div data-testid="hosts-tree-overview-section" className="space-y-0.5">
                {overviewOtherHosts.map((node) => (
                    <HostOverviewBranch
                        key={node.key}
                        node={node}
                        currentShellId={currentShellId}
                        hasActiveFilter={hasActiveFilter}
                        filterResetKey={filterResetKey}
                        palette={palette}
                        onSelectHost={onSelectHost}
                        onSelectShell={onSelectShell}
                        onSelectPivot={onSelectPivot}
                    />
                ))}
            </div>
        )}
    </>
);

export interface CreateShellForHostRequest {
    hostId?: string;
    hostName?: string;
    beaconId?: string;
}

export interface HostsTreeViewProps {
    theme?: HostsTreeTheme;
    hosts: KnowledgeHostItem[];
    shells: KnowledgeShellItem[];
    pivots: ShellPivotItem[];
    currentHostBeacons?: KnowledgeBeaconItem[];
    shellTasks?: ShellTaskItem[];
    currentHostId?: string;
    currentHostName?: string;
    currentHostIp?: string;
    currentShellId?: string;
    currentUserId?: string;
    currentUserName?: string;
    portalOpen?: boolean;
    onOpenPty?: () => void;
    onSelectShell?: (shellId: string) => void;
    onCreateShellForHost?: (request: CreateShellForHostRequest) => void;
    onSelectPivot?: (pivot: ShellPivotItem, targetShellId?: string) => void;
    onClosePivot?: (pivot: ShellPivotItem) => void;
    closingPivotIds?: string[];
    onOpenSshWithTarget?: (target: string) => void;
    onAddHost?: () => void;
    onAddCredential?: () => void;
    onClose?: () => void;
    showHeader?: boolean;
    className?: string;
}

export const HostsTreeView: React.FC<HostsTreeViewProps> = ({
    theme = "dark",
    hosts,
    shells,
    pivots,
    currentHostBeacons = [],
    shellTasks = [],
    currentHostId,
    currentHostName,
    currentHostIp,
    currentShellId,
    currentUserId,
    currentUserName,
    portalOpen = false,
    onOpenPty,
    onSelectShell,
    onCreateShellForHost,
    onSelectPivot,
    onClosePivot,
    closingPivotIds = [],
    onOpenSshWithTarget,
    onAddHost,
    onAddCredential,
    onClose,
    showHeader = true,
    className = "",
}) => {
    const palette = THEME_PALETTES[theme];
    const [viewMode, setViewMode] = useState<HostsTreeViewMode>("host");
    const [filters, setFilters] = useState<HostsFilterState>(EMPTY_HOSTS_FILTERS);
    const hasActiveFilter = hasActiveHostFilters(filters);
    const filterResetKey = `${filters.name}|${filters.ip}|${filters.input}|${filters.output}`;

    useEffect(() => {
        setViewMode("host");
    }, [currentShellId, currentHostId]);

    const sections = useMemo(
        () =>
            buildHostsTreeSections({
                hosts,
                shells,
                pivots,
                currentHostBeacons,
                currentHostId,
                currentHostName,
                currentHostIp,
                currentShellId,
            }),
        [
            hosts,
            shells,
            pivots,
            currentHostBeacons,
            currentHostId,
            currentHostName,
            currentHostIp,
            currentShellId,
        ]
    );

    const currentHostHasActivePty = useMemo(
        () =>
            Boolean(
                sections.currentHostNode?.activePivots.some(
                    (pivot) =>
                        String(pivot.kind || "").toLowerCase() === "pty" && !pivot.closedAt
                )
            ),
        [sections.currentHostNode]
    );

    const filteredSections = useMemo(
        () => filterHostsTreeSections(sections, filters, shellTasks),
        [sections, filters, shellTasks]
    );

    const overviewOtherHosts = useMemo(() => {
        const seen = new Set<string>();
        const combined: HostTreeNode[] = [];
        for (const list of [
            filteredSections.activePivotHosts,
            filteredSections.hiddenOtherHosts,
            filteredSections.availablePivotHosts,
        ]) {
            for (const node of list) {
                if (seen.has(node.key)) {
                    continue;
                }
                seen.add(node.key);
                combined.push(node);
            }
        }
        return combined.sort((a, b) =>
            compareHostTreeNodes(a, b, currentUserId, currentUserName)
        );
    }, [
        filteredSections.activePivotHosts,
        filteredSections.hiddenOtherHosts,
        filteredSections.availablePivotHosts,
        currentUserId,
        currentUserName,
    ]);

    const unfilteredHostDataByKey = useMemo(() => {
        const map = new Map<
            string,
            {
                beacons: HostBeaconNode[];
                activeShells: KnowledgeShellItem[];
                closedShells: KnowledgeShellItem[];
            }
        >();
        const register = (node: HostTreeNode | null) => {
            if (!node) return;
            map.set(node.key, {
                beacons: node.beacons,
                activeShells: node.activeShells,
                closedShells: node.closedShells,
            });
        };
        register(sections.currentHostNode);
        for (const node of sections.activePivotHosts) register(node);
        for (const node of sections.hiddenOtherHosts) register(node);
        for (const node of sections.availablePivotHosts) register(node);
        return map;
    }, [sections]);

    const handleSelectShell = (shell: KnowledgeShellItem) => {
        onSelectShell?.(String(shell.id));
        if (currentShellId && String(shell.id) === String(currentShellId)) {
            setViewMode("host");
        }
    };

    const handleSelectPivot = (
        pivot: ShellPivotItem,
        hostNode: HostTreeNode,
        explicitFallbackShellId?: string
    ) => {
        const hostShellId = hostNode.activeShells[0]?.id || hostNode.closedShells[0]?.id;
        const targetShellId =
            pivot.shellId ||
            explicitFallbackShellId ||
            (!hostNode.isCurrentHost && hostShellId ? String(hostShellId) : undefined);
        onSelectPivot?.(pivot, targetShellId);
        if (!targetShellId || (currentShellId && String(targetShellId) === String(currentShellId))) {
            setViewMode("host");
        }
    };

    const handleSelectAvailableHost = (hostNode: HostTreeNode) => {
        const targetAddr = hostNode.ip || hostNode.name;
        if (!targetAddr || !onOpenSshWithTarget) {
            return;
        }
        const ports = hostNode.sourceHost?.openPorts || [];
        const defaultPort = ports.includes(22) || ports.length === 0 ? 22 : ports[0];
        onOpenSshWithTarget(`${targetAddr}:${defaultPort}`);
    };

    const handleNavigateToBeaconOrCreate = (
        hostNode: HostTreeNode,
        beaconId: string,
        beaconShells: KnowledgeShellItem[]
    ): boolean => {
        const targetShell = pickPreferredShell(beaconShells, currentUserId, currentUserName);
        if (targetShell) {
            onSelectShell?.(String(targetShell.id));
            if (currentShellId && String(targetShell.id) === String(currentShellId)) {
                setViewMode("host");
            }
            return true;
        }
        if (onCreateShellForHost) {
            onCreateShellForHost({
                hostId: hostNode.hostId,
                hostName: hostNode.name,
                beaconId,
            });
            return true;
        }
        return false;
    };

    const handleSelectOverviewHost = (node: HostTreeNode) => {
        if (node.isCurrentHost) {
            setViewMode("host");
            return;
        }
        const unfiltered = unfilteredHostDataByKey.get(node.key);
        const candidateBeacons = unfiltered?.beacons || node.beacons;
        const candidateShells = unfiltered?.activeShells || node.activeShells;
        const bestBeaconId =
            pickBestBeaconIdForBeacons(candidateBeacons) || node.sourceHost?.primaryBeaconId;

        if (bestBeaconId) {
            const bestBeaconShells = candidateShells.filter(
                (s) => String(s.beaconId) === String(bestBeaconId)
            );
            if (handleNavigateToBeaconOrCreate(node, bestBeaconId, bestBeaconShells)) {
                return;
            }
        }

        const fallbackShell = pickPreferredShell(candidateShells, currentUserId, currentUserName);
        if (fallbackShell) {
            onSelectShell?.(String(fallbackShell.id));
            return;
        }
        if (node.activePivots.length > 0) {
            handleSelectPivot(node.activePivots[0], node);
            return;
        }
        handleSelectAvailableHost(node);
    };

    const handleSelectOtherShellsHost = (node: HostTreeNode, toggleClosed: () => void) => {
        const unfiltered = unfilteredHostDataByKey.get(node.key);
        const candidateShells = unfiltered?.activeShells || node.activeShells;
        const preferredShell = pickPreferredShell(candidateShells, currentUserId, currentUserName);
        if (preferredShell) {
            onSelectShell?.(String(preferredShell.id));
            return;
        }

        const candidateBeacons = unfiltered?.beacons || node.beacons;
        const bestBeaconId =
            pickBestBeaconIdForBeacons(candidateBeacons) ||
            node.sourceHost?.primaryBeaconId ||
            candidateShells.find((s) => s.beaconId)?.beaconId;
        if (bestBeaconId && onCreateShellForHost) {
            onCreateShellForHost({
                hostId: node.hostId,
                hostName: node.name,
                beaconId: bestBeaconId,
            });
            return;
        }
        if (node.activePivots.length > 0) {
            handleSelectPivot(node.activePivots[0], node);
            return;
        }
        if (hasActiveFilter && node.hasMatchingClosed && node.closedPivots.length > 0) {
            toggleClosed();
            return;
        }
        handleSelectAvailableHost(node);
    };

    const activeHostTitle =
        sections.currentHostNode?.name || currentHostName || currentHostIp || "Host";

    const hasVisibleSpecificResults =
        Boolean(filteredSections.currentHostNode) ||
        (hasActiveFilter && filteredSections.activePivotHosts.length > 0);
    const hasVisibleOverviewResults =
        Boolean(filteredSections.currentHostNode) || overviewOtherHosts.length > 0;
    const hasVisibleResults =
        viewMode === "overview" ? hasVisibleOverviewResults : hasVisibleSpecificResults;

    return (
        <div
            data-testid="hosts-tree-view"
            data-theme={theme}
            className={`flex flex-col h-full select-none ${palette.container} ${className}`}
        >
            {showHeader && (
                <HostsTreeHeader
                    viewMode={viewMode}
                    activeHostTitle={activeHostTitle}
                    palette={palette}
                    onSelectOverview={() => setViewMode("overview")}
                    onAddHost={onAddHost}
                    onAddCredential={onAddCredential}
                    onClose={onClose}
                />
            )}

            <HostsFilterPopover
                theme={theme}
                filters={filters}
                onChange={setFilters}
                onClear={() => setFilters(EMPTY_HOSTS_FILTERS)}
            />

            <div className="flex-1 overflow-y-auto px-2.5 py-1.5 space-y-1 select-text">
                {viewMode === "host" ? (
                    <SpecificHostViewContent
                        filteredSections={filteredSections}
                        currentShellId={currentShellId}
                        filters={filters}
                        hasActiveFilter={hasActiveFilter}
                        filterResetKey={filterResetKey}
                        showOpenPtyButton={!currentHostHasActivePty}
                        portalOpen={portalOpen}
                        closingPivotIds={closingPivotIds}
                        palette={palette}
                        onOpenPty={onOpenPty}
                        onSelectShell={handleSelectShell}
                        onSelectPivot={handleSelectPivot}
                        onClosePivot={onClosePivot}
                        onSelectAvailableHost={handleSelectAvailableHost}
                        onSelectOtherShellsHost={handleSelectOtherShellsHost}
                    />
                ) : (
                    <OverviewViewContent
                        currentHostNode={filteredSections.currentHostNode}
                        overviewOtherHosts={overviewOtherHosts}
                        currentShellId={currentShellId}
                        hasActiveFilter={hasActiveFilter}
                        filterResetKey={filterResetKey}
                        showOpenPtyButton={!currentHostHasActivePty}
                        portalOpen={portalOpen}
                        palette={palette}
                        onOpenPty={onOpenPty}
                        onSelectHost={handleSelectOverviewHost}
                        onSelectShell={handleSelectShell}
                        onSelectPivot={handleSelectPivot}
                    />
                )}

                {hasActiveFilter && !hasVisibleResults && (
                    <div
                        data-testid="hosts-tree-empty-search"
                        className={`py-8 px-3 text-center text-xs space-y-2 ${palette.emptyText}`}
                    >
                        <p>No matching hosts, shells, pivots, or PTYs.</p>
                        <button
                            type="button"
                            onClick={() => setFilters(EMPTY_HOSTS_FILTERS)}
                            className={`text-xs font-medium underline ${palette.clearFilterBtn}`}
                        >
                            Clear filter
                        </button>
                    </div>
                )}
            </div>

            <div className={`px-3 py-1.5 border-t text-[11px] ${palette.footer}`}>
                {viewMode === "overview"
                    ? "Select a host, shell, pivot, or PTY"
                    : "Select a pivot, PTY, or target host"}
            </div>
        </div>
    );
};

export interface HostsTreeDrawerProps
    extends Omit<HostsTreeViewProps, "onAddHost" | "onAddCredential" | "showHeader"> {
    isOpen: boolean;
    onClose: () => void;
    availableCredentials?: KnowledgeCredentialItem[];
    onHostSaved?: (savedHost: KnowledgeHostItem, mode: "create" | "edit") => void;
    onCredentialCreated?: (
        credential: KnowledgeCredentialItem,
        targetHostId?: string
    ) => void;
    onAddHost?: () => void;
    onAddCredential?: () => void;
}

export const HostsTreeDrawer: React.FC<HostsTreeDrawerProps> = ({
    isOpen,
    onClose,
    theme = "dark",
    availableCredentials = [],
    onHostSaved,
    onCredentialCreated,
    onAddHost,
    onAddCredential,
    ...treeProps
}) => {
    const [hostModalOpen, setHostModalOpen] = useState(false);
    const [credentialModalOpen, setCredentialModalOpen] = useState(false);

    if (!isOpen) {
        return null;
    }

    const borderSide = theme === "light" ? "border-gray-200" : "border-gray-800";

    return (
        <aside
            data-testid="shell-hosts-drawer"
            aria-label="Access drawer"
            className={`w-full md:w-96 max-w-full border-l ${borderSide} flex flex-col h-full z-20 shrink-0`}
        >
            <HostsTreeView
                {...treeProps}
                theme={theme}
                onClose={onClose}
                onAddHost={onAddHost || (() => setHostModalOpen(true))}
                onAddCredential={onAddCredential || (() => setCredentialModalOpen(true))}
                showHeader={true}
            />
            {hostModalOpen && (
                <HostModal
                    isOpen={hostModalOpen}
                    onClose={() => setHostModalOpen(false)}
                    initialHost={null}
                    availableCredentials={availableCredentials}
                    onHostSaved={(savedHost, mode) => {
                        onHostSaved?.(savedHost, mode);
                    }}
                />
            )}
            {credentialModalOpen && (
                <CredentialModal
                    isOpen={credentialModalOpen}
                    onClose={() => setCredentialModalOpen(false)}
                    currentHostId={treeProps.currentHostId}
                    currentHostName={treeProps.currentHostName}
                    currentHostIp={treeProps.currentHostIp}
                    hosts={treeProps.hosts}
                    onCredentialCreated={(createdCred, targetHostId) => {
                        onCredentialCreated?.(createdCred, targetHostId);
                    }}
                />
            )}
        </aside>
    );
};

export default HostsTreeDrawer;
