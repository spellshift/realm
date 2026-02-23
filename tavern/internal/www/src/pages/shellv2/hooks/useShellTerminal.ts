import { useEffect, useRef, useState } from "react";
import { Terminal } from "@xterm/xterm";
import { FitAddon } from "xterm-addon-fit";
import { HeadlessWasmAdapter } from "../../../lib/headless-adapter";
import { WebsocketControlFlowSignal, WebsocketMessage, WebsocketMessageKind } from "../websocket";
import { ShellState } from "../types";

export const useShellTerminal = (
    shellId: string | undefined,
    setPortalId: (id: number | null) => void,
    loading: boolean,
    error: any,
    data: any
) => {
    const termRef = useRef<HTMLDivElement>(null);
    const termInstance = useRef<Terminal | null>(null);
    const adapter = useRef<HeadlessWasmAdapter | null>(null);
    const [connectionError, setConnectionError] = useState<string | null>(null);

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

    // UI state for completions
    const [completions, setCompletions] = useState<string[]>([]);
    const [showCompletions, setShowCompletions] = useState(false);
    const [completionIndex, setCompletionIndex] = useState(0);
    const [completionPos, setCompletionPos] = useState({ x: 0, y: 0 });
    const completionsListRef = useRef<HTMLUListElement>(null);

    const lastBufferHeight = useRef(0);

    // We need a ref to access current completions inside onData without stale closure
    const completionsRef = useRef<{ list: string[], start: number, show: boolean, index: number }>({
        list: [], start: 0, show: false, index: 0
    });

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

        if (!data?.node) {
            setConnectionError("Shell not found.");
            return;
        }

        if (data.node.closedAt) {
            setConnectionError("This shell session is closed.");
            return;
        }

        // Initialize terminal
        termInstance.current = new Terminal({
            cursorBlink: true,
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

        termInstance.current.write("Eldritch v0.3.0\r\n");

        // Define redrawLine early so it can be used by adapter callback
        const redrawLine = () => {
            const term = termInstance.current;
            if (!term) return;
            const state = shellState.current;

            let contentToWrite = "";
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
                // In search mode, cursor is typically at the end of the match
                cursorIndex = contentToWrite.length;
            } else {
                contentToWrite = state.prompt + state.inputBuffer;
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
            term.write(contentToWrite.replace(/\n/g, "\r\n"));

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
        };

        const scheme = window.location.protocol === "https:" ? "wss" : "ws";
        const url = `${scheme}://${window.location.host}/shellv2/ws?shell_id=${shellId}`;

        adapter.current = new HeadlessWasmAdapter(url, (msg: WebsocketMessage) => {
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
                    color = "\x1b[31m"; // Red
                    break;
                case WebsocketMessageKind.Error:
                    content = msg.error;
                    color = "\x1b[31m"; // Red
                    break;
                case WebsocketMessageKind.ControlFlow:
                    if (msg.signal === WebsocketControlFlowSignal.TaskQueued && msg.message) {
                        content = msg.message + "\n";
                        color = "\x1b[33m"; // Yellow
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
        }, () => {
            termInstance.current?.write("Connected to Tavern.\r\n>>> ");
        });

        adapter.current.init();

        const updateCompletionsUI = (list: string[], start: number, show: boolean, index: number) => {
            setCompletions(list);
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
    }, [shellId, loading, error, data, setPortalId]);

    // Scroll active completion into view
    useEffect(() => {
        if (showCompletions && completionsListRef.current) {
            const activeElement = completionsListRef.current.children[completionIndex] as HTMLElement;
            if (activeElement) {
                activeElement.scrollIntoView({ block: "nearest" });
            }
        }
    }, [completionIndex, showCompletions]);

    return {
        termRef,
        completions,
        showCompletions,
        completionPos,
        completionIndex,
        connectionError,
        completionsListRef,
    };
};
