import React, { useEffect, useMemo, useRef, useState, useCallback } from "react";
import { Terminal } from "@xterm/xterm";
import { FitAddon } from "xterm-addon-fit";
import "@xterm/xterm/css/xterm.css";
import {
    BrowserWasmAdapter,
    ConnectionStatus,
    WEBSOCKET_CONNECT_TIMEOUT_MS,
    normalizeMultilineScript,
} from "../../../lib/browser-adapter";
import { WebsocketControlFlowSignal, WebsocketMessage, WebsocketMessageKind } from "../websocket";
import docsData from "../../../assets/eldritch-docs.json";
import {
    moveWordLeft,
    moveWordRight,
    highlightPythonSyntax,
    loadHistory,
    saveHistory,
    stripInternalMacro,
    getViewportNonEmptyLineCount,
    buildScrollbackPreservingClearSequence,
    transformClearAnsiForScrollback,
    isInsideString,
    getCursorLineInfo,
    moveCursorUpInBuffer,
    moveCursorDownInBuffer,
    moveCursorToLineStart,
    moveCursorToLineEnd,
    shouldContinueMultilineOnEnter,
    insertNewlineWithAutoIndent,
    formatMultilineTerminalBuffer,
} from "./shellUtils";
import { ShellTaskItem } from "./useShellData";

const docs = docsData as Record<string, { signature: string; description: string }>;

const wrapText = (text: string, cols: number) => {
    const inputLines = text.split('\n');
    let lines = [];
    let inMarkdownBlock = false;

    for (const line of inputLines) {
        if (line.trim().startsWith('```')) {
            inMarkdownBlock = !inMarkdownBlock;
            lines.push(line.trim());
            continue;
        }

        if (inMarkdownBlock) {
            lines.push('  ' + line.trimStart());
            continue;
        }

        const words = line.split(' ');
        let currentLine = '';
        for (const word of words) {
            if (currentLine.length + word.length + 1 > cols) {
                if (currentLine.length > 0) lines.push(currentLine);
                currentLine = word;
            } else {
                if (currentLine.length > 0) currentLine += ' ';
                currentLine += word;
            }
        }
        if (currentLine) lines.push(currentLine);
    }
    return lines.join('\r\n');
};

const writeHelpMessage = (term: Terminal, version: string) => {
    const versionStr = version ? ` ${version}` : "";
    term.write(`Welcome to Eldritch${versionStr}!\r\n`);
    term.write("Run \x1b[1m`libs()`\x1b[0m and \x1b[1m`builtins()`\x1b[0m to list available functionality.\r\n");
    term.write("Use \x1b[1m`dir(module)`\x1b[0m to enumerate a module's properties and methods (e.g. \x1b[1m`dir(file)`\x1b[0m).\r\n");
    term.write("\r\n");
    term.write("\x1b[1;33m# Macros\x1b[0m\r\n");
    term.write("  \x1b[1m`!command`\x1b[0m — Shorthand for \x1b[1m`sys.shell(command)`\x1b[0m.\r\n");
    term.write("\r\n");
    term.write("\x1b[1;33m# Meta Commands\x1b[0m\r\n");
    term.write("These commands run locally in your browser and are not sent to the agent.\r\n");
    term.write("  \x1b[1m`help`\x1b[0m                        — Show this message. Use \x1b[1m`help(module)`\x1b[0m for module docs (e.g. \x1b[1m`help(sys)`\x1b[0m).\r\n                                    Hover over any symbol for inline documentation.\r\n");
    term.write("  \x1b[1m`ssh(\"user:pass@host:port\")`\x1b[0m — Open an SSH session through the agent's portal connection.\r\n");
    term.write("  \x1b[1m`pty()`\x1b[0m                       — Open an interactive PTY session through the agent's portal connection.\r\n");
    term.write("\r\n");
    term.write("\x1b[1;33m# Example Usage\x1b[0m\r\n");
    term.write("\r\n");
    term.write("\x1b[1;33m## File\x1b[0m\r\n");
    term.write("  \x1b[1m`file.list(\"/tmp/\")`\x1b[0m                                         — List files in tmp\r\n");
    term.write("  \x1b[1m`file.move(\"/etc/passwd\", \"/etc/passwd.bak\")`\x1b[0m                — Move a file\r\n");
    term.write("  \x1b[1m`report.file(\"C:\\\\windows\\\\system32\\\\cmd.exe\")`\x1b[0m               — Send a file from a host to the c2\r\n");
    term.write("  \x1b[1m`assets.copy(\"file.exe\", \"C:\\\\Windows\\\\Temp\\\\small.exe\")`\x1b[0m    — Copy a file from tavern c2's assets to host\r\n");
    term.write("  \x1b[1m`file.read(\"/home/*/.bash_history\")`\x1b[0m                         — Read file contents\r\n");
    term.write("\r\n");
    term.write("\x1b[1;33m## Process\x1b[0m\r\n");
    term.write("  \x1b[1m`process.list()`\x1b[0m                                             — List processes\r\n");
    term.write("  \x1b[1m`net.conns()`\x1b[0m                                                — List network connections\r\n");
    term.write("  \x1b[1m`user.info()`\x1b[0m                                                — Get user info and permissions\r\n");
    term.write("\r\n");
    term.write("\x1b[1;33m## Execution\x1b[0m\r\n");
    term.write("  \x1b[1m`sys.exec(\"/bin/bash\", [\"-c\", \"sleep 10\"], disown=True)`\x1b[0m     — Run a new process in the background\r\n");
    term.write("  \x1b[1m`chain.tcp(\"127.0.0.1:8080\")`\x1b[0m                                — Link a TCP bind beacon\r\n");
    term.write("  \x1b[1m`agent.set_callback_interval(5)`\x1b[0m                             — Set callback sleep to five seconds\r\n");
    term.write("  \x1b[1m`portal.create()`\x1b[0m                                            — Upgrade shell to interactive and enable socks5\r\n");
    term.write("\r\n");
    term.write("Docs: \x1b[4mhttps://docs.realm.pub/user-guide/eldritch\x1b[0m\r\n");
};

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

