import React from "react";
import { ShellPivotItem } from "../../hooks/useShellData";
import { formatPivotTabLabel } from "../../utils/networkGraphUtils";

export interface GraphPivotsSectionProps {
    portalId: number | null;
    pivots: ShellPivotItem[];
    onNewPortal?: () => void;
    onClosePortal?: () => void;
    onOpenPty?: () => void;
    onOpenSshModal?: () => void;
    onOpenSshWithTarget?: (target: string) => void;
    onOpenPivotTab?: (type: "ssh" | "pty", target: string, pivotId: string) => void;
}

const ActivePivotActions: React.FC<{
    pivot: ShellPivotItem;
    onOpenPivotTab?: (type: "ssh" | "pty", target: string, pivotId: string) => void;
}> = ({ pivot, onOpenPivotTab }) => {
    const kindLower = String(pivot.kind || "").toLowerCase() === "pty" ? "pty" : "ssh";
    return (
        <button
            type="button"
            onClick={() => onOpenPivotTab?.(kindLower, pivot.destination, String(pivot.id))}
            className="text-[11px] text-gray-200 hover:text-white px-2 py-0.5 rounded bg-gray-800 hover:bg-gray-700 transition-colors"
        >
            Switch
        </button>
    );
};

const ClosedPivotActions: React.FC<{
    pivot: ShellPivotItem;
    onOpenPty?: () => void;
    onOpenSshWithTarget?: (target: string) => void;
    onOpenPivotTab?: (type: "ssh" | "pty", target: string, pivotId: string) => void;
}> = ({ pivot, onOpenPty, onOpenSshWithTarget, onOpenPivotTab }) => {
    const isPty = String(pivot.kind || "").toLowerCase() === "pty";
    const kindLower = isPty ? "pty" : "ssh";

    const handleReopen = () => {
        if (isPty) {
            onOpenPty?.();
            return;
        }
        onOpenSshWithTarget?.(pivot.destination);
    };

    return (
        <div className="flex items-center gap-1.5">
            <button
                type="button"
                onClick={handleReopen}
                className="text-[11px] text-gray-200 hover:text-white px-2 py-0.5 rounded bg-gray-800 hover:bg-gray-700 transition-colors"
            >
                Reopen
            </button>
            {onOpenPivotTab && (
                <button
                    type="button"
                    onClick={() => onOpenPivotTab(kindLower, pivot.destination, String(pivot.id))}
                    className="text-[11px] text-gray-400 hover:text-gray-200 px-1.5 py-0.5 transition-colors"
                >
                    Transcript
                </button>
            )}
        </div>
    );
};

const PivotRow: React.FC<{
    pivot: ShellPivotItem;
    onOpenPty?: () => void;
    onOpenSshWithTarget?: (target: string) => void;
    onOpenPivotTab?: (type: "ssh" | "pty", target: string, pivotId: string) => void;
}> = ({ pivot, onOpenPty, onOpenSshWithTarget, onOpenPivotTab }) => {
    const isOpen = !pivot.closedAt;
    const kindLabel = String(pivot.kind || "SSH").toUpperCase();
    const dotClass = isOpen ? "bg-emerald-400" : "bg-gray-600";
    const pivotLabel = formatPivotTabLabel({
        type: pivot.kind || "ssh",
        target: pivot.activeTarget || pivot.destination,
    });

    return (
        <div className="py-2 flex items-center justify-between gap-2">
            <div className="min-w-0 flex-1">
                <div className="flex items-center gap-1.5 text-xs text-gray-200 font-mono truncate">
                    <span className={`w-1.5 h-1.5 rounded-full shrink-0 ${dotClass}`} />
                    <span>{pivotLabel}</span>
                </div>
                <div className="text-[11px] text-gray-500 pl-3 truncate">
                    {kindLabel}
                    {pivot.hostName ? ` • ${pivot.hostName}` : ""}
                </div>
            </div>
            {isOpen ? (
                <ActivePivotActions pivot={pivot} onOpenPivotTab={onOpenPivotTab} />
            ) : (
                <ClosedPivotActions
                    pivot={pivot}
                    onOpenPty={onOpenPty}
                    onOpenSshWithTarget={onOpenSshWithTarget}
                    onOpenPivotTab={onOpenPivotTab}
                />
            )}
        </div>
    );
};

export const GraphPivotsSection: React.FC<GraphPivotsSectionProps> = ({
    portalId,
    pivots,
    onOpenPty,
    onOpenSshModal,
    onOpenSshWithTarget,
    onOpenPivotTab,
}) => {
    return (
        <div className="px-4 py-3 border-b border-gray-800" data-testid="graph-pivots-section">
            <div className="flex items-center justify-between mb-2">
                <span className="text-[11px] uppercase tracking-wider text-gray-400 font-medium">
                    Pivots ({pivots.length})
                </span>
                <div className="flex items-center gap-1.5">
                    {onOpenPty && (
                        <button
                            type="button"
                            onClick={onOpenPty}
                            disabled={!portalId}
                            className="px-2 py-0.5 text-[11px] rounded bg-gray-800 hover:bg-gray-700 disabled:opacity-40 disabled:pointer-events-none text-gray-200 transition-colors"
                        >
                            PTY
                        </button>
                    )}
                    {onOpenSshModal && (
                        <button
                            type="button"
                            onClick={onOpenSshModal}
                            disabled={!portalId}
                            className="px-2 py-0.5 text-[11px] rounded bg-gray-800 hover:bg-gray-700 disabled:opacity-40 disabled:pointer-events-none text-gray-200 transition-colors"
                        >
                            SSH
                        </button>
                    )}
                </div>
            </div>

            {pivots.length === 0 ? (
                <div className="py-2 text-xs text-gray-500">No pivot sessions recorded.</div>
            ) : (
                <div className="divide-y divide-gray-800/60">
                    {pivots.map((pivot) => (
                        <PivotRow
                            key={pivot.id}
                            pivot={pivot}
                            onOpenPty={onOpenPty}
                            onOpenSshWithTarget={onOpenSshWithTarget}
                            onOpenPivotTab={onOpenPivotTab}
                        />
                    ))}
                </div>
            )}
        </div>
    );
};

export default GraphPivotsSection;

