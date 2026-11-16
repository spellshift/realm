import React, { useRef } from "react";
import { PortalStreamStatus, useStreamingTerminal } from "../hooks/useStreamingTerminal";

interface PtyTerminalProps {
    portalId: number;
    pivotId?: number;
    shellId: string;
    isActive?: boolean;
    layoutKey?: string;
    fontSize?: number;
    onConnectionStatusChange?: (status: PortalStreamStatus) => void;
    onPivotCreated?: (pivotId: string) => void;
    onLastCommandChange?: (command: string) => void;
    onCloseTab?: () => void;
}

const PtyTerminal: React.FC<PtyTerminalProps> = ({
    portalId,
    pivotId,
    shellId,
    isActive,
    layoutKey,
    fontSize,
    onConnectionStatusChange,
    onPivotCreated,
}) => {
    const portalIdRef = useRef<number>(portalId);
    portalIdRef.current = portalId;

    const { termRef } = useStreamingTerminal({
        pivotId,
        isActive,
        layoutKey,
        fontSize,
        closedBannerLabel: "PTY Closed (Read-Only)",
        dependencyKey: `pty:${shellId}`,
        buildWsUrl: (scheme, host, activePivotId, dims) => {
            if (activePivotId) {
                return `${scheme}://${host}/portals/pty/ws?pivot_id=${activePivotId}`;
            }
            const sizeParams =
                dims && dims.rows > 0 && dims.cols > 0
                    ? `&rows=${dims.rows}&cols=${dims.cols}`
                    : "";
            return `${scheme}://${host}/portals/pty/ws?portal_id=${portalIdRef.current}&shell_id=${shellId}${sizeParams}`;
        },
        onSocketOpen: ({ term, ws, isAutoRetry, targetPivotId, sendResize, reportStatus }) => {
            if (!targetPivotId) {
                reportStatus("connected");
                term.options.cursorBlink = true;
                if (!isAutoRetry) {
                    term.write(`\x1b[90m[*] PTY Active (Portal #${portalIdRef.current})\x1b[0m\r\n`);
                }
            }
            sendResize(ws, Boolean(targetPivotId));
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

export default PtyTerminal;
