import { useEffect, useRef, useState } from "react";
import { Terminal } from "@xterm/xterm";
import { FitAddon } from "xterm-addon-fit";
import "@xterm/xterm/css/xterm.css";
import AlertError from "../../../components/tavern-base-ui/AlertError";

interface ShellSSHProps {
    target: string;
    portalId: number;
}

export const ShellSSH = ({ target, portalId }: ShellSSHProps) => {
    const termRef = useRef<HTMLDivElement>(null);
    const termInstance = useRef<Terminal | null>(null);
    const [connectionError, setConnectionError] = useState<string | null>(null);

    useEffect(() => {
        if (!termRef.current) return;

        // Initialize terminal
        termInstance.current = new Terminal({
            cursorBlink: true,
            macOptionIsMeta: true,
            theme: {
                background: "#1e1e1e",
                foreground: "#d4d4d4",
            },
            fontFamily: 'Menlo, Monaco, "Courier New", monospace',
            fontSize: 18,
        });

        const fitAddon = new FitAddon();
        termInstance.current.loadAddon(fitAddon);
        termInstance.current.open(termRef.current);
        fitAddon.fit();

        const handleResize = () => {
            fitAddon.fit();
        };
        window.addEventListener("resize", handleResize);

        termInstance.current.write(`Connecting to ${target} via Portal ${portalId}...\r\n`);

        const scheme = window.location.protocol === "https:" ? "wss" : "ws";
        const url = `${scheme}://${window.location.host}/portal/ssh?portal_id=${portalId}&target=${encodeURIComponent(target)}`;
        const socket = new WebSocket(url);

        socket.onmessage = (event) => {
            if (typeof event.data === "string") {
                termInstance.current?.write(event.data);
            } else if (event.data instanceof Blob) {
                const reader = new FileReader();
                reader.onload = () => {
                    if (reader.result) {
                        termInstance.current?.write(new Uint8Array(reader.result as ArrayBuffer));
                    }
                };
                reader.readAsArrayBuffer(event.data);
            }
        };

        socket.onerror = () => {
            setConnectionError("WebSocket connection error.");
        };

        socket.onclose = () => {
            termInstance.current?.write("\r\n\x1b[31mConnection closed.\x1b[0m\r\n");
        };

        termInstance.current.onData((data) => {
            if (socket.readyState === WebSocket.OPEN) {
                socket.send(data);
            }
        });

        return () => {
            window.removeEventListener("resize", handleResize);
            socket.close();
            termInstance.current?.dispose();
        };
    }, [target, portalId]);

    if (connectionError) {
        return (
            <div style={{ padding: "20px" }}>
                <AlertError label="SSH Connection Failed" details={connectionError} />
            </div>
        );
    }

    return (
        <div className="flex flex-col h-full p-5 bg-[#1e1e1e] text-[#d4d4d4]">
            <div className="flex-1 w-full relative">
                <div ref={termRef} className="absolute inset-0" />
            </div>
        </div>
    );
};
