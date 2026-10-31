import React, { useEffect, useRef, useState } from "react";
import { Pencil, RotateCcw } from "lucide-react";
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
    onRenamePivot?: (pivotId: string, newTitle: string) => void;
    onResetPivotName?: (pivotId: string) => void;
}

const ActivePivotActions: React.FC<{
    pivot: ShellPivotItem;
    onOpenPivotTab?: (type: "ssh" | "pty", target: string, pivotId: string) => void;
}> = ({ pivot, onOpenPivotTab }) => {
    const kindLower = String(pivot.kind || "").toLowerCase() === "pty" ? "pty" : "ssh";
    return (
        <button
            type="button"
            onClick={() =>
                onOpenPivotTab?.(
                    kindLower,
                    pivot.activeTarget || pivot.destination,
                    String(pivot.id)
                )
            }
            className="text-xs font-medium text-gray-200 hover:text-white px-2.5 py-1 rounded-md bg-gray-800 hover:bg-gray-700 transition-colors"
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
                className="text-xs font-medium text-gray-200 hover:text-white px-2.5 py-1 rounded-md bg-gray-800 hover:bg-gray-700 transition-colors"
            >
                Reopen
            </button>
            {onOpenPivotTab && (
                <button
                    type="button"
                    onClick={() =>
                        onOpenPivotTab(
                            kindLower,
                            pivot.activeTarget || pivot.destination,
                            String(pivot.id)
                        )
                    }
                    className="text-xs text-gray-400 hover:text-gray-200 px-1.5 py-1 transition-colors"
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
    onRenamePivot?: (pivotId: string, newTitle: string) => void;
    onResetPivotName?: (pivotId: string) => void;
}> = ({
    pivot,
    onOpenPty,
    onOpenSshWithTarget,
    onOpenPivotTab,
    onRenamePivot,
    onResetPivotName,
}) => {
    const isOpen = !pivot.closedAt;
    const kindLabel = String(pivot.kind || "SSH").toUpperCase();
    const dotClass = isOpen ? "bg-emerald-400" : "bg-gray-600";
    const pivotLabel = formatPivotTabLabel({
        type: pivot.kind || "ssh",
        target: pivot.activeTarget || pivot.destination,
        customTitle: pivot.customTitle,
    });

    const [isEditing, setIsEditing] = useState(false);
    const [editingTitle, setEditingTitle] = useState(pivot.customTitle || pivotLabel);
    const inputRef = useRef<HTMLInputElement>(null);

    useEffect(() => {
        if (isEditing && inputRef.current) {
            inputRef.current.focus();
            inputRef.current.select();
        }
    }, [isEditing]);

    const startEditing = () => {
        setEditingTitle(pivot.customTitle || pivotLabel);
        setIsEditing(true);
    };

    const commitRename = () => {
        const trimmed = editingTitle.trim();
        if (trimmed) {
            onRenamePivot?.(String(pivot.id), trimmed);
        } else {
            onResetPivotName?.(String(pivot.id));
        }
        setIsEditing(false);
    };

    const cancelRename = () => {
        setIsEditing(false);
    };

    return (
        <div className="py-2 flex items-center justify-between gap-2">
            <div className="min-w-0 flex-1">
                <div className="flex items-center gap-1.5 text-xs text-gray-200 font-mono truncate">
                    <span className={`w-1.5 h-1.5 rounded-full shrink-0 ${dotClass}`} />
                    {isEditing ? (
                        <input
                            ref={inputRef}
                            type="text"
                            data-testid={`rename-pivot-input-${pivot.id}`}
                            aria-label={`Rename pivot ${pivot.id}`}
                            value={editingTitle}
                            onChange={(e) => setEditingTitle(e.target.value)}
                            onKeyDown={(e) => {
                                if (e.key === "Enter") {
                                    e.preventDefault();
                                    e.stopPropagation();
                                    commitRename();
                                } else if (e.key === "Escape") {
                                    e.preventDefault();
                                    e.stopPropagation();
                                    cancelRename();
                                }
                            }}
                            onBlur={commitRename}
                            onClick={(e) => e.stopPropagation()}
                            className="bg-[#12151c] text-white text-xs px-1.5 py-0.5 rounded border border-purple-500 focus:outline-none focus:ring-1 focus:ring-purple-500 w-full"
                        />
                    ) : (
                        <div className="flex items-center gap-1.5 group/label min-w-0">
                            <span
                                onDoubleClick={(e) => {
                                    e.stopPropagation();
                                    startEditing();
                                }}
                                title="Double-click to rename"
                                className="cursor-pointer hover:text-white truncate"
                            >
                                {pivotLabel}
                            </span>
                            {onRenamePivot && (
                                <button
                                    type="button"
                                    onClick={(e) => {
                                        e.stopPropagation();
                                        startEditing();
                                    }}
                                    aria-label={`Rename pivot ${pivotLabel}`}
                                    title="Rename pivot"
                                    className="opacity-0 group-hover/label:opacity-100 p-0.5 text-gray-500 hover:text-gray-200 transition-opacity shrink-0"
                                >
                                    <Pencil className="w-3 h-3" />
                                </button>
                            )}
                            {pivot.customTitle && onResetPivotName && (
                                <button
                                    type="button"
                                    onClick={(e) => {
                                        e.stopPropagation();
                                        onResetPivotName(String(pivot.id));
                                    }}
                                    aria-label={`Reset name for ${pivotLabel}`}
                                    title="Reset to default name"
                                    className="opacity-0 group-hover/label:opacity-100 p-0.5 text-gray-500 hover:text-amber-400 transition-opacity shrink-0"
                                >
                                    <RotateCcw className="w-3 h-3" />
                                </button>
                            )}
                        </div>
                    )}
                </div>
                <div className="text-xs text-gray-500 pl-3 truncate">
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
    onRenamePivot,
    onResetPivotName,
}) => {
    return (
        <div className="px-3 py-3 border-b border-gray-800" data-testid="graph-pivots-section">
            <div className="flex items-center justify-between mb-2">
                <span className="text-xs uppercase tracking-wider text-gray-400 font-semibold">
                    Pivots ({pivots.length})
                </span>
                <div className="flex items-center gap-1.5">
                    {onOpenPty && (
                        <button
                            type="button"
                            onClick={onOpenPty}
                            disabled={!portalId}
                            className="px-2.5 py-1 text-xs font-medium rounded-md bg-gray-800 hover:bg-gray-700 disabled:opacity-40 disabled:pointer-events-none text-gray-200 transition-colors"
                        >
                            PTY
                        </button>
                    )}
                    {onOpenSshModal && (
                        <button
                            type="button"
                            onClick={onOpenSshModal}
                            disabled={!portalId}
                            className="px-2.5 py-1 text-xs font-medium rounded-md bg-gray-800 hover:bg-gray-700 disabled:opacity-40 disabled:pointer-events-none text-gray-200 transition-colors"
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
                            onRenamePivot={onRenamePivot}
                            onResetPivotName={onResetPivotName}
                        />
                    ))}
                </div>
            )}
        </div>
    );
};

export default GraphPivotsSection;

