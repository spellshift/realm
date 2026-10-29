import React, { useEffect, useRef, useState, useCallback } from "react";
import { Terminal } from "@xterm/xterm";
import { FitAddon } from "xterm-addon-fit";
import "@xterm/xterm/css/xterm.css";
import { WebsocketMessageKind } from "../websocket";
import { getViewportNonEmptyLineCount, transformClearAnsiForScrollback } from "../hooks/shellUtils";
import { CheckCircle2, Loader2, RefreshCw, SquareTerminal, XCircle } from "lucide-react";

interface PtyTerminalProps {
  portalId: number;
  pivotId?: number;
  shellId: string;
  isActive?: boolean;
  onConnectionStatusChange?: (status: "connecting" | "connected" | "disconnected") => void;
  onPivotCreated?: (pivotId: string) => void;
  onCloseTab?: () => void;
}

const PtyTerminal: React.FC<PtyTerminalProps> = ({
  portalId,
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
  const [reconnectCount, setReconnectCount] = useState<number>(0);
  const [displayPivotId, setDisplayPivotId] = useState<number | undefined>(pivotId);
  const [dimensions, setDimensions] = useState<{ cols: number; rows: number }>({ cols: 80, rows: 24 });

  useEffect(() => {
    if (pivotId !== undefined) {
      connectPivotIdRef.current = pivotId;
      setDisplayPivotId(pivotId);
    }
  }, [pivotId]);

  const handleReconnect = useCallback(() => {
    connectPivotIdRef.current = undefined;
    setDisplayPivotId(undefined);
    setErrorMessage(null);
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
        cursor: "#e2e8f0",
        cursorAccent: "#11131a",
        selectionBackground: "rgba(148, 163, 184, 0.28)",
      },
      fontFamily: "'JetBrains Mono', 'Fira Code', 'SFMono-Regular', Menlo, Monaco, Consolas, monospace",
      fontSize: 13.5,
      lineHeight: 1.35,
      scrollback: 10000000,
    });

    term.parser?.registerCsiHandler({ final: "J" }, (params) => {
      return params[0] === 3;
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
    let autoReconnectTimer: ReturnType<typeof setTimeout> | null = null;
    let endedRemotely = false;
    let hadFatalError = false;

    const openSocket = (isAutoRetry = false) => {
      if (cancelled) return;
      const scheme = window.location.protocol === "https:" ? "wss" : "ws";
      const targetPivotId = connectPivotIdRef.current;
      let wsUrl = `${scheme}://${window.location.host}/portals/pty/ws?portal_id=${portalId}&shell_id=${shellId}`;
      if (targetPivotId) {
        wsUrl = `${scheme}://${window.location.host}/portals/pty/ws?pivot_id=${targetPivotId}`;
      }
      const ws = new WebSocket(wsUrl);
      wsRef.current = ws;

      ws.onopen = () => {
        if (cancelled) return;
        if (!targetPivotId) {
          setConnectionStatus("connected");
          onConnectionStatusChangeRef.current?.("connected");
          term.options.cursorBlink = true;
          if (!isAutoRetry) {
            term.write(`\x1b[90m[*] PTY Active (Portal #${portalId})\x1b[0m\r\n`);
          }
        }
        sendResize(ws);
        term.focus();
      };

      ws.onmessage = (event) => {
        if (cancelled) return;
        try {
          const msg = JSON.parse(event.data);
          if (msg.kind === WebsocketMessageKind.Output) {
            if (!endedRemotely) {
              setConnectionStatus("connected");
              onConnectionStatusChangeRef.current?.("connected");
            }
            term.write(
              transformClearAnsiForScrollback(
                msg.output,
                term.rows,
                getViewportNonEmptyLineCount(term)
              )
            );
          } else if (msg.kind === WebsocketMessageKind.TaskError) {
            term.write(`\x1b[31m${msg.error}\x1b[0m`);
          } else if (msg.kind === WebsocketMessageKind.Error) {
            hadFatalError = true;
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
            if (msg.status === "connected") {
              setConnectionStatus("connected");
              onConnectionStatusChangeRef.current?.("connected");
              term.options.cursorBlink = true;
            } else if (msg.status === "closed") {
              endedRemotely = true;
              setConnectionStatus("disconnected");
              onConnectionStatusChangeRef.current?.("disconnected");
              term.options.cursorBlink = false;
              term.write(`\r\n\x1b[33m[*] PTY Closed (Read-Only)\x1b[0m\r\n`);
            }
          }
        } catch (e) {
          console.error("Failed to parse websocket message", e);
        }
      };

      ws.onclose = () => {
        if (cancelled) return;
        if (!endedRemotely && !hadFatalError && connectPivotIdRef.current) {
          setConnectionStatus("connecting");
          onConnectionStatusChangeRef.current?.("connecting");
          autoReconnectTimer = setTimeout(() => {
            openSocket(true);
          }, 2000);
          return;
        }
        setConnectionStatus("disconnected");
        onConnectionStatusChangeRef.current?.("disconnected");
        term.options.cursorBlink = false;
      };

      ws.onerror = (e) => {
        if (cancelled) return;
        console.error("PTY WebSocket error:", e);
      };
    };

    // Defer WebSocket creation so React.StrictMode's synchronous mount/unmount cycle
    // does not open a duplicate ghost ShellPivot on the backend.
    const connectTimer = setTimeout(() => {
      openSocket(false);
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
      if (autoReconnectTimer) clearTimeout(autoReconnectTimer);
      resizeObserver.disconnect();
      if (wsRef.current) {
        wsRef.current.close();
        wsRef.current = null;
      }
      term.dispose();
    };
  }, [portalId, shellId, reconnectCount]);

  useEffect(() => {
    if (isActive) {
      setTimeout(() => termInstance.current?.focus(), 0);
    }
  }, [isActive]);

  return (
    <div className="flex-grow flex flex-col bg-[#11131a] h-full overflow-hidden">
      {/* PTY Session Header Bar */}
      <div className="flex items-center justify-between px-3 py-1.5 bg-[#161922] border-b border-[#262b38] text-xs">
        <div className="flex items-center gap-2 min-w-0">
          <SquareTerminal size={14} className="text-gray-400 flex-shrink-0" />
          <span className="font-mono font-semibold text-gray-200 truncate">
            PTY Session{displayPivotId ? ` #${displayPivotId}` : ""}
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
              PTY Closed (Read-Only)
            </span>
          )}
          {connectionStatus === "disconnected" && errorMessage && (
            <span className="text-red-300 truncate">{errorMessage}</span>
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
              className="inline-flex items-center gap-1 px-2 py-0.5 rounded bg-[#2a2f3a] hover:bg-[#343b4d] text-gray-200 hover:text-white font-medium transition-colors"
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

      <div ref={termRef} className="flex-grow w-full h-full p-1" style={{ height: "100%" }} />
    </div>
  );
};

export default PtyTerminal;
