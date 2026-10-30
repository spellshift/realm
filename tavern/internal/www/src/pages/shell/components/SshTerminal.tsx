import React, { useRef } from "react";
import { PortalStreamStatus, useStreamingTerminal } from "../hooks/useStreamingTerminal";

interface SshTerminalProps {
    portalId: number;
    target: string;
    pivotId?: number;
    shellId: string;
    isActive?: boolean;
    layoutKey?: string;
    fontSize?: number;
    onConnectionStatusChange?: (status: PortalStreamStatus) => void;
    onPivotCreated?: (pivotId: string) => void;
    onCloseTab?: () => void;
}

const maskTargetPassword = (target: string): string => {
    const atIdx = target.lastIndexOf("@");
    if (atIdx === -1) return target;
    const userInfo = target.slice(0, atIdx);
    const hostPart = target.slice(atIdx);
    const colonIdx = userInfo.indexOf(":");
    if (colonIdx === -1) return target;
    return `${userInfo.slice(0, colonIdx)}${hostPart}`;
};

const SshTerminal: React.FC<SshTerminalProps> = ({
    portalId,
    target,
    pivotId,
    shellId,
    isActive,
    layoutKey,
    fontSize,
    onConnectionStatusChange,
    onPivotCreated,
}) => {
    const displayTarget = maskTargetPassword(target);
    const portalIdRef = useRef<number>(portalId);
    portalIdRef.current = portalId;

    const { termRef } = useStreamingTerminal({
        pivotId,
        isActive,
        layoutKey,
        fontSize,
        closedBannerLabel: "SSH Closed (Read-Only)",
        dependencyKey: `ssh:${target}:${displayTarget}:${shellId}`,
        buildWsUrl: (scheme, host, activePivotId) => {
            if (activePivotId) {
                return `${scheme}://${host}/portals/ssh/ws?pivot_id=${activePivotId}`;
            }
            return `${scheme}://${host}/portals/ssh/ws?portal_id=${portalIdRef.current}&target=${encodeURIComponent(target)}&shell_id=${shellId}`;
        },
        onSocketOpen: ({ term, ws, isAutoRetry, targetPivotId, sendResize, reportStatus }) => {
            reportStatus("connecting");
            if (!isAutoRetry && !targetPivotId) {
                term.write(
                    `\x1b[90m[*] Connecting to ${displayTarget} via portal #${portalIdRef.current}...\x1b[0m\r\n`
                );
            }
            sendResize(ws, true);
        },
        onStatusConnected: ({ term, ws, sendResize }) => {
            sendResize(ws, true);
            term.focus();
        },
        onConnectionStatusChange,
        onPivotCreated,
    });

    return (
        <div className="flex-grow flex flex-col bg-[#11131a] h-full min-w-0 min-h-0 overflow-hidden">
            <div ref={termRef} className="flex-grow w-full h-full min-w-0 min-h-0 overflow-hidden p-1" style={{ height: "100%" }} />
        </div>
    );
};

export default SshTerminal;
