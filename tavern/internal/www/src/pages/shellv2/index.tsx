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
             } else if (code === 9) { // Tab
                 // Check content for completion trigger
                 // If line is empty or we are at start, just indent.
                 if (currentLine.trim().length === 0) {
                     const indent = "    ";
                     currentLine += indent;
                     termInstance.current?.write(indent);
                 } else {
                     // Attempt completion
                     const completions = adapter.current?.complete(currentLine, currentLine.length);
                     if (completions && completions.length > 0) {
                         // Simple completion: print possibilities
                         // In a real shell, we might complete inline if unique, or show list.
                         // For now, let's print them on a new line and reprint prompt/buffer.

                         // If only one completion, append the suffix.
                         // But our completion engine returns full suggestions or suffixes?
                         // Interpreter::complete returns full strings usually.
                         // Actually, eldritch-core `complete` returns `(start_index, candidates)`.
                         // Our headless adapter returns `candidates` (array of strings).

                         // If we have candidates, let's see.
                         // To properly implement inline completion, we need to know the start index.
                         // But `HeadlessRepl.complete` in rust returns just the list.
                         // The user asked for "show potential autocompletions".

                         termInstance.current?.write("\r\n");
                         termInstance.current?.write(completions.join("  "));
                         termInstance.current?.write("\r\n");

                         // Re-print prompt and buffer
                         // We don't track prompt state perfectly here (could be ">>> " or ".. ")
                         // Assuming ">>> " for single line contexts where we usually complete.
                         // Ideally `adapter` should expose current prompt.
                         termInstance.current?.write(">>> " + currentLine);
                     } else {
                         // No completions, fallback to indent? Or do nothing?
                         // Usually tab does nothing if no completion.
                     }
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