export const INITIAL_SPINNER_TIMEOUT_MS = 1_500;

export const useShellTerminal = (
    shellId: string | undefined,
    loading: boolean,
    error: any,
    shellData: any,
    setPortalId: (id: number | null) => void,
    isLateCheckin: boolean,
    onOpenPortalTab?: (type: string, target: string, pivotId?: string) => void,
    onCommandSubmitted?: () => void,
    shellTasks: ShellTaskItem[] = []
) => {
    const termNodeRef = useRef<HTMLDivElement | null>(null);
    const [termMounted, setTermMounted] = useState<boolean>(false);
    const [reconnectInitTick, setReconnectInitTick] = useState<number>(0);
    const termRef = useMemo<React.RefObject<HTMLDivElement>>(() => {
        const refObj = {
            get current() {
                return termNodeRef.current;
            },
            set current(node: HTMLDivElement | null) {
                const prev = termNodeRef.current;
                termNodeRef.current = node;
                if (Boolean(prev) !== Boolean(node)) {
                    setTermMounted(Boolean(node));
                }
            },
        };
        return refObj as React.RefObject<HTMLDivElement>;
    }, []);
    const termInstance = useRef<Terminal | null>(null);
    const adapter = useRef<BrowserWasmAdapter | null>(null);
    const [connectionError, setConnectionError] = useState<string | null>(null);
    const [connectionStatus, setConnectionStatus] = useState<ConnectionStatus>("disconnected");
    const [connectionMessage, setConnectionMessage] = useState<string>("");
    const [isReplReady, setIsReplReady] = useState<boolean>(false);
    const [isInitialConnecting, setIsInitialConnecting] = useState<boolean>(true);
    const [localHistory, setLocalHistory] = useState<string[]>(() => loadHistory());

    const portalIdRef = useRef<number | null>(null);
    const isReplReadyRef = useRef<boolean>(false);
    const sessionInputs = useRef<string[]>([]);
    const shellTasksRef = useRef<ShellTaskItem[]>(shellTasks);
    const seenTaskIdsRef = useRef<Set<number>>(new Set());
    const seenBeforeReconnectRef = useRef<Set<number>>(new Set());
    const hasConnectedOnceRef = useRef<boolean>(false);
    const initialConnectTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);
    const onCommandSubmittedRef = useRef(onCommandSubmitted);
    const onOpenPortalTabRef = useRef(onOpenPortalTab);

    useEffect(() => {
        onCommandSubmittedRef.current = onCommandSubmitted;
        onOpenPortalTabRef.current = onOpenPortalTab;
    }, [onCommandSubmitted, onOpenPortalTab]);

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

    // Hydrate terminal history from GraphQL shellTasks when loaded
    useEffect(() => {
        shellTasksRef.current = shellTasks;
        if (!shellTasks || shellTasks.length === 0) return;

        const gqlInputs = shellTasks
            .map((t) => stripInternalMacro(t.input || ""))
            .filter(Boolean);
        if (gqlInputs.length === 0) return;

        const existing = shellState.current.history;
        const merged = [...existing];
        for (const cmd of gqlInputs) {
            if (merged[merged.length - 1] !== cmd && !merged.includes(cmd)) {
                merged.push(cmd);
            }
        }
        shellState.current.history = merged;
        saveHistory(merged);
        setLocalHistory(merged);
    }, [shellTasks]);

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

    const shellNodeId = shellData?.node?.id;
    const shellClosedAt = shellData?.node?.closedAt;

    // Ref for late checkin to access in event handlers
    const isLateCheckinRef = useRef(isLateCheckin);
    const connectionStatusRef = useRef(connectionStatus);
    const shellClosedAtRef = useRef(shellClosedAt);

    useEffect(() => {
        isLateCheckinRef.current = isLateCheckin;
        connectionStatusRef.current = connectionStatus;
        shellClosedAtRef.current = shellClosedAt;
        if (termInstance.current) {
            const isDimmed = connectionStatus !== "connected" || !!shellClosedAt;
            termInstance.current.options.theme = {
                foreground: isDimmed ? "#7b849b" : "#e2e8f0",
                background: "#11131a",
                cursor: "#e2e8f0",
                cursorAccent: "#11131a",
                selectionBackground: "rgba(148, 163, 184, 0.28)",
            };
        }
    }, [isLateCheckin, connectionStatus, shellClosedAt]);

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
            const formatted = formatMultilineTerminalBuffer(
                state.inputBuffer,
                state.cursorPos,
                state.prompt,
                "... "
            );
            contentToWrite = formatted.contentToWrite;
            contentToDisplay = formatted.contentToDisplay;
            cursorIndex = formatted.cursorIndex;
        }

        const termCols = Math.max(1, term.cols || 80);
        const getVisualLineCount = (text: string, cols: number) => {
            const lines = text.split("\n");
            let count = 0;
            for (let i = 0; i < lines.length; i++) {
                const line = lines[i];
                if (i > 0) count++;
                if (line.length > 0) {
                    count += Math.floor((line.length - 1) / cols);
                }
            }
            return count;
        };

        const rows = getVisualLineCount(contentToWrite, termCols);

        const prevRows = lastBufferHeight.current;
        if (prevRows > 0) {
            term.write(`\x1b[${prevRows}A`);
        }

        term.write("\r\x1b[J");
        term.write(contentToDisplay.replace(/\n/g, "\r\n"));

        lastBufferHeight.current = rows;

        if (!state.isSearching) {
            const prefix = contentToWrite.slice(0, cursorIndex);
            const cursorRow = getVisualLineCount(prefix, termCols);
            const lastLine = prefix.split("\n").pop() || "";
            const cursorCol = lastLine.length % termCols;

            const moveUp = rows - cursorRow;
            if (moveUp > 0) {
                term.write(`\x1b[${moveUp}A`);
            }

            term.write("\r");
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
        if (!termInstance.current || !termNodeRef.current) return;

        // Simple debounce could be added here if performance is an issue

        const rect = termNodeRef.current.getBoundingClientRect();
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

    useEffect(() => {
        if (loading) return;
        if (!shellId) {
            setConnectionError("No Shell ID provided in URL.");
            return;
        }
        if (error && !shellNodeId) {
            setConnectionError(`Failed to load shell: ${error.message}`);
            return;
        }
        if (!shellNodeId) {
            setConnectionError("Shell not found.");
            return;
        }
        setConnectionError(null);
    }, [shellId, loading, error, shellNodeId]);

    useEffect(() => {
        if (!termNodeRef.current || !shellId || !shellNodeId) return;

        portalIdRef.current = null;
        isReplReadyRef.current = false;
        setIsReplReady(false);
        setIsInitialConnecting(true);
        if (initialConnectTimeoutRef.current) {
            clearTimeout(initialConnectTimeoutRef.current);
        }
        initialConnectTimeoutRef.current = setTimeout(() => {
            initialConnectTimeoutRef.current = null;
            setIsInitialConnecting(false);
        }, Math.min(INITIAL_SPINNER_TIMEOUT_MS, WEBSOCKET_CONNECT_TIMEOUT_MS));
        seenTaskIdsRef.current.clear();
        seenBeforeReconnectRef.current.clear();
        hasConnectedOnceRef.current = false;
        lastBufferHeight.current = 0;
        shellState.current.inputBuffer = "";
        shellState.current.currentBlock = "";
        shellState.current.cursorPos = 0;
        shellState.current.historyIndex = -1;
        shellState.current.prompt = ">>> ";
        shellState.current.isSearching = false;
        shellState.current.searchQuery = "";

        // Initialize terminal
        termInstance.current = new Terminal({
            cursorBlink: true,
            macOptionIsMeta: true,
            theme: {
                background: "#11131a",
                foreground: connectionStatusRef.current !== "connected" ? "#7b849b" : "#e2e8f0",
                cursor: "#e2e8f0",
                cursorAccent: "#11131a",
                selectionBackground: "rgba(148, 163, 184, 0.28)",
            },
            fontFamily: "'JetBrains Mono', 'Fira Code', 'SFMono-Regular', Menlo, Monaco, Consolas, monospace",
            fontSize: 13.5,
            lineHeight: 1.35,
            scrollback: 10_000_000,
        });

        // Prevent CSI 3 J (\x1b[3J) from ever wiping the scrollback buffer
        termInstance.current.parser?.registerCsiHandler({ final: "J" }, (params) => {
            return params[0] === 3;
        });

        const fitAddon = new FitAddon();
        termInstance.current.loadAddon(fitAddon);
        termInstance.current.open(termNodeRef.current);
        fitAddon.fit();

        const handleResize = () => {
            fitAddon.fit();
        };
        window.addEventListener("resize", handleResize);

        // Also observe container size changes (e.g. tab switches)
        const resizeObserver = new ResizeObserver(() => {
            fitAddon.fit();
        });
        resizeObserver.observe(termNodeRef.current);

        termInstance.current.write("Eldritch v0.4.1\r\n");

        const scheduleRedraw = () => {
            if (redrawTimeoutRef.current) {
                clearTimeout(redrawTimeoutRef.current);
            }
            redrawTimeoutRef.current = setTimeout(() => {
                redrawLine();
            }, 35);
        };

        const scheme = window.location.protocol === "https:" ? "wss" : "ws";
        const url = `${scheme}://${window.location.host}/shell/ws?shell_id=${shellId}`;

        adapter.current = new BrowserWasmAdapter(
            url,
            (msg: WebsocketMessage) => {
                const term = termInstance.current;
                if (!term) return;

                // Deduplicate historical task output re-sent by the server after a WebSocket reconnect
                const maybeTaskId = (msg as any).shell_task_id;
                if (
                    typeof maybeTaskId === "number" &&
                    maybeTaskId > 0 &&
                    (msg.kind === WebsocketMessageKind.Output ||
                        msg.kind === WebsocketMessageKind.TaskError ||
                        msg.kind === WebsocketMessageKind.OutputFromOtherStream)
                ) {
                    if (seenBeforeReconnectRef.current.has(maybeTaskId)) {
                        return;
                    }
                    seenTaskIdsRef.current.add(maybeTaskId);
                }

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
                            color = "\x1b[38;5;178m"; // Gold
                        } else if (msg.signal === WebsocketControlFlowSignal.PortalUpgrade && msg.portal_id) {
                            const wasUpgraded = portalIdRef.current !== msg.portal_id;
                            portalIdRef.current = msg.portal_id;
                            setPortalId(msg.portal_id);
                            onCommandSubmittedRef.current?.();
                            if (wasUpgraded) {
                                content = `[*] Portal Active (#${msg.portal_id})\n`;
                                color = "\x1b[38;5;42m"; // Green
                            }
                        } else if (msg.signal === WebsocketControlFlowSignal.PortalDowngrade) {
                            portalIdRef.current = null;
                            setPortalId(null);
                            onCommandSubmittedRef.current?.();
                            content = "[*] Portal Closed (Async Mode)\n";
                            color = "\x1b[38;5;178m"; // Gold
                        }
                        break;
                    case WebsocketMessageKind.OutputFromOtherStream:
                        content = msg.output;
                        break;
                }

                if (content) {
                    const formatted = transformClearAnsiForScrollback(
                        content.replace(/\n/g, "\r\n"),
                        term.rows,
                        getViewportNonEmptyLineCount(term)
                    );
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
                isReplReadyRef.current = true;
                setIsReplReady(true);
            },
            (status: ConnectionStatus, message?: string) => {
                const prevStatus = connectionStatusRef.current;
                connectionStatusRef.current = status;
                setConnectionStatus(status);
                setConnectionMessage(message || "");
                if (status === "disconnected" || status === "reconnecting") {
                    seenBeforeReconnectRef.current = new Set(seenTaskIdsRef.current);
                    if (hasConnectedOnceRef.current || !initialConnectTimeoutRef.current) {
                        setIsInitialConnecting(false);
                    }
                } else if (status === "connected") {
                    if (initialConnectTimeoutRef.current) {
                        clearTimeout(initialConnectTimeoutRef.current);
                        initialConnectTimeoutRef.current = null;
                    }
                    setIsInitialConnecting(false);
                    if (hasConnectedOnceRef.current && prevStatus !== "connected") {
                        redrawLine();
                    }
                    hasConnectedOnceRef.current = true;
                }
            }
        );

        adapter.current.init();

        const handleData = (data: string, isPaste: boolean = false) => {
            // Check for closed shell or disconnected websocket and block input
            if (shellClosedAtRef.current) return;
            if (connectionStatusRef.current !== "connected") return;

            const code = data.charCodeAt(0);
            const state = shellState.current;
            const term = termInstance.current;
            if (!term) return;

            // If completions are showing, handle navigation
            if (completionsRef.current.show && !isPaste) {
                if (code === 9) { // Tab: cycle or apply if single
                    const list = completionsRef.current.list;
                    if (list.length === 1) {
                        applyCompletion(list[0]);
                    } else if (list.length > 1) {
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
                const clearSeq = buildScrollbackPreservingClearSequence(
                    term.rows,
                    term.buffer?.active?.cursorY ?? 0,
                    lastBufferHeight.current
                );
                term.write(clearSeq);
                lastBufferHeight.current = 0;
                redrawLine();
                term.scrollToBottom?.();
                return;
            }

            if (data === "\x01" || data === "\x1b[H" || data === "\x1bOH" || data === "\x1b[1~" || data === "\x1b[7~") { // Ctrl+A / Home
                state.cursorPos = moveCursorToLineStart(state.inputBuffer, state.cursorPos);
                redrawLine();
                return;
            }

            if (data === "\x05" || data === "\x1b[F" || data === "\x1bOF" || data === "\x1b[4~" || data === "\x1b[8~") { // Ctrl+E / End
                state.cursorPos = moveCursorToLineEnd(state.inputBuffer, state.cursorPos);
                redrawLine();
                return;
            }

            if (data === "\x15") { // Ctrl+U
                const { lineStartOffset } = getCursorLineInfo(state.inputBuffer, state.cursorPos);
                state.inputBuffer = state.inputBuffer.slice(0, lineStartOffset) + state.inputBuffer.slice(state.cursorPos);
                state.cursorPos = lineStartOffset;
                redrawLine();
                return;
            }

            if (data === "\x0b") { // Ctrl+K (Kill to end of current line)
                const { lineEndOffset } = getCursorLineInfo(state.inputBuffer, state.cursorPos);
                state.inputBuffer = state.inputBuffer.slice(0, state.cursorPos) + state.inputBuffer.slice(lineEndOffset);
                redrawLine();
                return;
            }

            // Explicit newline insertion: Alt+Enter or Shift+Enter (\n)
            if (data === "\x1b\r" || data === "\n") {
                const next = insertNewlineWithAutoIndent(state.inputBuffer, state.cursorPos);
                state.inputBuffer = next.buffer;
                state.cursorPos = next.cursorPos;
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

            if (data === "\x1b[3~") { // Delete (Forward Delete / Fn+Backspace)
                if (state.cursorPos < state.inputBuffer.length) {
                    state.inputBuffer = state.inputBuffer.slice(0, state.cursorPos) + state.inputBuffer.slice(state.cursorPos + 1);
                    redrawLine();
                }
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

            if (data === "\x1b[A") { // Up: navigate to previous line inside multi-line block first, then history
                const upPos = moveCursorUpInBuffer(state.inputBuffer, state.cursorPos);
                if (upPos !== null) {
                    state.cursorPos = upPos;
                    redrawLine();
                    return;
                }
                if (state.history.length > 0) {
                    if (state.historyIndex === -1) state.historyIndex = state.history.length - 1;
                    else if (state.historyIndex > 0) state.historyIndex--;
                    state.inputBuffer = state.history[state.historyIndex];
                    state.cursorPos = state.inputBuffer.length;
                    redrawLine();
                }
                return;
            }

            if (data === "\x1b[B") { // Down: navigate to next line inside multi-line block first, then history
                const downPos = moveCursorDownInBuffer(state.inputBuffer, state.cursorPos);
                if (downPos !== null) {
                    state.cursorPos = downPos;
                    redrawLine();
                    return;
                }
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
                    const prevChar = state.inputBuffer[state.cursorPos - 1];
                    state.cursorPos--;
                    if (prevChar === "\n" || state.inputBuffer.includes("\n")) {
                        redrawLine();
                    } else {
                        term.write("\x1b[D");
                    }
                }
                return;
            }

            if (data === "\x1b[C") { // Right
                if (state.cursorPos < state.inputBuffer.length) {
                    const currChar = state.inputBuffer[state.cursorPos];
                    state.cursorPos++;
                    if (currChar === "\n" || state.inputBuffer.includes("\n")) {
                        redrawLine();
                    } else {
                        term.write("\x1b[C");
                    }
                }
                return;
            }

            if (code === 0) { // Ctrl+Space
                if (!isPaste) {
                    const res = adapter.current?.complete(state.inputBuffer, state.cursorPos);
                    if (res) {
                        if (res.suggestions.length > 0) {
                            updateCompletionsUI(res.suggestions, res.start, true, 0);
                        } else {
                            updateCompletionsUI(["no suggestions"], state.cursorPos, true, 0);
                        }
                    }
                }
                return;
            }

            if (code === 9) { // Tab
                const { lines, lineIndex, col } = getCursorLineInfo(state.inputBuffer, state.cursorPos);
                const currentLineBeforeCursor = (lines[lineIndex] ?? "").slice(0, col);
                // Indent if current line before cursor is empty/whitespace or inside a string (or if we are pasting)
                if (isPaste || !currentLineBeforeCursor.trim() || isInsideString(state.inputBuffer, state.cursorPos)) {
                    const indent = "    ";
                    state.inputBuffer = state.inputBuffer.slice(0, state.cursorPos) + indent + state.inputBuffer.slice(state.cursorPos);
                    state.cursorPos += 4;
                    redrawLine();
                    return;
                }

                // Trigger completion
                const res = adapter.current?.complete(state.inputBuffer, state.cursorPos);
                if (res && res.suggestions.length > 0) {
                    if (res.suggestions.length === 1) {
                        updateCompletionsUI(res.suggestions, res.start, true, 0);
                        applyCompletion(res.suggestions[0]);
                    } else {
                        updateCompletionsUI(res.suggestions, res.start, true, 0);
                    }
                }
                return;
            }

            if (code >= 32 && code !== 127) {
                // Fast path for simple appending at the end of a single-line buffer
                if (data.length === 1 && state.cursorPos === state.inputBuffer.length && !state.inputBuffer.includes("\n")) {
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
                if (!isPaste && shouldContinueMultilineOnEnter(state.inputBuffer, state.cursorPos)) {
                    const next = insertNewlineWithAutoIndent(state.inputBuffer, state.cursorPos);
                    state.inputBuffer = next.buffer;
                    state.cursorPos = next.cursorPos;
                    redrawLine();
                    return;
                }

                // Finalize the block for execution
                const lines = state.inputBuffer.split("\n");
                let finalizedCode = state.inputBuffer;
                if (lines.length > 1 && lines[lines.length - 1].trim() === "") {
                    finalizedCode = lines.slice(0, -1).join("\n");
                }

                state.inputBuffer = finalizedCode;
                state.cursorPos = finalizedCode.length;
                redrawLine();
                term.write("\r\n");

                if (finalizedCode.trim() === "clear" || finalizedCode.trim() === "clear()") {
                    const rowsAfterEnter = Math.min(
                        Math.max(1, (term.rows || 24) - 1),
                        (term.buffer?.active?.cursorY ?? 0) + 1
                    );
                    const clearSeq = buildScrollbackPreservingClearSequence(
                        term.rows,
                        rowsAfterEnter,
                        0
                    );
                    term.write(clearSeq);
                    state.currentBlock = "";
                    state.historyIndex = -1;
                    state.inputBuffer = "";
                    state.cursorPos = 0;
                    state.prompt = ">>> ";
                    lastBufferHeight.current = 0;
                    redrawLine();
                    term.scrollToBottom?.();
                    return;
                }

                adapter.current?.reset();
                const isMultiline = finalizedCode.includes("\n");
                const res = isMultiline
                    ? adapter.current?.paste_input(finalizedCode + "\n")
                    : adapter.current?.input(finalizedCode);

                if (res?.status === "complete") {
                    if (finalizedCode.trim()) {
                        const trimmed = finalizedCode.trimEnd();
                        state.history.push(trimmed);
                        sessionInputs.current.push(trimmed);
                        saveHistory(state.history);
                        setLocalHistory([...state.history]);
                    }
                    state.currentBlock = "";
                    state.historyIndex = -1;
                    state.inputBuffer = "";
                    state.cursorPos = 0;
                    state.prompt = ">>> ";
                    onCommandSubmittedRef.current?.();
                } else if (res?.status === "meta") {
                    const metaCmd = res.meta_command;
                    if (metaCmd?.type === "help") {
                        const target = metaCmd.target;
                        if (!target) {
                            writeHelpMessage(term, adapter.current?.version || "");
                        } else {
                            const doc = docs[target];
                            if (doc) {
                                const wrappedDesc = wrapText(doc.description, term.cols);
                                term.write(`\r\n\x1b[1;36m${doc.signature}\x1b[0m\r\n`);
                                term.write(`${wrappedDesc}\r\n`);
                            } else {
                                term.write(`No documentation found for: ${target}\r\n`);
                            }
                        }
                    } else if (metaCmd?.type === "ssh") {
                        const target = metaCmd.target;
                        const pId = portalIdRef.current;
                        if (!pId) {
                            term.write(`\r\n\x1b[31mError: An active portal is required to initiate an SSH connection.\x1b[0m\r\n`);
                        } else {
                            term.write(`\r\nInitiating SSH connection to ${target}...\r\n`);
                            onOpenPortalTabRef.current?.("ssh", target);
                        }
                    } else if (metaCmd?.type === "pty") {
                        const pId = portalIdRef.current;
                        if (!pId) {
                            term.write(`\r\n\x1b[31mError: An active portal is required to initiate a PTY session.\x1b[0m\r\n`);
                        } else {
                            term.write(`\r\nOpening PTY session...\r\n`);
                            onOpenPortalTabRef.current?.("pty", "PTY");
                        }
                    }
                    term.write(">>> ");
                    state.currentBlock = "";
                    state.historyIndex = -1;
                    state.inputBuffer = "";
                    state.cursorPos = 0;
                    state.prompt = ">>> ";
                } else if (res?.status === "incomplete") {
                    adapter.current?.reset();
                    const next = insertNewlineWithAutoIndent(finalizedCode, finalizedCode.length);
                    state.inputBuffer = next.buffer;
                    state.cursorPos = next.cursorPos;
                    lastBufferHeight.current = 0;
                    redrawLine();
                    return;
                } else {
                    term.write(`\x1b[31mError: ${res?.message}\x1b[0m\r\n`);
                    if (isMultiline) {
                        // Keep the multi-line buffer intact so the operator can arrow up and fix the typo
                        state.inputBuffer = finalizedCode;
                        state.cursorPos = finalizedCode.length;
                        lastBufferHeight.current = 0;
                        redrawLine();
                        return;
                    }
                    term.write(">>> ");
                    state.currentBlock = "";
                    state.inputBuffer = "";
                    state.cursorPos = 0;
                    state.prompt = ">>> ";
                }
                lastBufferHeight.current = 0;
            } else if (code === 127) { // Backspace
                if (state.cursorPos > 0) {
                    const { lines, lineIndex, col } = getCursorLineInfo(state.inputBuffer, state.cursorPos);
                    const currentLineBefore = (lines[lineIndex] ?? "").slice(0, col);
                    // Smart dedent: if before cursor on this line is purely spaces aligned to 4 spaces, delete up to 4 spaces
                    if (col >= 4 && /^\s+$/.test(currentLineBefore) && col % 4 === 0) {
                        state.inputBuffer = state.inputBuffer.slice(0, state.cursorPos - 4) + state.inputBuffer.slice(state.cursorPos);
                        state.cursorPos -= 4;
                        redrawLine();
                    } else if (state.cursorPos === state.inputBuffer.length && !state.inputBuffer.includes("\n")) {
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
            if (!isPaste && completionsRef.current.show) {
                if (state.inputBuffer.endsWith("(") || code === 40 /* ( */) {
                    if (completionsRef.current.show) {
                        updateCompletionsUI([], 0, false, 0);
                    }
                } else {
                    const res = adapter.current?.complete(state.inputBuffer, state.cursorPos);
                    if (res && res.suggestions.length > 0) {
                        if (res.suggestions.length === 1 && state.inputBuffer.slice(res.start) === res.suggestions[0]) {
                            updateCompletionsUI([], 0, false, 0);
                        } else {
                            updateCompletionsUI(res.suggestions, res.start, true, 0);
                        }
                    } else {
                        if (completionsRef.current.show) {
                            updateCompletionsUI([], 0, false, 0);
                        }
                    }
                }
            }
        };

        termInstance.current.onData((data) => {
            // Check if this is a paste or multi-character sequence (not starting with ESC)
            if (data.length > 1 && data.charCodeAt(0) !== 27) {
                // Check for connection status and block input
                if (connectionStatusRef.current !== "connected" || shellClosedAtRef.current) return;

                const hasNewlines = data.includes('\r') || data.includes('\n');

                if (hasNewlines) {
                    // Normalize to \n and insert directly into the multi-line input buffer at the cursor
                    // so the operator can inspect, arrow Up/Down to edit any line, and press Enter to execute.
                    const normalized = data.replace(/\r\n/g, "\n").replace(/\r/g, "\n").replace(/\n+$/, "");
                    const state = shellState.current;
                    state.inputBuffer =
                        state.inputBuffer.slice(0, state.cursorPos) +
                        normalized +
                        state.inputBuffer.slice(state.cursorPos);
                    state.cursorPos += normalized.length;
                    redrawLine();
                } else {
                    for (const char of data) {
                        handleData(char, true);
                    }
                }
            } else {
                handleData(data, false);
            }
        });

        return () => {
            isReplReadyRef.current = false;
            setIsReplReady(false);
            if (initialConnectTimeoutRef.current) {
                clearTimeout(initialConnectTimeoutRef.current);
                initialConnectTimeoutRef.current = null;
            }
            window.removeEventListener("resize", handleResize);
            resizeObserver.disconnect();
            adapter.current?.close();
            adapter.current = null;
            termInstance.current?.dispose();
            termInstance.current = null;
            if (redrawTimeoutRef.current) clearTimeout(redrawTimeoutRef.current);
        };
    }, [shellId, shellNodeId, termMounted, reconnectInitTick, setPortalId, redrawLine, updateCompletionsUI, applyCompletion]);

    const getSessionInputs = useCallback(() => {
        if (shellTasksRef.current.length > 0) {
            return shellTasksRef.current
                .map((t) => stripInternalMacro(t.input || ""))
                .filter(Boolean)
                .join("\n");
        }
        return sessionInputs.current.join("\n");
    }, []);

    const setShellInput = useCallback((text: string) => {
        const term = termInstance.current;
        if (!term) return;
        const state = shellState.current;
        state.inputBuffer = text;
        state.cursorPos = text.length;
        redrawLine();
        term.focus();
    }, [redrawLine]);

    const executeCode = useCallback((rawCode: string): boolean => {
        const term = termInstance.current;
        if (!term || !adapter.current || !isReplReadyRef.current || !adapter.current.isReady()) {
            return false;
        }
        if (shellClosedAtRef.current || connectionStatusRef.current !== "connected") return false;

        const trimmedCode = normalizeMultilineScript(rawCode);
        if (!trimmedCode) return false;

        const state = shellState.current;

        // Clear any current partial input line
        const prevRows = lastBufferHeight.current;
        if (prevRows > 0) {
            term.write(`\x1b[${prevRows}A`);
        }
        term.write("\r\x1b[J");
        adapter.current.reset();

        // Render the executed code block with syntax highlighting
        const lines = trimmedCode.split("\n");
        lines.forEach((line, idx) => {
            const linePrompt = idx === 0 ? ">>> " : ".. ";
            term.write(`${linePrompt}${highlightPythonSyntax(line)}\r\n`);
        });

        const res = adapter.current.paste_input(trimmedCode);
        if (res?.status === "complete") {
            state.history.push(trimmedCode);
            sessionInputs.current.push(trimmedCode);
            saveHistory(state.history);
            setLocalHistory([...state.history]);
            state.currentBlock = "";
            state.historyIndex = -1;
            state.inputBuffer = "";
            state.cursorPos = 0;
            state.prompt = ">>> ";
            term.write(state.prompt);
            lastBufferHeight.current = 0;
            onCommandSubmittedRef.current?.();
            return true;
        } else if (res?.status === "meta") {
            const metaCmd = res.meta_command;
            if (metaCmd?.type === "help") {
                const target = metaCmd.target;
                if (!target) {
                    writeHelpMessage(term, adapter.current?.version || "");
                } else {
                    const doc = docs[target];
                    if (doc) {
                        const wrappedDesc = wrapText(doc.description, term.cols);
                        term.write(`\r\n\x1b[1;36m${doc.signature}\x1b[0m\r\n`);
                        term.write(`${wrappedDesc}\r\n`);
                    } else {
                        term.write(`No documentation found for: ${target}\r\n`);
                    }
                }
            } else if (metaCmd?.type === "ssh") {
                const target = metaCmd.target;
                const pId = portalIdRef.current;
                if (!pId) {
                    term.write(`\r\n\x1b[31mError: An active portal is required to initiate an SSH connection.\x1b[0m\r\n`);
                } else {
                    term.write(`\r\nInitiating SSH connection to ${target}...\r\n`);
                    onOpenPortalTabRef.current?.("ssh", target);
                }
            } else if (metaCmd?.type === "pty") {
                const pId = portalIdRef.current;
                if (!pId) {
                    term.write(`\r\n\x1b[31mError: An active portal is required to initiate a PTY session.\x1b[0m\r\n`);
                } else {
                    term.write(`\r\nOpening PTY session...\r\n`);
                    onOpenPortalTabRef.current?.("pty", "PTY");
                }
            }
            state.currentBlock = "";
            state.historyIndex = -1;
            state.inputBuffer = "";
            state.cursorPos = 0;
            state.prompt = ">>> ";
            term.write(state.prompt);
            lastBufferHeight.current = 0;
            return true;
        } else {
            // Incomplete or syntax error in multi-line block: reset and show message
            adapter.current.reset();
            const errMsg = res?.message || "Incomplete block (check indentation or unclosed brackets)";
            term.write(`\x1b[31mError: ${errMsg}\x1b[0m\r\n>>> `);
            state.currentBlock = "";
            state.inputBuffer = "";
            state.cursorPos = 0;
            state.prompt = ">>> ";
            lastBufferHeight.current = 0;
            return false;
        }
    }, []);

    const clearTerminal = useCallback(() => {
        const term = termInstance.current;
        if (!term) return;
        const clearSeq = buildScrollbackPreservingClearSequence(
            term.rows,
            term.buffer?.active?.cursorY ?? 0,
            lastBufferHeight.current
        );
        term.write(clearSeq);
        lastBufferHeight.current = 0;
        redrawLine();
        term.scrollToBottom?.();
    }, [redrawLine]);

    const focusTerminal = useCallback(() => {
        termInstance.current?.focus();
    }, []);

    const sendCtrlC = useCallback(() => {
        const term = termInstance.current;
        if (!term) return;
        const state = shellState.current;
        adapter.current?.reset();
        term.write("^C\r\n");
        state.inputBuffer = "";
        state.currentBlock = "";
        state.cursorPos = 0;
        state.historyIndex = -1;
        state.prompt = ">>> ";
        term.write(state.prompt);
        lastBufferHeight.current = 0;
    }, []);

    const sendCtrlR = useCallback(() => {
        const term = termInstance.current;
        if (!term) return;
        const state = shellState.current;
        state.isSearching = true;
        state.searchQuery = "";
        redrawLine();
    }, [redrawLine]);

    const reconnect = useCallback(() => {
        if (initialConnectTimeoutRef.current) {
            clearTimeout(initialConnectTimeoutRef.current);
            initialConnectTimeoutRef.current = null;
        }
        setIsInitialConnecting(false);
        if (adapter.current) {
            adapter.current.reconnect();
        } else {
            setReconnectInitTick((t) => t + 1);
        }
    }, []);

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
        isReplReady,
        isInitialConnecting,
        handleTooltipMouseEnter: cancelHideTooltip,
        handleTooltipMouseLeave: scheduleHideTooltip,
        getSessionInputs,
        setShellInput,
        executeCode,
        clearTerminal,
        localHistory,
        focusTerminal,
        sendCtrlC,
        sendCtrlR,
        reconnect
    };
};
