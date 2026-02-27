import { useEffect, useRef, useState, useCallback } from "react";
import { Terminal } from "@xterm/xterm";
import { FitAddon } from "xterm-addon-fit";
import "@xterm/xterm/css/xterm.css";
import { HeadlessWasmAdapter, ConnectionStatus } from "../../../lib/headless-adapter";
import { WebsocketControlFlowSignal, WebsocketMessage, WebsocketMessageKind } from "../websocket";
import docsData from "../../../assets/eldritch-docs.json";
import { moveWordLeft, moveWordRight, highlightPythonSyntax, loadHistory, saveHistory } from "./shellUtils";

const docs = docsData as Record<string, { signature: string; description: string }>;

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

export const useShellTerminal = (
    shellId: string | undefined,
    loading: boolean,
    error: any,
    shellData: any,
    setPortalId: (id: number | null) => void,
    isLateCheckin: boolean
) => {
    const termRef = useRef<HTMLDivElement>(null);
    const termInstance = useRef<Terminal | null>(null);
    const adapter = useRef<HeadlessWasmAdapter | null>(null);
    const [connectionError, setConnectionError] = useState<string | null>(null);
    const [connectionStatus, setConnectionStatus] = useState<ConnectionStatus>("disconnected");

    // Shell state
    const shellState = useRef<ShellState>({
        inputBuffer: "",
        cursorPos: 0,
        history: loadHistory(),
        historyIndex: -1,
        prompt: ">>> ",
        isSearching: false,
        searchQuery: "",
        currentBlock: ""
    });

    // UI state for completions
    const [completions, setCompletions] = useState<string[]>([]);
    const [completionStart, setCompletionStart] = useState(0); // Index where completion starts
    const [showCompletions, setShowCompletions] = useState(false);
    const [completionIndex, setCompletionIndex] = useState(0);
    const [completionPos, setCompletionPos] = useState({ x: 0, y: 0 });

    // Tooltip state
    const [tooltipState, setTooltipState] = useState<{
        visible: boolean;
        x: number;
        y: number;
        signature: string;
        description: string;
    }>({ visible: false, x: 0, y: 0, signature: "", description: "" });

    const currentTooltipWord = useRef<string | null>(null);

    const lastBufferHeight = useRef(0);

    // We need a ref to access current completions inside onData without stale closure
    const completionsRef = useRef<{ list: string[], start: number, show: boolean, index: number }>({
        list: [], start: 0, show: false, index: 0
    });

    const redrawTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);

    // Ref for late checkin to access in event handlers
    const isLateCheckinRef = useRef(isLateCheckin);
    const connectionStatusRef = useRef(connectionStatus);

    useEffect(() => {
        isLateCheckinRef.current = isLateCheckin;
        connectionStatusRef.current = connectionStatus;
        if (termInstance.current) {
            const isDimmed = isLateCheckin || connectionStatus !== "connected";
            termInstance.current.options.theme = {
                foreground: isDimmed ? "#777777" : "#d4d4d4",
                background: "#1e1e1e",
            };
        }
    }, [isLateCheckin, connectionStatus]);

    const redrawLine = useCallback(() => {
        const term = termInstance.current;
        if (!term) return;
        const state = shellState.current;

        let contentToWrite = "";
        let contentToDisplay = "";
        let cursorIndex = 0;

        if (state.isSearching) {
            const prompt = `(reverse-i-search)'${state.searchQuery}': `;
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
            contentToWrite = prompt + match;
            contentToDisplay = contentToWrite;
            // In search mode, cursor is typically at the end of the match
            cursorIndex = contentToWrite.length;
        } else {
            contentToWrite = state.prompt + state.inputBuffer;
            contentToDisplay = state.prompt + highlightPythonSyntax(state.inputBuffer);
            cursorIndex = state.prompt.length + state.cursorPos;
        }

        // Calculate rows based on newlines
        const rows = contentToWrite.split('\n').length - 1;

        // Move up to start of previous rendering (regardless of mode)
        const prevRows = lastBufferHeight.current;
        if (prevRows > 0) {
            term.write(`\x1b[${prevRows}A`);
        }

        // Clear everything below
        term.write("\r\x1b[J");

        // Write new content, ensuring newlines are carriage-return + newline
        term.write(contentToDisplay.replace(/\n/g, "\r\n"));

        // Update last height
        lastBufferHeight.current = rows;

        // Move cursor to correct position
        if (!state.isSearching) {
            // Calculate cursor position in terms of rows/cols relative to start
            const prefix = contentToWrite.slice(0, cursorIndex);
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
    }, []);

    const updateCompletionsUI = useCallback((list: string[], start: number, show: boolean, index: number) => {
        setCompletions(list);
        setCompletionStart(start);
        setShowCompletions(show);
        setCompletionIndex(index);
        completionsRef.current = { list, start, show, index };

        if (show && termInstance.current) {
            // Calculate position
            const cursorX = termInstance.current.buffer.active.cursorX;
            const cursorY = termInstance.current.buffer.active.cursorY;
            const charWidth = 9;
            const charHeight = 17;
            setCompletionPos({
                x: cursorX * charWidth + 20, // + padding
                y: cursorY * charHeight + 40 // + header/padding
            });
        }
    }, []);

    const applyCompletion = useCallback((completion: string) => {
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
    }, [redrawLine, updateCompletionsUI]);

    const handleMouseMove = useCallback((e: React.MouseEvent<HTMLDivElement>) => {
        if (!termInstance.current || !termRef.current) return;

        // Simple debounce could be added here if performance is an issue

        const rect = termRef.current.getBoundingClientRect();
        const cols = termInstance.current.cols;
        const rows = termInstance.current.rows;

        // Approximate cell dimensions
        // Note: This assumes the terminal fills the container.
        // xterm-addon-fit should ensure this mostly, but padding might affect it.
        // A better way is to use the internal metrics or measure a character.
        // For now, simple division:
        const cellWidth = rect.width / cols;
        const cellHeight = rect.height / rows;

        const x = e.clientX - rect.left;
        const y = e.clientY - rect.top;

        const col = Math.floor(x / cellWidth);
        const row = Math.floor(y / cellHeight);

        if (col < 0 || col >= cols || row < 0 || row >= rows) {
            setTooltipState(s => ({ ...s, visible: false }));
            return;
        }

        // Map visual row to buffer row
        const buffer = termInstance.current.buffer.active;
        const bufferRowIndex = buffer.viewportY + row;
        const line = buffer.getLine(bufferRowIndex);

        if (!line) {
            setTooltipState(s => ({ ...s, visible: false }));
            return;
        }

        const lineStr = line.translateToString(true);

        // Extract word
        if (col >= lineStr.length) {
            setTooltipState(s => ({ ...s, visible: false }));
            return;
        }

        const allowedChars = /[a-zA-Z0-9_\.]/;
        if (!allowedChars.test(lineStr[col])) {
            setTooltipState(s => ({ ...s, visible: false }));
            return;
        }

        let start = col;
        while (start > 0 && allowedChars.test(lineStr[start - 1])) {
            start--;
        }

        let end = col;
        while (end < lineStr.length && allowedChars.test(lineStr[end])) {
            end++;
        }

        const word = lineStr.slice(start, end);

        // Look up in docs
        if (docs[word]) {
            // If already showing tooltip for this word, don't update position
            if (currentTooltipWord.current === word && tooltipState.visible) {
                return;
            }

            currentTooltipWord.current = word;
            setTooltipState({
                visible: true,
                x: e.clientX, // Screen coordinates for fixed positioning
                y: e.clientY,
                signature: docs[word].signature,
                description: docs[word].description
            });
        } else {
             currentTooltipWord.current = null;
             setTooltipState(s => {
                 if (!s.visible) return s;
                 return { ...s, visible: false };
             });
        }
    }, [tooltipState.visible]);

    const shellNodeId = shellData?.node?.id;
    const shellClosedAt = shellData?.node?.closedAt;

    useEffect(() => {
        if (!termRef.current || loading) return;

        if (!shellId) {
            setConnectionError("No Shell ID provided in URL.");
            return;
        }

        if (error) {
            setConnectionError(`Failed to load shell: ${error.message}`);
            return;
        }

        if (!shellNodeId) {
            setConnectionError("Shell not found.");
            return;
        }

        if (shellClosedAt) {
            setConnectionError("This shell session is closed.");
            return;
        }

        // Initialize terminal
        termInstance.current = new Terminal({
            cursorBlink: true,
            macOptionIsMeta: true,
            theme: {
                background: "#1e1e1e",
                foreground: (isLateCheckinRef.current || connectionStatusRef.current !== "connected") ? "#777777" : "#d4d4d4",
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

        termInstance.current.write("Eldritch v0.3.0\r\n");

        // Define redrawLine early so it can be used by adapter callback
        const redrawLine = () => {
            const term = termInstance.current;
            if (!term) return;
            const state = shellState.current;

            let contentToWrite = "";
            let contentToDisplay = "";
            let cursorIndex = 0;

            if (state.isSearching) {
                const prompt = `(reverse-i-search)'${state.searchQuery}': `;
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
                contentToWrite = prompt + match;
                contentToDisplay = contentToWrite;
                // In search mode, cursor is typically at the end of the match
                cursorIndex = contentToWrite.length;
            } else {
                contentToWrite = state.prompt + state.inputBuffer;
                contentToDisplay = state.prompt + highlightPythonSyntax(state.inputBuffer);
                cursorIndex = state.prompt.length + state.cursorPos;
            }

            // Calculate rows based on visual line wrapping
            const termCols = term.cols;
            const getVisualLineCount = (text: string, cols: number) => {
                const lines = text.split('\n');
                let count = 0;
                for (let i = 0; i < lines.length; i++) {
                    const line = lines[i];
                    if (i > 0) count++; // Newline character
                    // Calculate wrapped lines for this segment
                    // Even an empty line takes 1 row if explicitly split
                    // But here, split('\n') gives empty string for consecutive newlines

                    if (line.length > 0) {
                        count += Math.floor((line.length - 1) / cols);
                    }
                    // If line is exactly cols length, it doesn't wrap to next line unless another char comes
                    // But we are counting *visual* rows.
                    // xterm wraps: if I write 80 chars on 80 col terminal, cursor is at (80, y)
                    // if I write 81 chars, cursor is at (1, y+1)
                }
                return count;
            };

            const rows = getVisualLineCount(contentToWrite, termCols);

            // Move up to start of previous rendering (regardless of mode)
            const prevRows = lastBufferHeight.current;
            if (prevRows > 0) {
                term.write(`\x1b[${prevRows}A`);
            }

            // Clear everything below
            term.write("\r\x1b[J");

            // Write new content, ensuring newlines are carriage-return + newline
            term.write(contentToDisplay.replace(/\n/g, "\r\n"));

            // Update last height
            lastBufferHeight.current = rows;

            // Move cursor to correct position
            if (!state.isSearching) {
                // Calculate cursor position in terms of rows/cols relative to start
                const prefix = contentToWrite.slice(0, cursorIndex);
                // Calculate rows occupied by prefix
                const cursorRow = getVisualLineCount(prefix, termCols);

                // Calculate cursor column
                const lastLine = prefix.split('\n').pop() || "";
                let cursorCol = lastLine.length % termCols;
                // If we are exactly at end of line (and not empty), it might be tricky
                // But xterm handles cursor positioning
                // If length is multiple of cols, cursor is effectively at index 0 of next line physically?
                // Actually, if we write 80 chars, cursor is at 80. Writing next char moves it.
                // We use relative movement.

                // We moved up `prevRows`. We wrote `rows` lines.
                // We are now at the end of the content.
                // We want to be at `cursorRow`.

                const moveUp = rows - cursorRow;
                if (moveUp > 0) {
                    term.write(`\x1b[${moveUp}A`);
                }

                term.write("\r"); // Go to start of line
                if (cursorCol > 0) {
                    term.write(`\x1b[${cursorCol}C`);
                }
            }
        };

        const scheduleRedraw = () => {
            if (redrawTimeoutRef.current) {
                clearTimeout(redrawTimeoutRef.current);
            }
            redrawTimeoutRef.current = setTimeout(() => {
                redrawLine();
            }, 50);
        };

        const scheme = window.location.protocol === "https:" ? "wss" : "ws";
        const url = `${scheme}://${window.location.host}/shellv2/ws?shell_id=${shellId}`;

        adapter.current = new HeadlessWasmAdapter(
            url,
            (msg: WebsocketMessage) => {
                const term = termInstance.current;
                if (!term) return;

                // Clear current input line(s) before printing output
                const prevRows = lastBufferHeight.current;
                if (prevRows > 0) {
                    term.write(`\x1b[${prevRows}A`);
                }
                term.write("\r\x1b[J");

                // Process message content
                let content = "";
                let color = "";

                switch (msg.kind) {
                    case WebsocketMessageKind.Output:
                        content = msg.output;
                        break;
                    case WebsocketMessageKind.TaskError:
                        content = msg.error;
                        color = "\x1b[38;2;255;0;0m"; // Red
                        break;
                    case WebsocketMessageKind.Error:
                        content = msg.error;
                        color = "\x1b[38;2;255;0;0m"; // Red
                        break;
                    case WebsocketMessageKind.ControlFlow:
                        if (msg.signal === WebsocketControlFlowSignal.TaskQueued && msg.message) {
                            content = msg.message + "\n";
                            color = "\x1b[38;5;178m"; // Purple
                        } else if (msg.signal === WebsocketControlFlowSignal.PortalUpgrade && msg.portal_id) {
                            setPortalId(msg.portal_id);
                        }
                        // Handle other control signals if needed
                        break;
                    case WebsocketMessageKind.OutputFromOtherStream:
                        content = msg.output;
                        break;
                }

                if (content) {
                    const formatted = content.replace(/\n/g, "\r\n");
                    if (color) {
                        term.write(color + formatted + "\x1b[0m");
                    } else {
                        term.write(formatted);
                    }

                    // Ensure there is a newline after output if not present, so prompt is on new line
                    if (!content.endsWith('\n')) {
                        term.write("\r\n");
                    }
                }

                // Reset input line state and redraw it at the bottom
                lastBufferHeight.current = 0;
                redrawLine();
            },
            () => {
                termInstance.current?.write("Connected to Tavern.\r\n>>> ");
            },
            (status: ConnectionStatus) => {
                setConnectionStatus(status);
            }
        );

        adapter.current.init();

        termInstance.current.onData((data) => {
            // Check for late checkin and block input
            if (isLateCheckinRef.current) return;
            // Check for connection status and block input
            if (connectionStatusRef.current !== "connected") return;

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

            if (data === "\x01" || data === "\x1b[H" || data === "\x1bOH") { // Ctrl+A / Home
                state.cursorPos = 0;
                redrawLine();
                return;
            }

            if (data === "\x05" || data === "\x1b[F" || data === "\x1bOF") { // Ctrl+E / End
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

            if (data === "\x0b") { // Ctrl+K (Kill to end)
                state.inputBuffer = state.inputBuffer.slice(0, state.cursorPos);
                redrawLine();
                return;
            }

            // Word Navigation: Left (Alt+Left, Alt+b)
            if (data === "\x1b[1;3D" || data === "\x1b[1;5D" || data === "\x1b\x1b[D" || data === "\x1bb") {
                state.cursorPos = moveWordLeft(state.inputBuffer, state.cursorPos);
                redrawLine();
                return;
            }

            // Word Navigation: Right (Alt+Right, Alt+f)
            if (data === "\x1b[1;3C" || data === "\x1b[1;5C" || data === "\x1b\x1b[C" || data === "\x1bf") {
                state.cursorPos = moveWordRight(state.inputBuffer, state.cursorPos);
                redrawLine();
                return;
            }

            // Word Deletion: Backward (Alt+Backspace / Ctrl+W)
            if (data === "\x17" || data === "\x1b\x7f" || data === "\x1b\x08") {
                const newPos = moveWordLeft(state.inputBuffer, state.cursorPos);
                const afterCursor = state.inputBuffer.slice(state.cursorPos);
                state.inputBuffer = state.inputBuffer.slice(0, newPos) + afterCursor;
                state.cursorPos = newPos;
                redrawLine();
                return;
            }

            // Word Deletion: Forward (Alt+Delete / Alt+d)
            if (data === "\x1b[3;3~" || data === "\x1bd") {
                const endPos = moveWordRight(state.inputBuffer, state.cursorPos);
                const beforeCursor = state.inputBuffer.slice(0, state.cursorPos);
                const afterWord = state.inputBuffer.slice(endPos);
                state.inputBuffer = beforeCursor + afterWord;
                // Cursor stays at current pos
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
                // Fast path for simple appending at the end of the line
                if (data.length === 1 && state.cursorPos === state.inputBuffer.length) {
                    state.inputBuffer += data;
                    state.cursorPos++;
                    term.write(data);
                    scheduleRedraw();
                } else {
                    state.inputBuffer = state.inputBuffer.slice(0, state.cursorPos) + data + state.inputBuffer.slice(state.cursorPos);
                    state.cursorPos += data.length;
                    redrawLine();
                }
            } else if (code === 13) { // Enter
                term.write("\r\n");
                const res = adapter.current?.input(state.inputBuffer);

                state.currentBlock += state.inputBuffer + "\n";

                if (res?.status === "complete") {
                    if (state.currentBlock.trim()) {
                        state.history.push(state.currentBlock.trimEnd());
                        saveHistory(state.history);
                    }
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
                    // Fast path for backspace at the end of the line
                    if (state.cursorPos === state.inputBuffer.length) {
                        state.inputBuffer = state.inputBuffer.slice(0, -1);
                        state.cursorPos--;
                        term.write('\b \b');
                        scheduleRedraw();
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
            if (redrawTimeoutRef.current) clearTimeout(redrawTimeoutRef.current);
        };
    }, [shellId, loading, error, shellNodeId, shellClosedAt, setPortalId, redrawLine, updateCompletionsUI, applyCompletion]);

    return {
        termRef,
        connectionError,
        completions,
        showCompletions,
        completionPos,
        completionIndex,
        handleMouseMove,
        tooltipState,
        handleCompletionSelect: applyCompletion,
        connectionStatus
    };
};
