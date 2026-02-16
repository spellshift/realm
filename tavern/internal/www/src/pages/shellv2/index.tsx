import { useEffect, useRef, useState } from "react";
import { Terminal } from "@xterm/xterm";
import { FitAddon } from "xterm-addon-fit";
import "@xterm/xterm/css/xterm.css";
import { HeadlessWasmAdapter } from "../../lib/headless-adapter";

interface ShellState {
    inputBuffer: string;
    cursorPos: number;
    history: string[];
    historyIndex: number;
    prompt: string;
    isSearching: boolean;
    searchQuery: string;
    currentBlock: string;
}

const ShellV2 = () => {
    const termRef = useRef<HTMLDivElement>(null);
    const termInstance = useRef<Terminal | null>(null);
    const adapter = useRef<HeadlessWasmAdapter | null>(null);

    // Shell state
    const shellState = useRef<ShellState>({
        inputBuffer: "",
        cursorPos: 0,
        history: [],
        historyIndex: -1,
        prompt: ">>> ",
        isSearching: false,
        searchQuery: "",
        currentBlock: ""
    });

    // UI state
    const [completions, setCompletions] = useState<string[]>([]);
    const [completionStart, setCompletionStart] = useState(0); // Index where completion starts
    const [showCompletions, setShowCompletions] = useState(false);
    const [completionIndex, setCompletionIndex] = useState(0);
    const [completionPos, setCompletionPos] = useState({ x: 0, y: 0 });

    const lastBufferHeight = useRef(0);

    // We need a ref to access current completions inside onData without stale closure
    const completionsRef = useRef<{ list: string[], start: number, show: boolean, index: number }>({
        list: [], start: 0, show: false, index: 0
    });

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

        const fitAddon = new FitAddon();
        termInstance.current.loadAddon(fitAddon);
        termInstance.current.open(termRef.current);
        fitAddon.fit();

        const handleResize = () => {
            fitAddon.fit();
        };
        window.addEventListener("resize", handleResize);

        termInstance.current.write("Initializing Headless REPL...\r\n");

        const scheme = window.location.protocol === "https:" ? "wss" : "ws";
        const url = `${scheme}://${window.location.host}/shellv2/ws`;

        adapter.current = new HeadlessWasmAdapter(url, (content) => {
            const formatted = content.replace(/\n/g, "\r\n");
            termInstance.current?.write(formatted);
            termInstance.current?.write(shellState.current.prompt);
        }, () => {
            termInstance.current?.write("Connected to backend.\r\n>>> ");
        });

        adapter.current.init();

        const redrawLine = () => {
            const term = termInstance.current;
            if (!term) return;
            const state = shellState.current;

            if (state.isSearching) {
                term.write("\r\x1b[K"); // Clear line for search
                const prompt = `(reverse-i-search)'${state.searchQuery}': `;
                // Find match
                let match = "";
                if (state.searchQuery) {
                    // Simple search backwards
                    for (let i = state.history.length - 1; i >= 0; i--) {
                        if (state.history[i].includes(state.searchQuery)) {
                            match = state.history[i];
                            break;
                        }
                    }
                }
                term.write(prompt + match);
            } else {
                // Optimized redraw: Handle multi-line
                const fullContent = state.prompt + state.inputBuffer;
                const rows = fullContent.split('\n').length - 1;

                // Move up to start of previous rendering
                const prevRows = lastBufferHeight.current;
                if (prevRows > 0) {
                    term.write(`\x1b[${prevRows}A`);
                }

                // Clear everything below
                term.write("\r\x1b[J");

                // Write new content
                term.write(fullContent);

                // Update last height
                lastBufferHeight.current = rows;

                // Move cursor to correct position
                // Calculate cursor position in terms of rows/cols relative to start
                const prefix = fullContent.slice(0, state.prompt.length + state.cursorPos);
                const cursorRow = prefix.split('\n').length - 1;
                const cursorCol = prefix.split('\n').pop()?.length || 0;

                // Current position after write is at end of content
                const totalRows = rows;
                // We need to move UP from end to cursorRow
                const moveUp = totalRows - cursorRow;
                if (moveUp > 0) {
                    term.write(`\x1b[${moveUp}A`);
                }

                term.write("\r"); // Go to start of line
                if (cursorCol > 0) {
                    term.write(`\x1b[${cursorCol}C`);
                }
            }
        };

        const updateCompletionsUI = (list: string[], start: number, show: boolean, index: number) => {
            setCompletions(list);
            setCompletionStart(start);
            setShowCompletions(show);
            setCompletionIndex(index);
            completionsRef.current = { list, start, show, index };

            if (show && termInstance.current) {
                // Calculate position
                // This is tricky without access to DOM cursor.
                // Approximation: lines from bottom?
                // Or relative to cursor.
                // xterm.js has `buffer.active.cursorX/Y`.
                const cursorX = termInstance.current.buffer.active.cursorX;
                const cursorY = termInstance.current.buffer.active.cursorY;
                // Convert to pixels... requires knowing cell size.
                // We can use a fixed approximation or helper.
                // For now, let's just center it or put it at top left of cursor line?
                // We can get element bounding rect.
                // const charWidth = termInstance.current._core._renderService.dimensions.actualCellWidth; // private API
                // Let's just use fixed pixel per char estimate for now: 9px width, 17px height
                const charWidth = 9;
                const charHeight = 17;
                setCompletionPos({
                    x: cursorX * charWidth + 20, // + padding
                    y: cursorY * charHeight + 40 // + header/padding
                });
            }
        };

        const applyCompletion = (completion: string) => {
            const state = shellState.current;
            const start = completionsRef.current.start;
            // Replace from start to cursorPos with completion
            // Ensure start is valid
            if (start >= 0 && start <= state.cursorPos) {
                const prefix = state.inputBuffer.slice(0, start);
                const suffix = state.inputBuffer.slice(state.cursorPos);
                state.inputBuffer = prefix + completion + suffix;
                state.cursorPos = start + completion.length;
                redrawLine();
            }
            updateCompletionsUI([], 0, false, 0);
        };

        termInstance.current.onData((data) => {
             const code = data.charCodeAt(0);
             const state = shellState.current;
             const term = termInstance.current;
             if (!term) return;

             // If completions are showing, handle navigation
             if (completionsRef.current.show) {
                 if (code === 9) { // Tab: cycle
                     const list = completionsRef.current.list;
                     if (list.length > 0) {
                         const next = (completionsRef.current.index + 1) % list.length;
                         updateCompletionsUI(list, completionsRef.current.start, true, next);
                     }
                     return;
                 }
                 if (code === 13) { // Enter: select
                     applyCompletion(completionsRef.current.list[completionsRef.current.index]);
                     return;
                 }
                 if (data === "\x1b[B") { // Down
                     const list = completionsRef.current.list;
                     if (list.length > 0) {
                         const next = (completionsRef.current.index + 1) % list.length;
                         updateCompletionsUI(list, completionsRef.current.start, true, next);
                     }
                     return;
                 }
                 if (data === "\x1b[A") { // Up
                     const list = completionsRef.current.list;
                     if (list.length > 0) {
                         const next = (completionsRef.current.index - 1 + list.length) % list.length;
                         updateCompletionsUI(list, completionsRef.current.start, true, next);
                     }
                     return;
                 }
                 if (code === 27) { // Esc: cancel
                     updateCompletionsUI([], 0, false, 0);
                     return;
                 }
                 // Allow other keys to fall through to input handler
             }

             if (state.isSearching) {
                 if (data === "\x12") { // Ctrl+R (Next match)
                     // Logic to find next match (skipping current index?)
                     // For simplicity, just redraw which finds the first match from end.
                     // To implement "next match", we need to track search index.
                     // But user requirement was just "provide history searching".
                     // Basic reverse search is usually sufficient.
                     // If we want to find next, we need state.searchIndex.
                     // Let's keep it simple: Ctrl+R just redraws for now (noop effectively unless we track index).
                     return;
                 }
                 if (data === "\x03" || data === "\x07") { // Ctrl+C / Ctrl+G
                     state.isSearching = false;
                     state.searchQuery = "";
                     redrawLine();
                     return;
                 }
                 if (code === 13) { // Enter
                     // Use the match
                     state.isSearching = false;
                     let match = "";
                     if (state.searchQuery) {
                        for (let i = state.history.length - 1; i >= 0; i--) {
                            if (state.history[i].includes(state.searchQuery)) {
                                match = state.history[i];
                                break;
                            }
                        }
                     }
                     state.inputBuffer = match;
                     state.cursorPos = match.length;
                     state.searchQuery = "";
                     redrawLine();
                     return;
                 }
                 if (code === 127) { // Backspace
                     state.searchQuery = state.searchQuery.slice(0, -1);
                     redrawLine();
                     return;
                 }
                 if (code >= 32) {
                     state.searchQuery += data;
                     redrawLine();
                     return;
                 }
                 // Ignore other keys in search mode for now
                 return;
             }

             // Normal mode
             if (data === "\x03") { // Ctrl+C
                 adapter.current?.reset();
                 term.write("^C\r\n");
                 state.inputBuffer = "";
                 state.currentBlock = "";
                 state.cursorPos = 0;
                 state.historyIndex = -1;
                 state.prompt = ">>> ";
                 term.write(state.prompt);
                 lastBufferHeight.current = 0;
                 return;
             }

             if (data === "\x12") { // Ctrl+R
                 state.isSearching = true;
                 state.searchQuery = "";
                 redrawLine();
                 return;
             }

             if (data === "\x0c") { // Ctrl+L
                 term.write('\x1b[2J\x1b[H');
                 lastBufferHeight.current = 0;
                 redrawLine();
                 return;
             }

             if (data === "\x01") { // Ctrl+A
                 state.cursorPos = 0;
                 redrawLine();
                 return;
             }

             if (data === "\x05") { // Ctrl+E
                 state.cursorPos = state.inputBuffer.length;
                 redrawLine();
                 return;
             }

             if (data === "\x15") { // Ctrl+U
                 state.inputBuffer = "";
                 state.cursorPos = 0;
                 redrawLine();
                 return;
             }

             if (data === "\x17") { // Ctrl+W
                 const beforeCursor = state.inputBuffer.slice(0, state.cursorPos);
                 const trimmed = beforeCursor.trimEnd();
                 const lastSpace = trimmed.lastIndexOf(" ");
                 const newPos = lastSpace === -1 ? 0 : lastSpace + 1;
                 const afterCursor = state.inputBuffer.slice(state.cursorPos);
                 state.inputBuffer = state.inputBuffer.slice(0, newPos) + afterCursor;
                 state.cursorPos = newPos;
                 redrawLine();
                 return;
             }

             if (data === "\x1b[A") { // Up
                 if (state.history.length > 0) {
                     if (state.historyIndex === -1) state.historyIndex = state.history.length - 1;
                     else if (state.historyIndex > 0) state.historyIndex--;
                     state.inputBuffer = state.history[state.historyIndex];
                     state.cursorPos = state.inputBuffer.length;
                     redrawLine();
                 }
                 return;
             }

             if (data === "\x1b[B") { // Down
                 if (state.historyIndex !== -1) {
                     if (state.historyIndex < state.history.length - 1) {
                         state.historyIndex++;
                         state.inputBuffer = state.history[state.historyIndex];
                     } else {
                         state.historyIndex = -1;
                         state.inputBuffer = "";
                     }
                     state.cursorPos = state.inputBuffer.length;
                     redrawLine();
                 }
                 return;
             }

             if (data === "\x1b[D") { // Left
                 if (state.cursorPos > 0) {
                     state.cursorPos--;
                     term.write("\x1b[D");
                 }
                 return;
             }

             if (data === "\x1b[C") { // Right
                 if (state.cursorPos < state.inputBuffer.length) {
                     state.cursorPos++;
                     term.write("\x1b[C");
                 }
                 return;
             }

             if (code === 9) { // Tab
                 // Indent if line is empty or whitespace
                 if (!state.inputBuffer.trim()) {
                     const indent = "    ";
                     state.inputBuffer = state.inputBuffer.slice(0, state.cursorPos) + indent + state.inputBuffer.slice(state.cursorPos);
                     state.cursorPos += 4;
                     term.write(indent);
                     return;
                 }

                 // Trigger completion
                 const res = adapter.current?.complete(state.inputBuffer, state.cursorPos);
                 if (res && res.suggestions.length > 0) {
                     if (res.suggestions.length === 1) {
                         // Auto complete
                         // Use applyCompletion logic but locally
                         const completion = res.suggestions[0];
                         const start = res.start;
                         if (start >= 0 && start <= state.cursorPos) {
                             const prefix = state.inputBuffer.slice(0, start);
                             const suffix = state.inputBuffer.slice(state.cursorPos);
                             state.inputBuffer = prefix + completion + suffix;
                             state.cursorPos = start + completion.length;
                             redrawLine();
                         }
                     } else {
                         // Show dropdown
                         updateCompletionsUI(res.suggestions, res.start, true, 0);
                     }
                 }
                 return;
             }

             if (code >= 32 && code !== 127) {
                 if (state.cursorPos === state.inputBuffer.length) {
                     // Fast path: append at end
                     state.inputBuffer += data;
                     state.cursorPos += data.length;
                     term.write(data);
                 } else {
                     // Insert in middle
                     state.inputBuffer = state.inputBuffer.slice(0, state.cursorPos) + data + state.inputBuffer.slice(state.cursorPos);
                     state.cursorPos += data.length;
                     redrawLine();
                 }
             } else if (code === 13) { // Enter
                 term.write("\r\n");
                 const res = adapter.current?.input(state.inputBuffer);

                 state.currentBlock += state.inputBuffer + "\n";

                 if (res?.status === "complete") {
                     if (state.currentBlock.trim()) state.history.push(state.currentBlock.trimEnd());
                     state.currentBlock = "";
                     state.historyIndex = -1;
                     state.inputBuffer = "";
                     state.cursorPos = 0;
                     state.prompt = ">>> ";
                 } else if (res?.status === "incomplete") {
                     state.prompt = res.prompt || ".. ";
                     term.write(state.prompt);
                     state.inputBuffer = "";
                     state.cursorPos = 0;
                 } else {
                     term.write(`Error: ${res?.message}\r\n>>> `);
                     state.currentBlock = "";
                     state.inputBuffer = "";
                     state.cursorPos = 0;
                     state.prompt = ">>> ";
                 }
                 lastBufferHeight.current = 0;
             } else if (code === 127) { // Backspace
                 if (state.cursorPos > 0) {
                     if (state.cursorPos === state.inputBuffer.length) {
                         // Fast path: delete at end
                         state.inputBuffer = state.inputBuffer.slice(0, -1);
                         state.cursorPos--;
                         term.write("\b \b");
                     } else {
                         state.inputBuffer = state.inputBuffer.slice(0, state.cursorPos - 1) + state.inputBuffer.slice(state.cursorPos);
                         state.cursorPos--;
                         redrawLine();
                     }
                 }
             }

             // Trigger completion updates if needed
             if (completionsRef.current.show || code === 46 /* . */) {
                 const res = adapter.current?.complete(state.inputBuffer, state.cursorPos);
                 if (res && res.suggestions.length > 0) {
                     updateCompletionsUI(res.suggestions, res.start, true, 0);
                 } else {
                     if (completionsRef.current.show) {
                         updateCompletionsUI([], 0, false, 0);
                     }
                 }
             }
        });

        return () => {
            window.removeEventListener("resize", handleResize);
            adapter.current?.close();
            termInstance.current?.dispose();
        };
    }, []);

    return (
        <div style={{ padding: "20px", height: "calc(100vh - 100px)", position: "relative" }}>
            <h1 className="text-xl font-bold mb-4">Shell V2 (Headless REPL)</h1>
            <div ref={termRef} style={{ height: "100%", width: "100%" }} />

            {showCompletions && (
                <div style={{
                    position: "absolute",
                    top: completionPos.y,
                    left: completionPos.x,
                    background: "#252526",
                    border: "1px solid #454545",
                    zIndex: 1000,
                    maxHeight: "200px",
                    overflowY: "auto",
                    boxShadow: "0 4px 6px rgba(0,0,0,0.3)",
                    color: "#cccccc",
                    fontFamily: 'Menlo, Monaco, "Courier New", monospace',
                    fontSize: "14px"
                }}>
                    <ul style={{ listStyle: "none", margin: 0, padding: 0 }}>
                        {completions.map((c, i) => (
                            <li key={i} style={{
                                padding: "4px 8px",
                                background: i === completionIndex ? "#094771" : "transparent",
                                cursor: "pointer"
                            }}>
                                {c}
                            </li>
                        ))}
                    </ul>
                </div>
            )}
        </div>
    );
};

export default ShellV2;
