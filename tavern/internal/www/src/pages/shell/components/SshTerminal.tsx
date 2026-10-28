import React, { useEffect, useRef, useState, useCallback } from "react";
import { Terminal } from "@xterm/xterm";
import { FitAddon } from "xterm-addon-fit";
import "@xterm/xterm/css/xterm.css";
import { WebsocketMessageKind } from "../websocket";
import { AlertTriangle, CheckCircle2, Loader2, RefreshCw, Terminal as TerminalIcon, XCircle } from "lucide-react";

interface SshTerminalProps {
  portalId: number;
  target: string;
  pivotId?: number;
  shellId: string;
  isActive?: boolean;
  onConnectionStatusChange?: (status: "connecting" | "connected" | "disconnected") => void;
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
  onConnectionStatusChange,
  onPivotCreated,
  onCloseTab,
}) => {
  const termRef = useRef<HTMLDivElement>(null);
  const termInstance = useRef<Terminal | null>(null);
  const wsRef = useRef<WebSocket | null>(null);
  const onConnectionStatusChangeRef = useRef(onConnectionStatusChange);
  const onPivotCreatedRef = useRef(onPivotCreated);
  const connectPivotIdRef = useRef<number | undefined>(pivotId);

  useEffect(() => {
    onConnectionStatusChangeRef.current = onConnectionStatusChange;
  }, [onConnectionStatusChange]);

  useEffect(() => {
    onPivotCreatedRef.current = onPivotCreated;
  }, [onPivotCreated]);

  const [connectionStatus, setConnectionStatus] = useState<"connecting" | "connected" | "disconnected">("connecting");
  const [errorMessage, setErrorMessage] = useState<string | null>(null);
  const [sessionEndedByRemote, setSessionEndedByRemote] = useState<boolean>(false);
  const [reconnectCount, setReconnectCount] = useState<number>(0);
  const [displayPivotId, setDisplayPivotId] = useState<number | undefined>(pivotId);
  const [dimensions, setDimensions] = useState<{ cols: number; rows: number }>({ cols: 80, rows: 24 });

  useEffect(() => {
    if (pivotId !== undefined) {
      connectPivotIdRef.current = pivotId;
      setDisplayPivotId(pivotId);
    }
  }, [pivotId]);

  const displayTarget = maskTargetPassword(target);

  const handleReconnect = useCallback(() => {
    // When reconnecting after a remote close, start a fresh SSH session over the active portal
    connectPivotIdRef.current = undefined;
    setDisplayPivotId(undefined);
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
        background: "#11131a",
        foreground: "#e2e8f0",
        cursor: "#a855f7",
        selectionBackground: "rgba(168, 85, 247, 0.25)",
      },
      fontFamily: "'JetBrains Mono', 'Fira Code', 'SFMono-Regular', Menlo, Monaco, Consolas, monospace",
      fontSize: 13.5,
      lineHeight: 1.35,
      scrollback: 500000,
    });

    const fitAddon = new FitAddon();
    term.loadAddon(fitAddon);

    term.open(termRef.current);
    fitAddon.fit();
    termInstance.current = term;
    setDimensions({ cols: term.cols, rows: term.rows });

    const sendResize = (ws: WebSocket | null) => {
      if (ws && ws.readyState === WebSocket.OPEN && term.rows > 0 && term.cols > 0) {
        ws.send(
          JSON.stringify({
            kind: "resize",
            rows: term.rows,
            cols: term.cols,
          })
        );
      }
    };

    const resizeObserver = new ResizeObserver(() => {
      fitAddon.fit();
      setDimensions({ cols: term.cols, rows: term.rows });
      sendResize(wsRef.current);
    });
    resizeObserver.observe(termRef.current);

    let cancelled = false;
    // Defer WebSocket creation so React.StrictMode's synchronous mount/unmount cycle
    // does not open a duplicate ghost ShellPivot on the backend.
    const connectTimer = setTimeout(() => {
      if (cancelled) return;
      const scheme = window.location.protocol === "https:" ? "wss" : "ws";
      const targetPivotId = connectPivotIdRef.current;
      let wsUrl = `${scheme}://${window.location.host}/portals/ssh/ws?portal_id=${portalId}&target=${encodeURIComponent(target)}&shell_id=${shellId}`;
      if (targetPivotId) {
        wsUrl = `${scheme}://${window.location.host}/portals/ssh/ws?pivot_id=${targetPivotId}`;
      }
      const ws = new WebSocket(wsUrl);
      wsRef.current = ws;

      ws.onopen = () => {
        setConnectionStatus("connecting");
        onConnectionStatusChangeRef.current?.("connecting");
        term.write(`\x1b[90m[*] Connecting to ${displayTarget} via portal #${portalId}...\x1b[0m\r\n`);
        sendResize(ws);
      };

      ws.onmessage = (event) => {
        try {
          const msg = JSON.parse(event.data);
          if (msg.kind === WebsocketMessageKind.Output) {
            setConnectionStatus("connected");
            onConnectionStatusChangeRef.current?.("connected");
            term.write(msg.output);
          } else if (msg.kind === WebsocketMessageKind.TaskError) {
            term.write(`\x1b[31m${msg.error}\x1b[0m`);
          } else if (msg.kind === WebsocketMessageKind.Error) {
            setErrorMessage(msg.error);
            setConnectionStatus("disconnected");
            onConnectionStatusChangeRef.current?.("disconnected");
            term.options.cursorBlink = false;
            term.write(`\r\n\x1b[31m[-] ${msg.error}\x1b[0m\r\n`);
          } else if (msg.kind === WebsocketMessageKind.Status) {
            if (msg.pivot_id) {
              connectPivotIdRef.current = msg.pivot_id;
              setDisplayPivotId(msg.pivot_id);
              onPivotCreatedRef.current?.(String(msg.pivot_id));
            }
            if (msg.status === "connecting") {
              setConnectionStatus("connecting");
              onConnectionStatusChangeRef.current?.("connecting");
            } else if (msg.status === "connected") {
              setConnectionStatus("connected");
              onConnectionStatusChangeRef.current?.("connected");
              sendResize(ws);
              term.focus();
            } else if (msg.status === "closed") {
              setSessionEndedByRemote(true);
              setConnectionStatus("disconnected");
              onConnectionStatusChangeRef.current?.("disconnected");
              term.options.cursorBlink = false;
              term.write(`\r\n\x1b[33m[*] SSH Closed (Read-Only)\x1b[0m\r\n`);
            }
          }
        } catch (e) {
          console.error("Failed to parse websocket message", e);
        }
      };

      ws.onclose = () => {
        if (cancelled) return;
        setConnectionStatus("disconnected");
        onConnectionStatusChangeRef.current?.("disconnected");
        term.options.cursorBlink = false;
      };

      ws.onerror = (e) => {
        if (cancelled) return;
        console.error("SSH WebSocket error:", e);
        setConnectionStatus("disconnected");
        onConnectionStatusChangeRef.current?.("disconnected");
      };
    }, 0);

    // User input to WebSocket
    term.onData((data) => {
      if (wsRef.current && wsRef.current.readyState === WebSocket.OPEN) {
        wsRef.current.send(JSON.stringify({ kind: WebsocketMessageKind.Input, input: data }));
      }
    });

    return () => {
      cancelled = true;
      clearTimeout(connectTimer);
      resizeObserver.disconnect();
      if (wsRef.current) {
        wsRef.current.close();
        wsRef.current = null;
      }
      term.dispose();
    };
  }, [portalId, target, displayTarget, shellId, reconnectCount]);

  useEffect(() => {
    if (isActive) {
      setTimeout(() => termInstance.current?.focus(), 0);
    }
  }, [isActive]);

  return (
    <div className="flex-grow flex flex-col rounded-md border border-[#262b38] bg-[#11131a] h-full overflow-hidden">
      {/* Pivot Session Header Bar */}
      <div className="flex items-center justify-between px-3 py-1.5 bg-[#161922] border-b border-[#262b38] text-xs">
        <div className="flex items-center gap-2 min-w-0">
          <TerminalIcon size={14} className="text-purple-400 flex-shrink-0" />
          <span className="font-mono font-semibold text-gray-200 truncate">
            SSH • {displayTarget}{displayPivotId ? ` (#${displayPivotId})` : ""}
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
              Negotiating SSH...
            </span>
          )}
          {connectionStatus === "disconnected" && (
            <span className="inline-flex items-center gap-1 px-2 py-0.5 rounded-full bg-red-500/20 text-red-400 border border-red-500/40 font-semibold">
              <XCircle size={11} />
              Session Ended
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
              className="inline-flex items-center gap-1 px-2 py-0.5 rounded bg-purple-600/80 hover:bg-purple-600 text-white font-medium transition-colors"
            >
              <RefreshCw size={12} />
              Reconnect
            </button>
          )}
          {onCloseTab && (
            <button
              type="button"
              onClick={handleTerminateAndClose}
              className="inline-flex items-center gap-1 px-2 py-0.5 rounded bg-[#2a2f3a] hover:bg-red-600/80 text-gray-300 hover:text-white transition-colors"
              title="Close SSH tab"
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
                  ? `SSH session to ${displayTarget} was closed.`
                  : `SSH terminal disconnected from ${displayTarget}.`}
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
                Reconnect SSH
              </button>
            ) : (
              <span className="text-red-300/90 italic">Re-open a portal on the shell tab to reconnect</span>
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

export default SshTerminal;
