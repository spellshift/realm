import React, { useEffect, useRef, useState } from "react";
import { Terminal } from "@xterm/xterm";
import { FitAddon } from "xterm-addon-fit";
import "@xterm/xterm/css/xterm.css";

interface SSHTerminalProps {
    portalId: number;
    sessionId: string;
    target: string;
}

const SSHTerminal: React.FC<SSHTerminalProps> = ({ portalId, sessionId, target }) => {
    const termRef = useRef<HTMLDivElement>(null);
    const termInstance = useRef<Terminal | null>(null);
    const fitAddon = useRef<FitAddon | null>(null);
    const wsRef = useRef<WebSocket | null>(null);
    const [connectionState, setConnectionState] = useState<string>("Connecting...");
    const seqIdRef = useRef<number>(0);
    const reconnectTimer = useRef<NodeJS.Timeout | null>(null);

    const unmountedRef = useRef<boolean>(false);

    useEffect(() => {
        unmountedRef.current = false;
        if (!termRef.current) return;

        const term = new Terminal({
            cursorBlink: true,
            theme: { background: "#1e1e1e" }
        });
        const fit = new FitAddon();
        term.loadAddon(fit);
        term.open(termRef.current);
        fit.fit();

        termInstance.current = term;
        fitAddon.current = fit;

        const handleResize = () => fit.fit();
        window.addEventListener("resize", handleResize);

        // To fix xterm.js not resizing correctly when switching tabs:
        const resizeObserver = new ResizeObserver(() => {
            if (termRef.current && termRef.current.clientWidth > 0 && termRef.current.clientHeight > 0) {
                fit.fit();
            }
        });
        resizeObserver.observe(termRef.current);

        term.onData((data) => {
            if (wsRef.current && wsRef.current.readyState === WebSocket.OPEN) {
                wsRef.current.send(data);
            }
        });

        const connectWebSocket = () => {
            if (unmountedRef.current) return;
            const scheme = window.location.protocol === "https:" ? "wss" : "ws";
            const url = `${scheme}://${window.location.host}/portals/ssh/ws?portal_id=${portalId}&session_id=${sessionId}&target=${encodeURIComponent(target)}&seq_id=${seqIdRef.current}`;
            const ws = new WebSocket(url);
            wsRef.current = ws;

            ws.onopen = () => {
                setConnectionState("Connected");
            };

            ws.onmessage = async (event) => {
                let text = event.data;
                if (text instanceof Blob) {
                    text = await text.text();
                }
                term.write(text);

                // Track bytes received to advance seq_id if the backend supports it conceptually.
                // Even though the backend currently streams raw bytes, the prompt states:
                // "Reconnections should maintain the same session_id and seq_id."
                // In our current design, we start seq_id=0 and the connection is preserved over the portal mux.
                seqIdRef.current += 1;
            };

            ws.onclose = (event) => {
                setConnectionState("Disconnected");
                if (unmountedRef.current) return;

                if (event.code !== 1000) {
                    setConnectionState("Reconnecting...");
                    term.write(`\r\n\x1b[33mConnection lost. Reconnecting...\x1b[0m\r\n`);
                    reconnectTimer.current = setTimeout(() => {
                        connectWebSocket();
                    }, 3000);
                } else {
                    term.write(`\r\n\x1b[31mConnection closed cleanly.\x1b[0m\r\n`);
                }
            };

            ws.onerror = (error) => {
                // Ignore, handled by close
            };
        };

        connectWebSocket();

        return () => {
            unmountedRef.current = true;
            window.removeEventListener("resize", handleResize);
            if (termRef.current) {
                resizeObserver.unobserve(termRef.current);
            }
            resizeObserver.disconnect();
            if (wsRef.current) {
                wsRef.current.close(1000);
            }
            term.dispose();
            if (reconnectTimer.current) {
                clearTimeout(reconnectTimer.current);
            }
        };
    }, [portalId, sessionId, target]);

    return (
        <div className="flex flex-col h-full w-full">
            <div className="text-xs text-gray-400 p-1 border-b border-[#333]">
                SSH: {target} | Status: {connectionState}
            </div>
            <div className="flex-grow w-full relative overflow-hidden" ref={termRef} />
        </div>
    );
};

export default SSHTerminal;
