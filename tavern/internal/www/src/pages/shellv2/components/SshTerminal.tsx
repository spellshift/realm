import { useEffect, useRef, useState } from "react";
import { Terminal } from "@xterm/xterm";
import { FitAddon } from "xterm-addon-fit";
import "@xterm/xterm/css/xterm.css";

interface SshTerminalProps {
    portalId: number;
    initialCommand?: string;
}

/** Parse "ssh user@host" or "ssh host", defaulting user to "root" */
function parseSshTarget(cmd?: string): { user: string; host: string } {
    if (!cmd) return { user: "root", host: "localhost" };
    const stripped = cmd.startsWith("ssh ") ? cmd.slice(4).trim() : cmd.trim();
    if (stripped.includes("@")) {
        const [user, host] = stripped.split("@", 2);
        return { user: user || "root", host: host || "localhost" };
    }
    return { user: "root", host: stripped || "localhost" };
}

const SshTerminal = ({ portalId, initialCommand }: SshTerminalProps) => {
    const termRef = useRef<HTMLDivElement>(null);
    const wsRef = useRef<WebSocket | null>(null);
    const wasmSshRef = useRef<any>(null);
    const streamIdRef = useRef<string>(crypto.randomUUID());
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

        try {
            fitAddon.fit();
        } catch (e) {
            console.warn("fitAddon.fit failed in SshTerminal", e);
        }

        const resizeObserver = new ResizeObserver(() => {
            if (termRef.current && termRef.current.clientWidth > 0) {
                try {
                    fitAddon.fit();
                    if (wasmSshRef.current) {
                        wasmSshRef.current.resize_pty(term.cols, term.rows);
                    }
                } catch (_) {
                    // ignore
                }
            }
        });
        resizeObserver.observe(termRef.current);

        const scheme = window.location.protocol === "https:" ? "wss" : "ws";
        const ws = new WebSocket(`${scheme}://${window.location.host}/portal/ws`);
        wsRef.current = ws;

        const { user, host } = parseSshTarget(initialCommand);

        ws.onopen = async () => {
            ws.send(JSON.stringify({ portalId }));
            setStatus(`Connecting SSH to ${user}@${host}…`);
            term.focus();

            try {
                // @ts-ignore
                const wasmModule = await import(/* webpackIgnore: true */ "/wasm/eldritch_wasm.js");
                await wasmModule.default("/wasm/eldritch_wasm_bg.wasm");

                /** Called by russh to write TCP bytes — proxy them to the Portal WebSocket as TCP motes */
                const onTcpSend = (data: Uint8Array) => {
                    console.log("[SshTerminal] onTcpSend called, bytes:", data.length, "ws state:", ws.readyState);
                    if (ws.readyState !== WebSocket.OPEN) {
                        console.warn("[SshTerminal] onTcpSend: WS not open, dropping", data.length, "bytes");
                        return;
                    }
                    sequenceIdRef.current++;
                    let binary = "";
                    for (let i = 0; i < data.length; i++) binary += String.fromCharCode(data[i]);
                    const msg = JSON.stringify({
                        mote: {
                            streamId: streamIdRef.current,
                            seqId: sequenceIdRef.current,
                            tcp: {
                                data: btoa(binary),
                                dstAddr: host,
                                dstPort: 22,
                            }
                        }
                    });
                    console.log("[SshTerminal] sending TCP mote:", msg.slice(0, 200));
                    ws.send(msg);
                };

                const onStdout = (data: Uint8Array) => { term.write(data); };
                const onStderr = (data: Uint8Array) => { term.write(data); };
                const onDisconnect = (message?: string) => {
                    const reason = message || "SSH session ended";
                    setStatus(`Disconnected: ${reason}`);
                    term.write(`\r\n\x1b[31m[${reason}]\x1b[0m\r\n`);
                };

                const wasmSsh = new wasmModule.WasmSsh(
                    user, onTcpSend, onStdout, onStderr, onDisconnect,
                    term.cols, term.rows,
                );
                wasmSshRef.current = wasmSsh;
                setStatus(`SSH: ${user}@${host}`);

                // Catch async WASM RuntimeError panics (they propagate as unhandled rejections)
                const panicHandler = (ev: PromiseRejectionEvent) => {
                    const msg = ev.reason?.message || String(ev.reason);
                    if (msg.includes("unreachable") || msg.includes("RuntimeError")) {
                        console.error("[SshTerminal] WASM panic:", msg);
                        term.write(`\r\n\x1b[31m[SSH internal error: ${msg}]\x1b[0m\r\n`);
                        setStatus("Error");
                        ev.preventDefault();
                    }
                };
                window.addEventListener("unhandledrejection", panicHandler);
                // Clean up the handler when the component unmounts (handled in cleanup below)
            } catch (e) {
                console.error("Failed to initialize WasmSsh", e);
                term.write(`\r\n\x1b[31m[Failed to initialize SSH WASM: ${e}]\x1b[0m\r\n`);
                setStatus("Error");
            }
        };

        ws.onmessage = async (e) => {
            let raw = e.data;
            if (raw instanceof Blob) raw = await raw.text();
            try {
                const resp = JSON.parse(raw);
                const mote = resp?.mote;
                console.log("[SshTerminal] ws.onmessage mote keys:", mote ? Object.keys(mote) : "no mote", "wasmReady:", !!wasmSshRef.current);
                if (mote?.tcp?.data && wasmSshRef.current) {
                    console.log("[SshTerminal] feeding TCP mote, b64 len:", mote.tcp.data.length);
                    // TCP response from portal — feed raw bytes into the SSH WASM parser
                    const binaryString = atob(mote.tcp.data);
                    const bytes = new Uint8Array(binaryString.length);
                    for (let i = 0; i < binaryString.length; i++) {
                        bytes[i] = binaryString.charCodeAt(i);
                    }
                    wasmSshRef.current.on_tcp_recv(bytes);
                } else if (mote && !mote.tcp) {
                    console.log("[SshTerminal] non-TCP mote received (bytes/shell?):", JSON.stringify(mote).slice(0, 200));
                }
            } catch (err) {
                console.error("Failed to parse portal message", err);
            }
        };

        ws.onclose = () => {
            setStatus("Disconnected");
            term.write("\r\n\x1b[31m[Disconnected from Portal]\x1b[0m\r\n");
        };

        // Forward xterm user input to SSH stdin
        term.onData((data) => {
            if (wasmSshRef.current) {
                wasmSshRef.current.on_stdin(new TextEncoder().encode(data));
            }
        });

        return () => {
            resizeObserver.disconnect();
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
