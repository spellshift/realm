import React, { useMemo, useState } from "react";
import {
    Network,
    PowerOff,
    Search,
    Shell,
    SquareTerminal,
    Terminal,
    X,
} from "lucide-react";
import { ShellPivotItem } from "../hooks/useShellData";

interface ShellPivotsDrawerProps {
    isOpen: boolean;
    onClose: () => void;
    portalId: number | null;
    pivots: ShellPivotItem[];
    onOpenPivotTab: (type: "pty" | "ssh", target: string, pivotId?: string) => void;
    onOpenPty: () => void;
    onOpenSshModal: () => void;
    onCreatePortal?: () => void;
    onNewPortal?: () => void;
    onClosePortal: () => void;
}

const ShellPivotsDrawer: React.FC<ShellPivotsDrawerProps> = ({
    isOpen,
    onClose,
    portalId,
    pivots,
    onOpenPivotTab,
    onOpenPty,
    onOpenSshModal,
    onCreatePortal,
    onNewPortal,
    onClosePortal,
}) => {
    const handleCreatePortal = onCreatePortal || onNewPortal || (() => {});
    const [searchQuery, setSearchQuery] = useState("");
    const [statusFilter, setStatusFilter] = useState<"all" | "active" | "closed">("all");

    const activeCount = useMemo(
        () => pivots.filter((p) => !p.closedAt).length,
        [pivots]
    );

    const filteredPivots = useMemo(() => {
        return pivots.filter((p) => {
            if (statusFilter === "active" && p.closedAt) return false;
            if (statusFilter === "closed" && !p.closedAt) return false;
            if (
                searchQuery.trim() &&
                !`${p.kind} ${p.destination} ${p.id}`
                    .toLowerCase()
                    .includes(searchQuery.toLowerCase())
            ) {
                return false;
            }
            return true;
        });
    }, [pivots, statusFilter, searchQuery]);

    if (!isOpen) return null;

    return (
        <div
            data-testid="shell-pivots-drawer"
            className="w-full md:w-96 max-w-full shrink-0 border-l border-gray-800 bg-gray-900 text-gray-200 flex flex-col h-full overflow-hidden shadow-2xl z-20"
        >
            {/* Header */}
            <div className="flex items-center justify-between px-3 py-2 border-b border-gray-800 bg-gray-900">
                <div className="flex items-center gap-1.5">
                    <Network className="w-4 h-4 text-gray-400" />
                    <h2 className="text-xs font-semibold uppercase tracking-wider text-gray-200">
                        Pivots
                    </h2>
                    {pivots.length > 0 && (
                        <span className="text-xs font-mono px-1.5 py-0.5 rounded bg-gray-800 text-gray-400">
                            {pivots.length}
                        </span>
                    )}
                </div>
                <button
                    type="button"
                    onClick={onClose}
                    aria-label="Close pivots drawer"
                    className="p-1 rounded-md hover:bg-gray-800 text-gray-400 hover:text-white transition-colors"
                >
                    <X className="w-4 h-4" />
                </button>
            </div>

            {/* Active Portal Controls */}
            <div className="p-3 border-b border-gray-800 bg-gray-900 space-y-2.5">
                {portalId ? (
                    <>
                        <div className="flex items-center justify-between text-xs">
                            <span className="inline-flex items-center gap-1.5 text-emerald-400 font-medium">
                                <span className="w-2 h-2 rounded-full bg-emerald-400" />
                                Portal #{portalId} Active
                            </span>
                            <button
                                type="button"
                                onClick={onClosePortal}
                                className="inline-flex items-center gap-1 px-2.5 py-1 rounded-md text-xs font-semibold text-red-300 hover:bg-red-900/40 border border-red-500/30 shadow-sm transition-colors"
                            >
                                <PowerOff className="w-3.5 h-3.5" />
                                Close Portal
                            </button>
                        </div>
                        <div className="grid grid-cols-2 gap-2">
                            <button
                                type="button"
                                onClick={onOpenPty}
                                className="inline-flex items-center justify-center gap-1.5 px-2.5 py-1 rounded-md bg-gray-800 hover:bg-gray-700 border border-gray-700 text-gray-200 hover:text-white text-xs font-semibold shadow-sm transition-colors"
                            >
                                <SquareTerminal className="w-4 h-4 text-gray-400" />
                                PTY
                            </button>
                            <button
                                type="button"
                                onClick={onOpenSshModal}
                                className="inline-flex items-center justify-center gap-1.5 px-2.5 py-1 rounded-md bg-gray-800 hover:bg-gray-700 border border-gray-700 text-gray-200 hover:text-white text-xs font-semibold shadow-sm transition-colors"
                            >
                                <Terminal className="w-4 h-4 text-gray-400" />
                                SSH
                            </button>
                        </div>
                    </>
                ) : (
                    <div className="flex items-center justify-between gap-2">
                        <div className="text-xs text-gray-400">
                            An active portal is required for interactive PTY & SSH pivots.
                        </div>
                        <button
                            type="button"
                            onClick={handleCreatePortal}
                            className="inline-flex items-center gap-1 px-2.5 py-1 rounded-md bg-gray-800 hover:bg-gray-700 border border-gray-700 text-white text-xs font-semibold shrink-0 shadow-sm transition-colors"
                        >
                            <Shell className="w-4 h-4 text-gray-400" />
                            Open Portal
                        </button>
                    </div>
                )}
            </div>

            {/* Search & Status Filters */}
            <div className="p-3 border-b border-gray-800 bg-gray-900 space-y-2">
                <div className="relative">
                    <Search className="w-4 h-4 text-gray-400 absolute left-2.5 top-1/2 -translate-y-1/2 pointer-events-none" />
                    <input
                        type="text"
                        value={searchQuery}
                        onChange={(e) => setSearchQuery(e.target.value)}
                        placeholder="Search PTY or SSH pivots..."
                        className="w-full pl-8 pr-3 py-1.5 rounded-md bg-gray-800 border border-gray-700 text-xs text-gray-200 placeholder-gray-400 focus:outline-none focus:border-purple-500"
                    />
                </div>
                <div className="flex items-center gap-1.5 text-xs">
                    {(["all", "active", "closed"] as const).map((f) => (
                        <button
                            key={f}
                            type="button"
                            onClick={() => setStatusFilter(f)}
                            className={`px-2.5 py-1 rounded-md capitalize font-semibold border transition-colors ${
                                statusFilter === f
                                    ? "bg-purple-600 text-white border-purple-500 shadow-sm"
                                    : "bg-gray-800 text-gray-400 hover:text-gray-200 border-gray-700"
                            }`}
                        >
                            {f === "active" ? `Active (${activeCount})` : f}
                        </button>
                    ))}
                </div>
            </div>

            {/* Pivot List */}
            <div className="flex-1 overflow-y-auto p-3 space-y-2">
                {filteredPivots.length === 0 ? (
                    <div className="text-center py-8 text-xs text-gray-500">
                        No pivot sessions match your filter.
                    </div>
                ) : (
                    filteredPivots.map((pivot) => {
                        const isPty = pivot.kind?.toUpperCase() === "PTY";
                        const label = isPty ? "PTY" : pivot.destination;
                        const isClosed = Boolean(pivot.closedAt);

                        return (
                            <div
                                key={pivot.id}
                                className="rounded-lg border border-gray-800 bg-gray-800/40 p-2.5 space-y-2"
                            >
                                <div className="flex items-center justify-between gap-2">
                                    <div className="flex items-center gap-2 min-w-0">
                                        {isPty ? (
                                            <SquareTerminal className="w-4 h-4 text-gray-400 shrink-0" />
                                        ) : (
                                            <Terminal className="w-4 h-4 text-gray-400 shrink-0" />
                                        )}
                                        <span className="text-xs font-mono font-semibold text-gray-200 truncate">
                                            {isPty ? `PTY #${pivot.id}` : `SSH • ${pivot.destination}`}
                                        </span>
                                    </div>
                                    <span
                                        className={`text-[10px] px-1.5 py-0.5 rounded font-medium ${
                                            isClosed
                                                ? "bg-gray-500/15 text-gray-400 border border-gray-500/30"
                                                : "bg-emerald-500/15 text-emerald-300 border border-emerald-500/30"
                                        }`}
                                    >
                                        {isClosed ? "Closed" : "Active"}
                                    </span>
                                </div>

                                <div className="flex items-center justify-between text-xs text-gray-400 pt-1 border-t border-gray-800">
                                    <span>Pivot ID #{pivot.id}</span>
                                    <button
                                        type="button"
                                        onClick={() =>
                                            onOpenPivotTab(
                                                isPty ? "pty" : "ssh",
                                                label,
                                                pivot.id
                                            )
                                        }
                                        className="inline-flex items-center gap-1 px-2.5 py-1 rounded-md bg-gray-800 hover:bg-gray-700 text-gray-200 hover:text-white border border-gray-700 text-xs font-semibold shadow-sm transition-colors"
                                    >
                                        {isClosed ? "View Saved Transcript" : "Switch to Active Tab"}
                                    </button>
                                </div>
                            </div>
                        );
                    })
                )}
            </div>
        </div>
    );
};

export default ShellPivotsDrawer;
