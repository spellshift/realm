import React, { useEffect, useRef, useState } from "react";
import { Terminal } from "@xterm/xterm";
import { FitAddon } from "xterm-addon-fit";
import "@xterm/xterm/css/xterm.css";
import { WebsocketMessageKind } from "../websocket";

interface SshTerminalProps {
  portalId: number;
  target: string;
  pivotId?: number;
  shellId: string;
  isActive?: boolean;
  onConnectionStatusChange?: (status: "connecting" | "connected" | "disconnected") => void;
}

const SshTerminal: React.FC<SshTerminalProps> = ({ portalId, target, pivotId, shellId, isActive, onConnectionStatusChange }) => {
  const termRef = useRef<HTMLDivElement>(null);
  const termInstance = useRef<Terminal | null>(null);
  const wsRef = useRef<WebSocket | null>(null);

  const [connectionStatus, setConnectionStatus] = useState<"connecting" | "connected" | "disconnected">("connecting");
  const [errorMessage, setErrorMessage] = useState<string | null>(null);

  useEffect(() => {
    if (!termRef.current) return;

    const term = new Terminal({
      cursorBlink: true,
      theme: { background: "#1e1e1e" },
      fontFamily: "'Fira Code', 'Courier New', monospace",
      fontSize: 14,
      scrollback: 500000,
    });

    const fitAddon = new FitAddon();
    term.loadAddon(fitAddon);

    term.open(termRef.current);
    fitAddon.fit();
    termInstance.current = term;

    // Handle Resize
    const resizeObserver = new ResizeObserver(() => {
      fitAddon.fit();
    });
    resizeObserver.observe(termRef.current);

    // WebSocket Connection
    const scheme = window.location.protocol === "https:" ? "wss" : "ws";
    let wsUrl = `${scheme}://${window.location.host}/portals/ssh/ws?portal_id=${portalId}&target=${encodeURIComponent(target)}&shell_id=${shellId}`;
    if (pivotId) {
      wsUrl = `${scheme}://${window.location.host}/portals/ssh/ws?pivot_id=${pivotId}`;
    }
    const ws = new WebSocket(wsUrl);
    wsRef.current = ws;

    ws.onopen = () => {
      setConnectionStatus("connected");
      onConnectionStatusChange?.("connected");
      term.write(`\r\n\x1b[32mConnected to SSH portal for ${target}\x1b[0m\r\n`);
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
          term.write(`\r\n\x1b[31mConnection Error: ${msg.error}\x1b[0m\r\n`);
        }
      } catch (e) {
        console.error("Failed to parse websocket message", e);
      }
    };

    ws.onclose = () => {
      setConnectionStatus("disconnected");
      onConnectionStatusChange?.("disconnected");
      term.write(`\r\n\x1b[33mConnection closed\x1b[0m\r\n`);
    };

    ws.onerror = (e) => {
      console.error("SSH WebSocket error:", e);
      setConnectionStatus("disconnected");
      onConnectionStatusChange?.("disconnected");
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
  }, [portalId, target]);

  useEffect(() => {
    if (isActive) {
      // Delay focus so it runs after the browser finishes focusing the
      // clicked tab header element.
      setTimeout(() => termInstance.current?.focus(), 0);
    }
  }, [isActive]);

  return (
    <div className="flex-grow flex flex-col relative rounded border border-[#333] h-full overflow-hidden">
        {connectionStatus !== "connected" && (
            <div className="absolute top-0 w-full z-10 bg-yellow-600/20 text-yellow-500 text-xs px-2 py-1 text-center">
                {connectionStatus === "connecting" ? "Connecting to SSH server..." : `Disconnected ${errorMessage ? "- " + errorMessage : ""}`}
            </div>
        )}
      <div ref={termRef} className="flex-grow w-full h-full" style={{ height: "100%" }} />
    </div>
  );
};

export default SshTerminal;
