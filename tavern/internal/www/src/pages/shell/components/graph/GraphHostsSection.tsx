import React from "react";
import { Link } from "react-router-dom";
import { KnowledgeHostItem } from "../../hooks/useShellData";
import { formatSubnetCidr } from "../../utils/networkGraphUtils";

export interface GraphHostsSectionProps {
    hosts: KnowledgeHostItem[];
    onOpenSshWithTarget: (target: string) => void;
}

const HostTitleLink: React.FC<{ host: KnowledgeHostItem }> = ({ host }) => {
    const detailedHostId = host.hostId || (!host.networkHostId ? host.id : undefined);
    if (detailedHostId) {
        return (
            <Link
                to={`/hosts/${detailedHostId}`}
                className="text-xs font-medium text-gray-200 hover:text-white truncate transition-colors"
            >
                {host.name}
            </Link>
        );
    }

    return <span className="text-xs font-medium text-gray-200 truncate">{host.name}</span>;
};

const HostRow: React.FC<{
    host: KnowledgeHostItem;
    onOpenSshWithTarget: (target: string) => void;
}> = ({ host, onOpenSshWithTarget }) => {
    const ip = host.ipAddress || host.primaryIP || "";
    const subnet = host.subnet || formatSubnetCidr(ip);
    const ports = host.openPorts || [];
    const targetAddr = ip || host.hostname || host.name;
    const defaultPort = ports.includes(22) || ports.length === 0 ? 22 : ports[0];

    return (
        <div className="py-2 flex items-center justify-between gap-2">
            <div className="min-w-0 flex-1">
                <div className="flex items-center gap-2">
                    <HostTitleLink host={host} />
                    {host.hostname && host.hostname !== host.name && (
                        <span className="text-[11px] text-gray-500 font-mono truncate">
                            ({host.hostname})
                        </span>
                    )}
                </div>
                <div className="flex items-center flex-wrap gap-x-2 gap-y-0.5 text-[11px] text-gray-400 font-mono mt-0.5">
                    {ip && <span>{ip}</span>}
                    {subnet && <span className="text-gray-500">{subnet}</span>}
                    {ports.length > 0 && (
                        <span className="text-gray-400">
                            ports: {ports.map((p) => `:${p}`).join(", ")}
                        </span>
                    )}
                </div>
            </div>
            {targetAddr && (
                <button
                    type="button"
                    onClick={() => onOpenSshWithTarget(`${targetAddr}:${defaultPort}`)}
                    className="text-[11px] text-gray-200 hover:text-white px-2 py-0.5 rounded bg-gray-800 hover:bg-gray-700 shrink-0 transition-colors"
                >
                    Pivot
                </button>
            )}
        </div>
    );
};

export const GraphHostsSection: React.FC<GraphHostsSectionProps> = ({
    hosts,
    onOpenSshWithTarget,
}) => {
    const adjacentHosts = hosts.filter((h) => !h.isCurrentHost);

    return (
        <div className="px-4 py-3 border-b border-gray-800" data-testid="graph-hosts-section">
            <div className="flex items-center justify-between mb-1">
                <span className="text-[11px] uppercase tracking-wider text-gray-400 font-medium">
                    Adjacent Hosts ({adjacentHosts.length})
                </span>
            </div>
            {adjacentHosts.length === 0 ? (
                <div className="py-2 text-xs text-gray-500">No adjacent hosts discovered yet.</div>
            ) : (
                <div className="divide-y divide-gray-800/60">
                    {adjacentHosts.map((host) => (
                        <HostRow
                            key={host.id}
                            host={host}
                            onOpenSshWithTarget={onOpenSshWithTarget}
                        />
                    ))}
                </div>
            )}
        </div>
    );
};

export default GraphHostsSection;

