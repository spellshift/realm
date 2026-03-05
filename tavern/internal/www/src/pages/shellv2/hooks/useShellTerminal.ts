import { useEffect, useRef, useState, useCallback } from "react";
import { Terminal } from "@xterm/xterm";
import { FitAddon } from "xterm-addon-fit";
import "@xterm/xterm/css/xterm.css";
import { BrowserWasmAdapter, ConnectionStatus } from "../../../lib/browser-adapter";
import { WebsocketControlFlowSignal, WebsocketMessage, WebsocketMessageKind } from "../websocket";
import docsData from "../../../assets/eldritch-docs.json";
import { moveWordLeft, moveWordRight, highlightPythonSyntax, loadHistory, saveHistory, isInsideString } from "./shellUtils";

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
    const adapter = useRef<BrowserWasmAdapter | null>(null);
    const [connectionError, setConnectionError] = useState<string | null>(null);
    const [connectionStatus, setConnectionStatus] = useState<ConnectionStatus>("disconnected");
    const [connectionMessage, setConnectionMessage] = useState<string>("");

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
    const tooltipTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);

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
        if (completion === "no suggestions") {
            updateCompletionsUI([], 0, false, 0);
            return;
        }

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

    const scheduleHideTooltip = useCallback(() => {
        if (tooltipTimeoutRef.current) clearTimeout(tooltipTimeoutRef.current);
        tooltipTimeoutRef.current = setTimeout(() => {
            currentTooltipWord.current = null;
            setTooltipState(s => ({ ...s, visible: false }));
        }, 25);
    }, []);

    const cancelHideTooltip = useCallback(() => {
        if (tooltipTimeoutRef.current) clearTimeout(tooltipTimeoutRef.current);
        tooltipTimeoutRef.current = null;
    }, []);

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
            scheduleHideTooltip();
            return;
        }

        // Map visual row to buffer row
        const buffer = termInstance.current.buffer.active;
        const bufferRowIndex = buffer.viewportY + row;
        const line = buffer.getLine(bufferRowIndex);

        if (!line) {
            scheduleHideTooltip();
            return;
        }

        const lineStr = line.translateToString(true);

        // Extract word
        if (col >= lineStr.length) {
            scheduleHideTooltip();
            return;
        }

        const allowedChars = /[a-zA-Z0-9_\.]/;
        if (!allowedChars.test(lineStr[col])) {
            scheduleHideTooltip();
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
            cancelHideTooltip();
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
            scheduleHideTooltip();
        }
    }, [tooltipState.visible, scheduleHideTooltip, cancelHideTooltip]);

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

        try {
            fitAddon.fit();
        } catch (e) {
            console.warn("fitAddon.fit failed", e);
        }

        const resizeObserver = new ResizeObserver(() => {
            if (termRef.current && termRef.current.clientWidth > 0) {
                try {
                    fitAddon.fit();
                } catch (e) {
                    // Ignore if it still fails
                }
            }
        });
        resizeObserver.observe(termRef.current);

        termInstance.current.write("Eldritch v0.3.0\r\n");

        // Define redrawLine locally for use inside closures (adapter callback + key handler)
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
                    for (let i = state.history.length - 1; i >= 0; i--) {
                        if (state.history[i].includes(state.searchQuery)) {
                            match = state.history[i];
                            break;
                        }
                    }
                }
                contentToWrite = prompt + match;
                contentToDisplay = contentToWrite;
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
                    if (line.length > 0) {
                        count += Math.floor((line.length - 1) / cols);
                    }
                }
                return count;
            };

            const rows = getVisualLineCount(contentToWrite, termCols);

            // Move up to start of previous rendering
            const prevRows = lastBufferHeight.current;
            if (prevRows > 0) {
                term.write(`\x1b[${prevRows}A`);
            }

            // Clear everything below
            term.write("\r\x1b[J");

            // Write new content
            term.write(contentToDisplay.replace(/\n/g, "\r\n"));

            // Update last height
            lastBufferHeight.current = rows;

            // Move cursor to correct position
            if (!state.isSearching) {
                const prefix = contentToWrite.slice(0, cursorIndex);
                const cursorRow = getVisualLineCount(prefix, termCols);
                const lastLine = prefix.split('\n').pop() || "";
                const cursorCol = lastLine.length % termCols;

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

        const scheme = window.location.protocol === "https:" ? "wss" : "ws";
        const url = `${scheme}://${window.location.host}/shellv2/ws?shell_id=${shellId}`;

        adapter.current = new BrowserWasmAdapter(
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
            (status: ConnectionStatus, message?: string) => {
                setConnectionStatus(status);
                setConnectionMessage(message || "");
            }
        );

        adapter.current.init();

        // ─────────────────────────────────────────────────────────────────
        // Pure-TypeScript key handling via xterm's onData
        // ─────────────────────────────────────────────────────────────────
        const setupKeys = () => {
            // Helper: check whether an input line closes an open Python block
            const needsContinuation = (line: string): boolean => {
                const trimmed = line.trimEnd();
                return trimmed.endsWith(":") || trimmed.endsWith("\\") || trimmed.endsWith(",");
            };

            // Helper: detect SSH meta command pattern ssh(...) or ssh "host"
            const detectSshCommand = (input: string): string[] | null => {
                // Match: ssh(...) or ssh("...") or ssh 'host' or ssh user@host etc.
                const sshMatch = input.match(/^\s*ssh\s*\(\s*(.*)\s*\)\s*$/) ||
                    input.match(/^\s*ssh\s+(.+)\s*$/);
                if (sshMatch) {
                    const args = sshMatch[1].trim().replace(/^["']|["']$/g, "");
                    return [args];
                }
                return null;
            };

            // Helper: send line over WebSocket
            const sendLine = (line: string) => {
                const ws = (adapter.current as any)?.ws;
                const isOpen = (adapter.current as any)?.isWsOpen;
                const term = termInstance.current;
                if (isOpen && ws) {
                    ws.send(JSON.stringify({
                        kind: WebsocketMessageKind.Input,
                        input: line
                    }));
                } else {
                    term?.write("Error: WebSocket not connected\r\n");
                }
            };

            const handleData = (data: string) => {
                if (isLateCheckinRef.current) return;
                if (connectionStatusRef.current !== "connected") return;

                const state = shellState.current;
                const term = termInstance.current;
                if (!term) return;

                // ── ANSI escape sequences (arrow keys, etc.) ──────────────
                if (data.startsWith("\x1b[") || data.startsWith("\x1bO")) {
                    const seq = data.slice(2) || data.slice(2);
                    const code = data.startsWith("\x1b[") ? data.slice(2) : data.slice(2);

                    // Arrow keys
                    if (data === "\x1b[D" || data === "\x1bOD") {
                        // Left arrow
                        if (state.cursorPos > 0) {
                            state.cursorPos--;
                            redrawLine();
                        }
                        return;
                    }
                    if (data === "\x1b[C" || data === "\x1bOC") {
                        // Right arrow
                        if (state.cursorPos < state.inputBuffer.length) {
                            state.cursorPos++;
                            redrawLine();
                        }
                        return;
                    }
                    if (data === "\x1b[A" || data === "\x1bOA") {
                        // Up arrow — history previous
                        if (state.history.length === 0) return;
                        if (state.historyIndex === -1) {
                            state.historyIndex = state.history.length - 1;
                        } else if (state.historyIndex > 0) {
                            state.historyIndex--;
                        }
                        state.inputBuffer = state.history[state.historyIndex];
                        state.cursorPos = state.inputBuffer.length;
                        redrawLine();
                        return;
                    }
                    if (data === "\x1b[B" || data === "\x1bOB") {
                        // Down arrow — history next
                        if (state.historyIndex === -1) return;
                        if (state.historyIndex < state.history.length - 1) {
                            state.historyIndex++;
                            state.inputBuffer = state.history[state.historyIndex];
                        } else {
                            state.historyIndex = -1;
                            state.inputBuffer = "";
                        }
                        state.cursorPos = state.inputBuffer.length;
                        redrawLine();
                        return;
                    }

                    // Home / End
                    if (data === "\x1b[H" || data === "\x1b[1~") {
                        state.cursorPos = 0;
                        redrawLine();
                        return;
                    }
                    if (data === "\x1b[F" || data === "\x1b[4~") {
                        state.cursorPos = state.inputBuffer.length;
                        redrawLine();
                        return;
                    }

                    // Delete key
                    if (data === "\x1b[3~") {
                        if (state.cursorPos < state.inputBuffer.length) {
                            state.inputBuffer =
                                state.inputBuffer.slice(0, state.cursorPos) +
                                state.inputBuffer.slice(state.cursorPos + 1);
                            redrawLine();
                        }
                        return;
                    }

                    // Alt+Left (word jump left) — various terminals send different codes
                    if (data === "\x1b[1;3D" || data === "\x1bb" || data === "\x1b\x1b[D") {
                        state.cursorPos = moveWordLeft(state.inputBuffer, state.cursorPos);
                        redrawLine();
                        return;
                    }
                    // Alt+Right (word jump right)
                    if (data === "\x1b[1;3C" || data === "\x1bf" || data === "\x1b\x1b[C") {
                        state.cursorPos = moveWordRight(state.inputBuffer, state.cursorPos);
                        redrawLine();
                        return;
                    }

                    // Alt+Backspace — delete word backwards (some terminals)
                    if (data === "\x1b\x7f" || data === "\x1b\b") {
                        const newPos = moveWordLeft(state.inputBuffer, state.cursorPos);
                        state.inputBuffer =
                            state.inputBuffer.slice(0, newPos) +
                            state.inputBuffer.slice(state.cursorPos);
                        state.cursorPos = newPos;
                        redrawLine();
                        return;
                    }

                    // Ignore other escape sequences
                    return;
                }

                // ── Control characters ────────────────────────────────────
                if (data.length === 1) {
                    const code = data.charCodeAt(0);

                    // Ctrl+A — go to start of line
                    if (code === 1) {
                        state.cursorPos = 0;
                        redrawLine();
                        return;
                    }
                    // Ctrl+E — go to end of line
                    if (code === 5) {
                        state.cursorPos = state.inputBuffer.length;
                        redrawLine();
                        return;
                    }
                    // Ctrl+B — move one char left
                    if (code === 2) {
                        if (state.cursorPos > 0) {
                            state.cursorPos--;
                            redrawLine();
                        }
                        return;
                    }
                    // Ctrl+F — move one char right
                    if (code === 6) {
                        if (state.cursorPos < state.inputBuffer.length) {
                            state.cursorPos++;
                            redrawLine();
                        }
                        return;
                    }
                    // Ctrl+K — kill to end of line
                    if (code === 11) {
                        state.inputBuffer = state.inputBuffer.slice(0, state.cursorPos);
                        redrawLine();
                        return;
                    }
                    // Ctrl+U — kill to start of line
                    if (code === 21) {
                        state.inputBuffer = state.inputBuffer.slice(state.cursorPos);
                        state.cursorPos = 0;
                        redrawLine();
                        return;
                    }
                    // Ctrl+W — delete word backwards
                    if (code === 23) {
                        const newPos = moveWordLeft(state.inputBuffer, state.cursorPos);
                        state.inputBuffer =
                            state.inputBuffer.slice(0, newPos) +
                            state.inputBuffer.slice(state.cursorPos);
                        state.cursorPos = newPos;
                        redrawLine();
                        return;
                    }
                    // Ctrl+C — interrupt / clear line
                    if (code === 3) {
                        if (state.isSearching) {
                            state.isSearching = false;
                            state.searchQuery = "";
                        }
                        term.write("^C\r\n");
                        state.inputBuffer = "";
                        state.cursorPos = 0;
                        state.historyIndex = -1;
                        state.currentBlock = "";
                        state.prompt = ">>> ";
                        lastBufferHeight.current = 0;
                        redrawLine();
                        updateCompletionsUI([], 0, false, 0);
                        return;
                    }
                    // Ctrl+L — clear screen
                    if (code === 12) {
                        term.write('\x1b[2J\x1b[H');
                        lastBufferHeight.current = 0;
                        redrawLine();
                        return;
                    }
                    // Ctrl+R — toggle reverse search
                    if (code === 18) {
                        state.isSearching = !state.isSearching;
                        state.searchQuery = "";
                        redrawLine();
                        return;
                    }
                    // Ctrl+D — EOF / exit (only if buffer is empty)
                    if (code === 4) {
                        if (state.inputBuffer === "" && state.currentBlock === "") {
                            term.write("\r\nSession closed locally via exit.\r\n");
                        }
                        return;
                    }

                    // TAB — completion
                    if (code === 9) {
                        const cRef = completionsRef.current;
                        if (cRef.show && cRef.list.length > 0) {
                            // Cycle through completions
                            const nextIdx = (cRef.index + 1) % cRef.list.length;
                            updateCompletionsUI(cRef.list, cRef.start, true, nextIdx);
                            applyCompletion(cRef.list[nextIdx]);
                        } else {
                            // Request completions from WASM
                            const result = adapter.current?.complete(state.inputBuffer, state.cursorPos);
                            if (result && result.suggestions.length > 0) {
                                if (result.suggestions.length === 1) {
                                    applyCompletion(result.suggestions[0]);
                                } else {
                                    updateCompletionsUI(result.suggestions, result.start, true, 0);
                                    applyCompletion(result.suggestions[0]);
                                }
                            }
                        }
                        return;
                    }

                    // ENTER — submit line
                    if (code === 13) {
                        if (state.isSearching) {
                            // Accept search result
                            let match = "";
                            for (let i = state.history.length - 1; i >= 0; i--) {
                                if (state.history[i].includes(state.searchQuery)) {
                                    match = state.history[i];
                                    break;
                                }
                            }
                            state.isSearching = false;
                            state.searchQuery = "";
                            state.inputBuffer = match;
                            state.cursorPos = match.length;
                            redrawLine();
                            return;
                        }

                        const line = state.inputBuffer;
                        term.write("\r\n");
                        lastBufferHeight.current = 0;
                        state.historyIndex = -1;
                        updateCompletionsUI([], 0, false, 0);

                        // Blank line: if in block, submit; otherwise just re-prompt
                        if (line.trim() === "") {
                            if (state.currentBlock !== "") {
                                // Submit the accumulated block
                                const fullBlock = state.currentBlock;
                                state.currentBlock = "";
                                state.prompt = ">>> ";
                                state.inputBuffer = "";
                                state.cursorPos = 0;

                                if (fullBlock.trim()) {
                                    state.history.push(fullBlock.trim());
                                    saveHistory(state.history);
                                }

                                // Check for SSH meta command in single-line blocks
                                const sshArgs = detectSshCommand(fullBlock);
                                if (sshArgs) {
                                    window.dispatchEvent(new CustomEvent("ELD_META_COMMAND", {
                                        detail: {
                                            shellId,
                                            command: "ssh",
                                            args: sshArgs
                                        }
                                    }));
                                    redrawLine();
                                    return;
                                }

                                sendLine(fullBlock);
                                redrawLine();
                            } else {
                                state.inputBuffer = "";
                                state.cursorPos = 0;
                                redrawLine();
                            }
                            return;
                        }

                        // Check for SSH meta command (single line, no block)
                        if (state.currentBlock === "") {
                            const sshArgs = detectSshCommand(line);
                            if (sshArgs) {
                                state.inputBuffer = "";
                                state.cursorPos = 0;

                                window.dispatchEvent(new CustomEvent("ELD_META_COMMAND", {
                                    detail: {
                                        shellId,
                                        command: "ssh",
                                        args: sshArgs
                                    }
                                }));
                                redrawLine();
                                return;
                            }
                        }

                        // Multi-line block accumulation
                        if (needsContinuation(line) || state.currentBlock !== "") {
                            // Append current line to block
                            const indent = state.inputBuffer.match(/^(\s*)/)?.[1] ?? "";
                            const newIndent = needsContinuation(line) ? indent + "    " : indent;
                            if (state.currentBlock === "") {
                                state.currentBlock = line + "\n";
                            } else {
                                state.currentBlock += line + "\n";
                            }

                            // If the line ends a block (blank terminator handled above), we already submitted
                            state.prompt = "... ";
                            state.inputBuffer = newIndent;
                            state.cursorPos = newIndent.length;
                            redrawLine();
                            return;
                        }

                        // Plain single-line command — submit immediately
                        const trimmed = line.trim();
                        if (trimmed) {
                            state.history.push(trimmed);
                            saveHistory(state.history);
                        }

                        state.inputBuffer = "";
                        state.cursorPos = 0;

                        if (trimmed === "exit" || trimmed === "quit()") {
                            term.write("Session closed locally via exit.\r\n");
                            return;
                        }

                        sendLine(line);
                        redrawLine();
                        return;
                    }

                    // BACKSPACE (DEL char)
                    if (code === 127 || code === 8) {
                        if (state.isSearching) {
                            state.searchQuery = state.searchQuery.slice(0, -1);
                            redrawLine();
                            return;
                        }
                        if (state.cursorPos > 0) {
                            state.inputBuffer =
                                state.inputBuffer.slice(0, state.cursorPos - 1) +
                                state.inputBuffer.slice(state.cursorPos);
                            state.cursorPos--;
                            redrawLine();
                        }
                        return;
                    }

                    // Filter out remaining control characters
                    if (code < 32) return;
                }

                // ── Printable characters (including multi-byte paste) ──────
                if (state.isSearching) {
                    state.searchQuery += data;
                    redrawLine();
                    return;
                }

                // Insert data at cursor position
                state.inputBuffer =
                    state.inputBuffer.slice(0, state.cursorPos) +
                    data +
                    state.inputBuffer.slice(state.cursorPos);
                state.cursorPos += data.length;

                // Clear completions on any text input
                updateCompletionsUI([], 0, false, 0);

                redrawLine();
            };

            const dataDispose = termInstance.current!.onData(handleData);
            return () => dataDispose.dispose();
        };

        const disposeKeyHandler = setupKeys();

        return () => {
            resizeObserver.disconnect();
            adapter.current?.close();
            disposeKeyHandler();
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
        connectionStatus,
        connectionMessage,
        handleTooltipMouseEnter: cancelHideTooltip,
        handleTooltipMouseLeave: scheduleHideTooltip
    };
};
