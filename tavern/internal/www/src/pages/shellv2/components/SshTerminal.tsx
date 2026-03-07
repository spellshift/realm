import React, { useEffect, useRef, useState } from "react";
import { Terminal } from "@xterm/xterm";
import { FitAddon } from "xterm-addon-fit";
import "@xterm/xterm/css/xterm.css";
import AlertError from "../../../components/tavern-base-ui/AlertError";

interface SshTerminalProps {
    portalId: number;
    sessionId: string;
    target: string;
}

const SshTerminal: React.FC<SshTerminalProps> = ({ portalId, sessionId, target }) => {
    const termRef = useRef<HTMLDivElement>(null);
    const termInstance = useRef<Terminal | null>(null);
    const wsRef = useRef<WebSocket | null>(null);
    const [error, setError] = useState<string | null>(null);

    useEffect(() => {
        if (!termRef.current) return;

        let reconnectTimeout: NodeJS.Timeout | null = null;
        let isDisposed = false;
        let seqId = 0;

        const term = new Terminal({
            cursorBlink: true,
            theme: {
                background: "#1e1e1e",
                foreground: "#d4d4d4",
            },
            fontFamily: 'Menlo, Monaco, "Courier New", monospace',
            fontSize: 14,
            macOptionIsMeta: true,
        });

        const fitAddon = new FitAddon();
        term.loadAddon(fitAddon);
        term.open(termRef.current);
        fitAddon.fit();
        termInstance.current = term;

        const connect = () => {
            if (isDisposed) return;

            const protocol = window.location.protocol === "https:" ? "wss" : "ws";
            const url = `${protocol}://${window.location.host}/portals/ssh/ws?portal_id=${portalId}&session_id=${sessionId}&seq_id=${seqId}&target=${encodeURIComponent(target)}`;

            const ws = new WebSocket(url);
            wsRef.current = ws;

            term.writeln(`Connecting to ${target} via portal ${portalId}...`);

            ws.onopen = () => {
                term.writeln(`\x1b[32mConnected!\x1b[0m\r\n`);
                setError(null);
            };

            ws.onmessage = (event) => {
                // Check if the data is a Blob
                if (event.data instanceof Blob) {
                    const reader = new FileReader();
                    reader.onload = () => {
                        const text = reader.result as string;
                        term.write(text);
                    };
                    reader.readAsText(event.data);
                } else {
                    term.write(event.data);
                }
            };

            ws.onclose = (event) => {
                if (isDisposed) return;

                term.writeln(`\r\n\x1b[31mConnection closed (Code: ${event.code}, Reason: ${event.reason})\x1b[0m`);
                if (event.code !== 1000) {
                    term.writeln(`\r\n\x1b[33mReconnecting in 3 seconds...\x1b[0m`);
                    reconnectTimeout = setTimeout(connect, 3000);
                }
            };

            ws.onerror = (e) => {
                console.error("SSH WebSocket Error", e);
                // term.writeln(`\r\n\x1b[31mWebSocket Error.\x1b[0m`);
                // We let onclose handle the reconnection logic
            };
        };

        connect();

        const handleResize = () => fitAddon.fit();
        window.addEventListener("resize", handleResize);

        term.onData((data) => {
            if (wsRef.current && wsRef.current.readyState === WebSocket.OPEN) {
                seqId++; // Increment seq_id on sending messages
                wsRef.current.send(data);
            }
        });

        return () => {
            isDisposed = true;
            if (reconnectTimeout) {
                clearTimeout(reconnectTimeout);
            }
            window.removeEventListener("resize", handleResize);
            if (wsRef.current) {
                wsRef.current.close();
            }
            term.dispose();
        };
    }, [portalId, sessionId, target]);

    if (error) {
        return (
            <div className="h-full flex-grow relative border border-[#333] p-5 flex items-start">
                <div className="w-full">
                   <AlertError label="SSH Connection Failed" details={error} />
                   <div className="mt-4 border border-[#333] rounded overflow-hidden" style={{height: "calc(100vh - 250px)"}}>
                        <div ref={termRef} style={{ height: "100%", width: "100%" }} />
                   </div>
                </div>
            </div>
        );
    }

    return (
        <div className="h-full flex-grow rounded overflow-hidden relative border border-[#333]">
            <div ref={termRef} style={{ height: "100%", width: "100%" }} />
        </div>
    );
};

export default SshTerminal;
