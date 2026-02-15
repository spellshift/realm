import { useEffect, useRef } from "react";
import { Terminal } from "@xterm/xterm";
import "@xterm/xterm/css/xterm.css";
import { HeadlessWasmAdapter } from "../../lib/headless-adapter";

const ShellV2 = () => {
    const termRef = useRef<HTMLDivElement>(null);
    const termInstance = useRef<Terminal | null>(null);
    const adapter = useRef<HeadlessWasmAdapter | null>(null);

    useEffect(() => {
        if (!termRef.current) return;

        // Initialize terminal
        termInstance.current = new Terminal({
            cursorBlink: true,
            theme: {
                background: "#1e1e1e",
                foreground: "#d4d4d4",
            },
            fontFamily: 'Menlo, Monaco, "Courier New", monospace',
            fontSize: 14,
        });

        // Fit addon could be used here but keeping it simple
        termInstance.current.open(termRef.current);
        termInstance.current.write("Initializing Headless REPL...\r\n");

        // Initialize adapter
        const scheme = window.location.protocol === "https:" ? "wss" : "ws";
        const url = `${scheme}://${window.location.host}/shellv2/ws`;

        adapter.current = new HeadlessWasmAdapter(url, (content) => {
            // On output from backend
            const formatted = content.replace(/\n/g, "\r\n");
            termInstance.current?.write(formatted);
            // Re-print prompt after output
            termInstance.current?.write(">>> ");
        }, () => {
            termInstance.current?.write("Connected to backend.\r\n>>> ");
        });

        adapter.current.init();

        // Handle input
        let currentLine = "";

        termInstance.current.onData((data) => {
             const code = data.charCodeAt(0);
             if (code === 13) { // Enter
                 termInstance.current?.write("\r\n");
                 const res = adapter.current?.input(currentLine);
                 if (res?.status === "complete") {
                     currentLine = "";
                     // Backend will send output, which will trigger prompt
                 } else if (res?.status === "incomplete") {
                     termInstance.current?.write(res.prompt || ".. ");
                     currentLine = "";
                 } else if (res?.status === "error") {
                     termInstance.current?.write(`Error: ${res.message}\r\n>>> `);
                     currentLine = "";
                 }
             } else if (code === 127) { // Backspace
                 if (currentLine.length > 0) {
                     currentLine = currentLine.slice(0, -1);
                     termInstance.current?.write("\b \b");
                 }
             } else if (code >= 32) { // Printable
                 currentLine += data;
                 termInstance.current?.write(data);
             }
        });

        return () => {
            adapter.current?.close();
            termInstance.current?.dispose();
        };
    }, []);

    return (
        <div style={{ padding: "20px", height: "calc(100vh - 100px)" }}>
            <h1 className="text-xl font-bold mb-4">Shell V2 (Headless REPL)</h1>
            <div ref={termRef} style={{ height: "100%", width: "100%" }} />
        </div>
    );
};

export default ShellV2;
