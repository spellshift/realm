import React, { useEffect, useRef, useState, useCallback } from "react";
import { Terminal } from "@xterm/xterm";
import { FitAddon } from "xterm-addon-fit";
import "@xterm/xterm/css/xterm.css";
import { WebsocketMessageKind } from "../websocket";
import { AlertTriangle, CheckCircle2, Loader2, RefreshCw, SquareTerminal, XCircle } from "lucide-react";

interface PtyTerminalProps {
  portalId: number;
  pivotId?: number;
  shellId: string;
  isActive?: boolean;
  onConnectionStatusChange?: (status: "connecting" | "connected" | "disconnected") => void;
  onCloseTab?: () => void;
}

const PtyTerminal: React.FC<PtyTerminalProps> = ({
  portalId,
  pivotId,
  shellId,
  isActive,
  onConnectionStatusChange,
  onCloseTab,
}) => {
  const termRef = useRef<HTMLDivElement>(null);
  const termInstance = useRef<Terminal | null>(null);
  const wsRef = useRef<WebSocket | null>(null);
  const onConnectionStatusChangeRef = useRef(onConnectionStatusChange);

  useEffect(() => {
    onConnectionStatusChangeRef.current = onConnectionStatusChange;
  }, [onConnectionStatusChange]);

  const [connectionStatus, setConnectionStatus] = useState<"connecting" | "connected" | "disconnected">("connecting");
  const [errorMessage, setErrorMessage] = useState<string | null>(null);
  const [sessionEndedByRemote, setSessionEndedByRemote] = useState<boolean>(false);
  const [reconnectCount, setReconnectCount] = useState<number>(0);
  const [activePivotId, setActivePivotId] = useState<number | undefined>(pivotId);
  const [dimensions, setDimensions] = useState<{ cols: number; rows: number }>({ cols: 80, rows: 24 });

  const handleReconnect = useCallback(() => {
    setActivePivotId(undefined);
    setErrorMessage(null);
    setSessionEndedByRemote(false);
    setConnectionStatus("connecting");
    onConnectionStatusChangeRef.current?.("connecting");
    setReconnectCount((c) => c + 1);
  }, []);

  const handleTerminateAndClose = useCallback(() => {
    if (wsRef.current && wsRef.current.readyState === WebSocket.OPEN) {
      try {
        wsRef.current.send(JSON.stringify({ kind: "close" }));
      } catch {
        // ignore
      }
    }
    onCloseTab?.();
  }, [onCloseTab]);

  useEffect(() => {
    if (!termRef.current) return;

    const term = new Terminal({
      cursorBlink: true,
      theme: {
        background: "#14161b",
        foreground: "#e2e8f0",
        cursor: "#10b981",
      },
      fontFamily: "'Fira Code', Menlo, Monaco, 'Courier New', monospace",
      fontSize: 14,
      scrollback: 500000,
    });

    const fitAddon = new FitAddon();
    term.loadAddon(fitAddon);

    term.open(termRef.current);
    fitAddon.fit();
    termInstance.current = term;
    setDimensions({ cols: term.cols, rows: term.rows });

    const sendResize = (ws: WebSocket) => {
      if (ws.readyState === WebSocket.OPEN && term.rows > 0 && term.cols > 0) {
        ws.send(
          JSON.stringify({
            kind: "resize",
            rows: term.rows,
            cols: term.cols,
          })
        );
      }
    };

    // WebSocket Connection
    const scheme = window.location.protocol === "https:" ? "wss" : "ws";
    let wsUrl = `${scheme}://${window.location.host}/portals/pty/ws?portal_id=${portalId}&shell_id=${shellId}`;
    if (activePivotId) {
      wsUrl = `${scheme}://${window.location.host}/portals/pty/ws?pivot_id=${activePivotId}`;
    }
    const ws = new WebSocket(wsUrl);
    wsRef.current = ws;

    const resizeObserver = new ResizeObserver(() => {
      fitAddon.fit();
      setDimensions({ cols: term.cols, rows: term.rows });
      sendResize(ws);
    });
    resizeObserver.observe(termRef.current);

    ws.onopen = () => {
      setConnectionStatus("connected");
      onConnectionStatusChangeRef.current?.("connected");
      term.write(`\r\n\x1b[38;5;42m[PTY] Connected to interactive PTY session via portal #${portalId}\x1b[0m\r\n`);
      sendResize(ws);
      term.focus();
    };

    ws.onmessage = (event) => {
      try {
        const msg = JSON.parse(event.data);
        if (msg.kind === WebsocketMessageKind.Output) {
          term.write(msg.output);
        } else if (msg.kind === WebsocketMessageKind.TaskError) {
          term.write(`\x1b[31m${msg.error}\x1b[0m`);
        } else if (msg.kind === WebsocketMessageKind.Error) {
          setErrorMessage(msg.error);
          setConnectionStatus("disconnected");
          onConnectionStatusChangeRef.current?.("disconnected");
          term.options.cursorBlink = false;
          term.write(`\r\n\x1b[31m[PTY Error] ${msg.error}\x1b[0m\r\n`);
        } else if (msg.kind === WebsocketMessageKind.Status && msg.status === "closed") {
          setSessionEndedByRemote(true);
          setConnectionStatus("disconnected");
          onConnectionStatusChangeRef.current?.("disconnected");
          term.options.cursorBlink = false;
          term.write(`\r\n\x1b[33m[PTY Session Closed — Terminal is now read-only]\x1b[0m\r\n`);
        }
      } catch (e) {
        console.error("Failed to parse websocket message", e);
      }
    };

    ws.onclose = () => {
      setConnectionStatus("disconnected");
      onConnectionStatusChangeRef.current?.("disconnected");
      term.options.cursorBlink = false;
      term.write(`\r\n\x1b[33m[PTY Connection Closed]\x1b[0m\r\n`);
    };

    ws.onerror = (e) => {
      console.error("PTY WebSocket error:", e);
      setConnectionStatus("disconnected");
      onConnectionStatusChangeRef.current?.("disconnected");
    };

    // User input to WebSocket
    term.onData((data) => {
      if (ws.readyState === WebSocket.OPEN) {
        ws.send(JSON.stringify({ kind: WebsocketMessageKind.Input, input: data }));
      }
    });

    return () => {
      resizeObserver.disconnect();
      ws.close();
      term.dispose();
    };
  }, [portalId, activePivotId, shellId, reconnectCount]);

  useEffect(() => {
    if (isActive) {
      setTimeout(() => termInstance.current?.focus(), 0);
    }
  }, [isActive]);

  return (
    <div className="flex-grow flex flex-col rounded-md border border-[#2e3440] bg-[#14161b] h-full overflow-hidden">
      {/* PTY Session Header Bar */}
      <div className="flex items-center justify-between px-3 py-1.5 bg-[#1b1e26] border-b border-[#2e3440] text-xs">
        <div className="flex items-center gap-2 min-w-0">
          <SquareTerminal size={14} className="text-emerald-400 flex-shrink-0" />
          <span className="font-mono font-semibold text-gray-200 truncate">
            PTY Session{activePivotId ? ` #${activePivotId}` : ""}
          </span>
          {connectionStatus === "connected" && (
            <span className="inline-flex items-center gap-1 px-2 py-0.5 rounded-full bg-emerald-500/15 text-emerald-400 border border-emerald-500/30 font-medium">
              <CheckCircle2 size={11} />
              Connected
            </span>
          )}
          {connectionStatus === "connecting" && (
            <span className="inline-flex items-center gap-1 px-2 py-0.5 rounded-full bg-amber-500/15 text-amber-300 border border-amber-500/30 font-medium">
              <Loader2 size={11} className="animate-spin" />
              Starting PTY...
            </span>
          )}
          {connectionStatus === "disconnected" && (
            <span className="inline-flex items-center gap-1 px-2 py-0.5 rounded-full bg-red-500/20 text-red-400 border border-red-500/40 font-semibold">
              <XCircle size={11} />
              PTY Closed
            </span>
          )}
        </div>

        <div className="flex items-center gap-2">
          <span className="text-gray-500 font-mono hidden sm:inline">
            {dimensions.cols}×{dimensions.rows}
          </span>
          {portalId > 0 && connectionStatus === "disconnected" && (
            <button
              type="button"
              onClick={handleReconnect}
              className="inline-flex items-center gap-1 px-2 py-0.5 rounded bg-emerald-600/80 hover:bg-emerald-600 text-white font-medium transition-colors"
            >
              <RefreshCw size={12} />
              Spawn New PTY
            </button>
          )}
          {onCloseTab && (
            <button
              type="button"
              onClick={handleTerminateAndClose}
              className="inline-flex items-center gap-1 px-2 py-0.5 rounded bg-[#2a2f3a] hover:bg-red-600/80 text-gray-300 hover:text-white transition-colors"
              title="Close PTY tab"
            >
              Close Tab
            </button>
          )}
        </div>
      </div>

      {/* Prominent Disconnected / Dead Terminal Banner */}
      {connectionStatus === "disconnected" && (
        <div
          role="alert"
          className="flex flex-wrap items-center justify-between gap-2 px-3 py-2 bg-red-950/70 border-b border-red-500/40 text-xs text-red-200"
        >
          <div className="flex items-center gap-2">
            <AlertTriangle size={15} className="text-red-400 flex-shrink-0" />
            <div>
              <span className="font-semibold text-red-100">
                {sessionEndedByRemote
                  ? "This PTY process has exited or was closed."
                  : "PTY terminal session is disconnected."}
              </span>{" "}
              {errorMessage && <span className="text-red-300">Reason: {errorMessage}. </span>}
              <span className="text-red-300/80">
                Terminal is read-only; scrollback history is preserved below.
              </span>
            </div>
          </div>
          <div className="flex items-center gap-2">
            {portalId > 0 ? (
              <button
                type="button"
                onClick={handleReconnect}
                className="inline-flex items-center gap-1 px-2.5 py-1 rounded bg-red-500/30 hover:bg-red-500/50 border border-red-400/40 text-red-100 font-medium transition-colors"
              >
                <RefreshCw size={12} />
                Spawn New PTY
              </button>
            ) : (
              <span className="text-red-300/90 italic">Re-open a portal on the shell tab to start a new PTY</span>
            )}
            {onCloseTab && (
              <button
                type="button"
                onClick={handleTerminateAndClose}
                className="px-2.5 py-1 rounded bg-black/30 hover:bg-black/50 text-gray-200 transition-colors"
              >
                Dismiss Tab
              </button>
            )}
          </div>
        </div>
      )}

      <div ref={termRef} className="flex-grow w-full h-full p-1" style={{ height: "100%" }} />
    </div>
  );
};

export default PtyTerminal;
