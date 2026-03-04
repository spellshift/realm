import { useEffect, useRef, useState } from "react";
import { Terminal } from "@xterm/xterm";
import { FitAddon } from "xterm-addon-fit";
import "@xterm/xterm/css/xterm.css";
import { v4 as uuidv4 } from "uuid";

interface SshTerminalProps {
    portalId: number;
}

const SshTerminal = ({ portalId }: SshTerminalProps) => {
    const termRef = useRef<HTMLDivElement>(null);
    const termInstance = useRef<Terminal | null>(null);
    const wsRef = useRef<WebSocket | null>(null);
    const streamIdRef = useRef<string>(uuidv4());
    const sequenceIdRef = useRef<number>(0);

    const [status, setStatus] = useState("Connecting...");

    useEffect(() => {
        if (!termRef.current) return;

        const term = new Terminal({
            cursorBlink: true,
            theme: { background: "#1e1e1e", foreground: "#d4d4d4" },
            fontFamily: 'Menlo, Monaco, "Courier New", monospace',
            fontSize: 18,
        });

        const fitAddon = new FitAddon();
        term.loadAddon(fitAddon);
        term.open(termRef.current);
        fitAddon.fit();
        termInstance.current = term;

        const handleResize = () => fitAddon.fit();
        window.addEventListener("resize", handleResize);

        const scheme = window.location.protocol === "https:" ? "wss" : "ws";
        const ws = new WebSocket(`${scheme}://${window.location.host}/portal/ws`);
        wsRef.current = ws;

        ws.onopen = () => {
            // Send Registration
            ws.send(JSON.stringify({
                portalId: portalId
            }));
            setStatus("Connected");
            term.focus();
        };

        ws.onmessage = async (e) => {
            let data = e.data;
            if (data instanceof Blob) {
                data = await data.text();
            }
            try {
                const resp = JSON.parse(data);
                const mote = resp.mote;
                if (mote && mote.bytes) {
                    if (mote.bytes.data) {
                        const binaryString = atob(mote.bytes.data);
                        term.write(binaryString);
                    }
                }
            } catch (err) {
                console.error("Failed to parse portal message", err);
            }
        };

        ws.onclose = () => {
            setStatus("Disconnected");
            term.write("\r\n\x1b[31m[Disconnected from Portal]\x1b[0m\r\n");
        };

        term.onData((data) => {
            if (ws.readyState === WebSocket.OPEN) {
                sequenceIdRef.current++;
                const req = {
                    mote: {
                        streamId: streamIdRef.current,
                        seqId: sequenceIdRef.current,
                        bytes: {
                            kind: "BYTES_PAYLOAD_KIND_PTY",
                            data: btoa(data)
                        }
                    }
                };
                ws.send(JSON.stringify(req));
            }
        });

        return () => {
            window.removeEventListener("resize", handleResize);
            ws.close();
            term.dispose();
        };
    }, [portalId]);

    return (
        <div className="flex flex-col h-full w-full">
            <div className="text-xs text-gray-400 mb-2">Portal ID: {portalId} - {status}</div>
            <div className="flex-1 min-h-0 relative">
                <div ref={termRef} className="absolute inset-0" />
            </div>
        </div>
    );
};

export default SshTerminal;
