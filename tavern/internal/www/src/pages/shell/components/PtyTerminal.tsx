import React, { useEffect, useRef } from "react";
import { Terminal } from "@xterm/xterm";
import { FitAddon } from "xterm-addon-fit";
import "@xterm/xterm/css/xterm.css";
import { WebsocketMessageKind } from "../websocket";
import { getViewportNonEmptyLineCount, transformClearAnsiForScrollback } from "../hooks/shellUtils";
import {
  WEBSOCKET_CONNECT_TIMEOUT_MS,
  getExponentialBackoffDelay,
} from "../../../lib/browser-adapter";

interface PtyTerminalProps {
  portalId: number;
  pivotId?: number;
  shellId: string;
  isActive?: boolean;
  onConnectionStatusChange?: (status: "connecting" | "connected" | "disconnected") => void;
  onPivotCreated?: (pivotId: string) => void;
  onLastCommandChange?: (command: string) => void;
  onCloseTab?: () => void;
}

const PtyTerminal: React.FC<PtyTerminalProps> = ({
  portalId,
  pivotId,
  shellId,
  isActive,
  onConnectionStatusChange,
  onPivotCreated,
  onLastCommandChange,
}) => {
  const termRef = useRef<HTMLDivElement>(null);
  const termInstance = useRef<Terminal | null>(null);
  const wsRef = useRef<WebSocket | null>(null);
  const onConnectionStatusChangeRef = useRef(onConnectionStatusChange);
  const onPivotCreatedRef = useRef(onPivotCreated);
  const onLastCommandChangeRef = useRef(onLastCommandChange);
  const connectPivotIdRef = useRef<number | undefined>(pivotId);
  const inputLineBufferRef = useRef<string>("");

  useEffect(() => {
    onConnectionStatusChangeRef.current = onConnectionStatusChange;
  }, [onConnectionStatusChange]);

  useEffect(() => {
    onPivotCreatedRef.current = onPivotCreated;
  }, [onPivotCreated]);

  useEffect(() => {
    onLastCommandChangeRef.current = onLastCommandChange;
  }, [onLastCommandChange]);

  useEffect(() => {
    if (pivotId !== undefined) {
      connectPivotIdRef.current = pivotId;
    }
  }, [pivotId]);

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
      sendResize(wsRef.current);
    });
    resizeObserver.observe(termRef.current);

    let cancelled = false;
    let autoReconnectTimer: ReturnType<typeof setTimeout> | null = null;
    let connectTimeoutTimer: ReturnType<typeof setTimeout> | null = null;
    let reconnectAttempts = 0;
    let endedRemotely = false;
    let hadFatalError = false;

    const clearSocketTimers = () => {
      if (autoReconnectTimer) {
        clearTimeout(autoReconnectTimer);
        autoReconnectTimer = null;
      }
      if (connectTimeoutTimer) {
        clearTimeout(connectTimeoutTimer);
        connectTimeoutTimer = null;
      }
    };

    const detachSocket = (ws: WebSocket | null) => {
      if (!ws) return;
      if (wsRef.current === ws) {
        wsRef.current = null;
      }
      ws.onopen = null;
      ws.onmessage = null;
      ws.onclose = null;
      ws.onerror = null;
      try {
        if (ws.readyState === WebSocket.OPEN || ws.readyState === WebSocket.CONNECTING) {
          ws.close();
        }
      } catch {
        // Ignore close errors on stale socket
      }
    };

    const scheduleReconnect = () => {
      if (cancelled || endedRemotely || hadFatalError) return;
      if (autoReconnectTimer) return;
      onConnectionStatusChangeRef.current?.("connecting");
      const delay = getExponentialBackoffDelay(reconnectAttempts);
      reconnectAttempts++;
      autoReconnectTimer = setTimeout(() => {
        autoReconnectTimer = null;
        openSocket(true);
      }, delay);
    };

    const openSocket = (isAutoRetry = false) => {
      if (cancelled) return;
      clearSocketTimers();
      detachSocket(wsRef.current);

      const scheme = window.location.protocol === "https:" ? "wss" : "ws";
      const targetPivotId = connectPivotIdRef.current;
      let wsUrl = `${scheme}://${window.location.host}/portals/pty/ws?portal_id=${portalId}&shell_id=${shellId}`;
      if (targetPivotId) {
        wsUrl = `${scheme}://${window.location.host}/portals/pty/ws?pivot_id=${targetPivotId}`;
      }
      const ws = new WebSocket(wsUrl);
      wsRef.current = ws;

      connectTimeoutTimer = setTimeout(() => {
        if (cancelled || wsRef.current !== ws) return;
        if (ws.readyState === WebSocket.CONNECTING) {
          detachSocket(ws);
          scheduleReconnect();
        }
      }, WEBSOCKET_CONNECT_TIMEOUT_MS);

      ws.onopen = () => {
        if (cancelled || wsRef.current !== ws) return;
        if (connectTimeoutTimer) {
          clearTimeout(connectTimeoutTimer);
          connectTimeoutTimer = null;
        }
        reconnectAttempts = 0;
        if (!targetPivotId) {
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
        if (cancelled || wsRef.current !== ws) return;
        try {
          const msg = JSON.parse(event.data);
          if (msg.kind === WebsocketMessageKind.Output) {
            if (!endedRemotely) {
              reconnectAttempts = 0;
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
            onConnectionStatusChangeRef.current?.("disconnected");
            term.options.cursorBlink = false;
            term.write(`\r\n\x1b[31m[-] ${msg.error}\x1b[0m\r\n`);
          } else if (msg.kind === WebsocketMessageKind.Status) {
            if (msg.pivot_id) {
              connectPivotIdRef.current = msg.pivot_id;
              onPivotCreatedRef.current?.(String(msg.pivot_id));
            }
            if (msg.status === "connected") {
              reconnectAttempts = 0;
              onConnectionStatusChangeRef.current?.("connected");
              term.options.cursorBlink = true;
            } else if (msg.status === "closed") {
              endedRemotely = true;
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
        if (cancelled || wsRef.current !== ws) return;
        if (connectTimeoutTimer) {
          clearTimeout(connectTimeoutTimer);
          connectTimeoutTimer = null;
        }
        if (!endedRemotely && !hadFatalError) {
          scheduleReconnect();
          return;
        }
        onConnectionStatusChangeRef.current?.("disconnected");
        term.options.cursorBlink = false;
      };

      ws.onerror = (e) => {
        if (cancelled || wsRef.current !== ws) return;
        console.error("PTY WebSocket error:", e);
      };
    };

    // Defer WebSocket creation so React.StrictMode's synchronous mount/unmount cycle
    // does not open a duplicate ghost ShellPivot on the backend.
    const connectTimer = setTimeout(() => {
      openSocket(false);
    }, 0);

    // User input to WebSocket + track last submitted command for the tab label
    term.onData((data) => {
      if (wsRef.current && wsRef.current.readyState === WebSocket.OPEN) {
        wsRef.current.send(JSON.stringify({ kind: WebsocketMessageKind.Input, input: data }));
      }
      if (data === "\x03" || data === "\x15") {
        inputLineBufferRef.current = "";
        return;
      }
      if (data === "\x7f" || data === "\b") {
        inputLineBufferRef.current = inputLineBufferRef.current.slice(0, -1);
        return;
      }
      if (data.startsWith("\x1b")) {
        return;
      }
      for (const ch of data) {
        if (ch === "\r" || ch === "\n") {
          const cmd = inputLineBufferRef.current.trim();
          if (cmd && cmd.toLowerCase() !== "exit") {
            const truncated = cmd.length > 24 ? `${cmd.slice(0, 24)}...` : cmd;
            onLastCommandChangeRef.current?.(truncated);
          }
          inputLineBufferRef.current = "";
        } else if (ch.charCodeAt(0) >= 32) {
          inputLineBufferRef.current += ch;
        }
      }
    });

    return () => {
      cancelled = true;
      clearTimeout(connectTimer);
      clearSocketTimers();
      resizeObserver.disconnect();
      detachSocket(wsRef.current);
      term.dispose();
    };
  }, [portalId, shellId]);

  useEffect(() => {
    if (isActive) {
      setTimeout(() => termInstance.current?.focus(), 0);
    }
  }, [isActive]);

  return (
    <div className="flex-grow flex flex-col bg-[#11131a] h-full overflow-hidden">
      <div ref={termRef} className="flex-grow w-full h-full p-1" style={{ height: "100%" }} />
    </div>
  );
};

export default PtyTerminal;
