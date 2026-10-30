import React from "react";
import { Link } from "react-router-dom";
import { KnowledgeShellItem } from "../../hooks/useShellData";

export interface GraphShellsSectionProps {
    shells: KnowledgeShellItem[];
    currentShellId?: string;
}

const ShellRowAction: React.FC<{ shellId: string; isCurrent: boolean }> = ({
    shellId,
    isCurrent,
}) => {
    if (isCurrent) {
        return <span className="text-[11px] text-gray-500">Current</span>;
    }

    return (
        <Link
            to={`/shells/${shellId}`}
            className="text-[11px] text-gray-200 hover:text-white px-2 py-0.5 rounded bg-gray-800 hover:bg-gray-700 transition-colors"
        >
            Switch
        </Link>
    );
};

const formatShellSubtext = (shell: KnowledgeShellItem): string => {
    const user = shell.beaconPrincipal || "";
    const host = shell.hostName || "";
    if (user && host) {
        return `${user}@${host}`;
    }
    return user || host;
};

const ShellRow: React.FC<{
    shell: KnowledgeShellItem;
    currentShellId?: string;
}> = ({ shell, currentShellId }) => {
    const isCurrent = Boolean(currentShellId && String(shell.id) === String(currentShellId));
    const isActive = !shell.closedAt;
    const dotClass = isActive ? "bg-emerald-400" : "bg-gray-600";
    const subtext = formatShellSubtext(shell);

    return (
        <div className="py-2 flex items-center justify-between gap-2">
            <div className="min-w-0 flex-1">
                <div className="flex items-center gap-1.5 text-xs text-gray-200 font-mono truncate">
                    <span className={`w-1.5 h-1.5 rounded-full shrink-0 ${dotClass}`} />
                    <span>{shell.beaconName}</span>
                </div>
                {subtext && (
                    <div className="text-[11px] text-gray-500 pl-3 font-mono truncate">
                        {subtext}
                    </div>
                )}
            </div>
            <ShellRowAction shellId={shell.id} isCurrent={isCurrent} />
        </div>
    );
};

export const GraphShellsSection: React.FC<GraphShellsSectionProps> = ({
    shells,
    currentShellId,
}) => {
    return (
        <div className="px-4 py-3 border-b border-gray-800" data-testid="graph-shells-section">
            <div className="flex items-center justify-between mb-1">
                <span className="text-[11px] uppercase tracking-wider text-gray-400 font-medium">
                    Shells ({shells.length})
                </span>
            </div>
            {shells.length === 0 ? (
                <div className="py-2 text-xs text-gray-500">No shells available.</div>
            ) : (
                <div className="divide-y divide-gray-800/60">
                    {shells.map((s) => (
                        <ShellRow key={s.id} shell={s} currentShellId={currentShellId} />
                    ))}
                </div>
            )}
        </div>
    );
};

export default GraphShellsSection;

