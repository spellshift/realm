import React, { useEffect, useMemo, useRef, useState, useCallback } from "react";
import { Terminal } from "@xterm/xterm";
import { FitAddon } from "xterm-addon-fit";
import "@xterm/xterm/css/xterm.css";
import {
    BrowserWasmAdapter,
    ConnectionStatus,
    DEFAULT_ELDRITCH_VERSION,
    WEBSOCKET_CONNECT_TIMEOUT_MS,
    normalizeMultilineScript,
} from "../../../lib/browser-adapter";
import { WebsocketControlFlowSignal, WebsocketMessage, WebsocketMessageKind } from "../websocket";
import docsData from "../../../assets/eldritch-docs.json";
import {
    moveWordLeft,
    moveWordRight,
    findReverseHistoryMatch,
    highlightPythonSyntax,
    loadHistory,
    saveHistory,
    stripInternalMacro,
    isTerminalElementVisible,
    getViewportNonEmptyLineCount,
    buildScrollbackPreservingClearSequence,
    transformClearAnsiForScrollback,
    isInsideString,
    getCursorLineInfo,
    moveCursorUpInBuffer,
    moveCursorDownInBuffer,
    moveCursorToLineStart,
    moveCursorToLineEnd,
    dedentCurrentLine,
    shouldContinueMultilineOnEnter,
    insertNewlineWithAutoIndent,
    formatMultilineTerminalBuffer,
    buildTerminalRedrawSequence,
    handleTerminalCustomKeyEvent,
} from "./shellUtils";
import {
    DEFAULT_RC_ALIASES_SCRIPT,
    registerUserSymbolsFromCode,
    stdlibRegistry,
} from "../utils/eldritchLsp";
import { loadRcScript } from "../components/MultiLineComposer";
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
    preferredCol: number | null;
    savedDraftBuffer: string;
    history: string[];
    historyIndex: number;
    prompt: string;
    isSearching: boolean;
    searchQuery: string;
    searchMatchIndex: number;
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
    shellTasks: ShellTaskItem[] = [],
    fontSize: number = 13
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
    const fitAddonRef = useRef<FitAddon | null>(null);
    const fontSizeRef = useRef<number>(fontSize);
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
        preferredCol: null,
        savedDraftBuffer: "",
        history: loadHistory(),
        historyIndex: -1,
        prompt: ">>> ",
        isSearching: false,
        searchQuery: "",
        searchMatchIndex: 0,
        currentBlock: ""
    });

    // Register user symbols from RC script on terminal initialization
    useEffect(() => {
        const rcScript = loadRcScript() || DEFAULT_RC_ALIASES_SCRIPT;
        registerUserSymbolsFromCode(rcScript);
    }, []);

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
            registerUserSymbolsFromCode(cmd);
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
    const [promptPos, setPromptPos] = useState<{
        top: number;
        left: number;
        width: number;
        height: number;
        visible: boolean;
    }>({ top: 6, left: 6, width: 24, height: 18, visible: false });
    const [isAwaitingCommand, setIsAwaitingCommand] = useState(false);
    const [completedTaskIds, setCompletedTaskIds] = useState<string[]>([]);
    const awaitingCommandTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);

    const markCommandAwaiting = useCallback(() => {
        setIsAwaitingCommand(true);
        if (awaitingCommandTimeoutRef.current) {
            clearTimeout(awaitingCommandTimeoutRef.current);
        }
        awaitingCommandTimeoutRef.current = setTimeout(() => {
            awaitingCommandTimeoutRef.current = null;
            setIsAwaitingCommand(false);
        }, 3500);
    }, []);

    const clearCommandAwaiting = useCallback(() => {
        if (awaitingCommandTimeoutRef.current) {
            clearTimeout(awaitingCommandTimeoutRef.current);
            awaitingCommandTimeoutRef.current = null;
        }
        setIsAwaitingCommand(false);
    }, []);

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
    const lastCursorRow = useRef(0);

    // We need a ref to access current completions inside onData without stale closure
    const completionsRef = useRef<{ list: string[], start: number, show: boolean, index: number }>({
        list: [], start: 0, show: false, index: 0
    });

    const redrawTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);

    const shellNodeId = shellData?.node?.id;
    const shellClosedAt = shellData?.node?.closedAt;
    const beaconName = shellData?.node?.beacon?.name;

    // Ref for late checkin and beacon name to access in event handlers
    const isLateCheckinRef = useRef(isLateCheckin);
    const connectionStatusRef = useRef(connectionStatus);
    const shellClosedAtRef = useRef(shellClosedAt);
    const beaconNameRef = useRef<string>(beaconName || "");

    useEffect(() => {
        isLateCheckinRef.current = isLateCheckin;
        connectionStatusRef.current = connectionStatus;
        shellClosedAtRef.current = shellClosedAt;
        beaconNameRef.current = beaconName || "";
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
    }, [isLateCheckin, connectionStatus, shellClosedAt, beaconName]);

    const updatePromptPos = useCallback(() => {
        const term = termInstance.current;
        const container = termNodeRef.current;
        if (!term || !container || !isReplReadyRef.current || shellState.current.isSearching) {
            setPromptPos((prev) => (prev.visible ? { ...prev, visible: false } : prev));
            return;
        }

        const activeBuf = term.buffer?.active;
        const cursorY = activeBuf?.cursorY ?? 0;
        const baseY = activeBuf?.baseY ?? 0;
        const viewportY = activeBuf?.viewportY ?? 0;

        const promptScreenRow = Math.max(0, cursorY - lastCursorRow.current);
        const visibleRow = promptScreenRow + (baseY - viewportY);
        const rows = Math.max(1, term.rows || 24);
        const cols = Math.max(1, term.cols || 80);

        if (visibleRow < 0 || visibleRow >= rows) {
            setPromptPos((prev) => (prev.visible ? { ...prev, visible: false } : prev));
            return;
        }

        let offsetX = 6;
        let offsetY = 6;
        let cellHeight = fontSizeRef.current * 1.35;
        let cellWidth = fontSizeRef.current * 0.6;

        const screenEl = container.querySelector(".xterm-screen") as HTMLElement | null;
        if (screenEl) {
            const containerRect = container.getBoundingClientRect();
            const screenRect = screenEl.getBoundingClientRect();
            if (screenRect.width > 0 && screenRect.height > 0) {
                offsetX = screenRect.left - containerRect.left;
                offsetY = screenRect.top - containerRect.top;
                cellHeight = screenRect.height / rows;
                cellWidth = screenRect.width / cols;
            }
        }

        const nextTop = Math.round(offsetY + visibleRow * cellHeight);
        const nextLeft = Math.round(offsetX);
        const nextWidth = Math.max(20, Math.round(3 * cellWidth));
        const nextHeight = Math.max(16, Math.round(cellHeight));

        setPromptPos((prev) => {
            if (
                prev.visible &&
                prev.top === nextTop &&
                prev.left === nextLeft &&
                prev.width === nextWidth &&
                prev.height === nextHeight
            ) {
                return prev;
            }
            return {
                top: nextTop,
                left: nextLeft,
                width: nextWidth,
                height: nextHeight,
                visible: true,
            };
        });
    }, []);

    const redrawLine = useCallback(() => {
        if (redrawTimeoutRef.current) {
            clearTimeout(redrawTimeoutRef.current);
            redrawTimeoutRef.current = null;
        }
        const term = termInstance.current;
        if (!term) return;
        const state = shellState.current;

        let contentToWrite = "";
        let contentToDisplay = "";
        let cursorIndex = 0;

        if (state.isSearching) {
            const prompt = `(reverse-i-search)'${state.searchQuery}': `;
            const match = findReverseHistoryMatch(
                state.history,
                state.searchQuery,
                state.searchMatchIndex
            );
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
        const redraw = buildTerminalRedrawSequence(
            contentToWrite,
            contentToDisplay,
            cursorIndex,
            termCols,
            lastCursorRow.current
        );

        term.write(redraw.sequence);
        lastBufferHeight.current = redraw.totalRows;
        lastCursorRow.current = redraw.cursorRow;
        updatePromptPos();
        if (typeof window !== "undefined" && typeof window.requestAnimationFrame === "function") {
            window.requestAnimationFrame(() => updatePromptPos());
        }
    }, [updatePromptPos]);

    const updateCompletionsUI = useCallback((list: string[], start: number, show: boolean, index: number) => {
        setCompletions(list);
        setCompletionStart(start);
        setShowCompletions(show);
        setCompletionIndex(index);
        completionsRef.current = { list, start, show, index };

        if (show && termInstance.current) {
            const term = termInstance.current;
            const cursorX = term.buffer.active.cursorX;
            const cursorY = term.buffer.active.cursorY;
            const rows = Math.max(1, term.rows || 24);
            const cols = Math.max(1, term.cols || 80);
            let charHeight = fontSizeRef.current * 1.35;
            let charWidth = fontSizeRef.current * 0.6;
            const container = termNodeRef.current;
            const screenEl = container?.querySelector(".xterm-screen") as HTMLElement | null;
            if (screenEl) {
                const screenRect = screenEl.getBoundingClientRect();
                if (screenRect.width > 0 && screenRect.height > 0) {
                    charHeight = screenRect.height / rows;
                    charWidth = screenRect.width / cols;
                }
            }
            setCompletionPos({
                x: Math.round(cursorX * charWidth + 20), // + padding
                y: Math.round(cursorY * charHeight + 40), // + header/padding
            });
        }
    }, []);

    const applyCompletion = useCallback((completion: string, explicitStart?: number) => {
        if (completion === "no suggestions") {
            updateCompletionsUI([], 0, false, 0);
            return;
        }

        const state = shellState.current;
        const start = explicitStart !== undefined ? explicitStart : completionsRef.current.start;
        // Replace from start to cursorPos with completion
        // Ensure start is valid
        if (start >= 0 && start <= state.cursorPos) {
            const prefix = state.inputBuffer.slice(0, start);
            const suffix = state.inputBuffer.slice(state.cursorPos);

            let completionText = completion;
            let cursorOffset = completion.length;

            if (
                !suffix.trimStart().startsWith("(") &&
                !completion.endsWith("=") &&
                !completion.endsWith("(") &&
                !completion.endsWith(")")
            ) {
                const userFn = stdlibRegistry.userFunctions.get(completion);
                if (userFn) {
                    if (userFn.minArgs === 0) {
                        completionText = `${completion}()`;
                        cursorOffset = completionText.length;
                    } else {
                        completionText = `${completion}(`;
                        cursorOffset = completionText.length;
                    }
                } else {
                    const builtinFn = stdlibRegistry.builtins.get(completion);
                    if (builtinFn) {
                        if (builtinFn.minArgs === 0) {
                            completionText = `${completion}()`;
                            cursorOffset = completionText.length;
                        } else {
                            completionText = `${completion}(`;
                            cursorOffset = completionText.length;
                        }
                    } else {
                        const receiverMatch = prefix.match(/([a-zA-Z_][a-zA-Z0-9_]*)\.$/);
                        if (receiverMatch) {
                            const modName = receiverMatch[1];
                            const modFn = stdlibRegistry.functions.get(`${modName}.${completion}`);
                            if (modFn) {
                                if (modFn.minArgs === 0) {
                                    completionText = `${completion}()`;
                                    cursorOffset = completionText.length;
                                } else {
                                    completionText = `${completion}(`;
                                    cursorOffset = completionText.length;
                                }
                            }
                        }
                    }
                }
            }

            state.inputBuffer = prefix + completionText + suffix;
            state.cursorPos = start + cursorOffset;
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
        lastCursorRow.current = 0;
        shellState.current.inputBuffer = "";
        shellState.current.currentBlock = "";
        shellState.current.cursorPos = 0;
        shellState.current.preferredCol = null;
        shellState.current.savedDraftBuffer = "";
        shellState.current.historyIndex = -1;
        shellState.current.prompt = ">>> ";
        shellState.current.isSearching = false;
        shellState.current.searchQuery = "";
        shellState.current.searchMatchIndex = 0;

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
            fontSize: fontSizeRef.current,
            lineHeight: 1.35,
            scrollback: 10_000_000,
        });

        // Prevent CSI 3 J (\x1b[3J) from ever wiping the scrollback buffer
        termInstance.current.parser?.registerCsiHandler({ final: "J" }, (params) => {
            return params[0] === 3;
        });

        const fitAddon = new FitAddon();
        fitAddonRef.current = fitAddon;
        termInstance.current.loadAddon(fitAddon);
        termInstance.current.open(termNodeRef.current);
        if (isTerminalElementVisible(termNodeRef.current)) {
            fitAddon.fit();
        }

        termInstance.current.onWriteParsed?.(() => updatePromptPos());
        termInstance.current.onCursorMove?.(() => updatePromptPos());
        termInstance.current.onScroll?.(() => updatePromptPos());
        termInstance.current.onRender?.(() => updatePromptPos());

        const handleResize = () => {
            if (!isTerminalElementVisible(termNodeRef.current)) {
                return;
            }
            fitAddon.fit();
            termInstance.current?.refresh?.(0, Math.max(0, (termInstance.current?.rows || 1) - 1));
            updatePromptPos();
        };
        window.addEventListener("resize", handleResize);

        // Also observe container size changes (e.g. tab switches)
        const resizeObserver = new ResizeObserver(() => {
            if (!isTerminalElementVisible(termNodeRef.current)) {
                return;
            }
            fitAddon.fit();
            termInstance.current?.refresh?.(0, Math.max(0, (termInstance.current?.rows || 1) - 1));
            updatePromptPos();
        });
        resizeObserver.observe(termNodeRef.current);

        termInstance.current.write(`Eldritch v${DEFAULT_ELDRITCH_VERSION}\r\n`);

        const scheme = window.location.protocol === "https:" ? "wss" : "ws";
        const url = `${scheme}://${window.location.host}/shell/ws?shell_id=${shellId}`;

        adapter.current = new BrowserWasmAdapter(
            url,
            (msg: WebsocketMessage) => {
                const term = termInstance.current;
                if (!term) return;

                if (
                    msg.kind === WebsocketMessageKind.ControlFlow &&
                    msg.signal === WebsocketControlFlowSignal.TaskQueued
                ) {
                    markCommandAwaiting();
                    onCommandSubmittedRef.current?.();
                    return;
                }

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
                    const idStr = String(maybeTaskId);
                    setCompletedTaskIds((prev) => (prev.includes(idStr) ? prev : [...prev, idStr]));
                }

                // Clear current input line(s) from the prompt's top row before printing output
                const prevCursorRow = lastCursorRow.current;
                if (prevCursorRow > 0) {
                    term.write(`\x1b[${prevCursorRow}A`);
                }
                term.write("\r\x1b[J");

                // Process message content
                let content = "";
                let color = "";

                switch (msg.kind) {
                    case WebsocketMessageKind.Output:
                        content = msg.output;
                        clearCommandAwaiting();
                        onCommandSubmittedRef.current?.();
                        break;
                    case WebsocketMessageKind.TaskError:
                        content = msg.error;
                        color = "\x1b[38;2;255;0;0m"; // Red
                        clearCommandAwaiting();
                        onCommandSubmittedRef.current?.();
                        break;
                    case WebsocketMessageKind.Error:
                        content = msg.error;
                        color = "\x1b[38;2;255;0;0m"; // Red
                        clearCommandAwaiting();
                        break;
                    case WebsocketMessageKind.ControlFlow:
                        if (msg.signal === WebsocketControlFlowSignal.PortalUpgrade && msg.portal_id) {
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
                        onCommandSubmittedRef.current?.();
                        break;
                }

                if (content) {
                    const formatted = transformClearAnsiForScrollback(
                        content.replace(/\n/g, "\r\n"),
                        term.rows,
                        getViewportNonEmptyLineCount(term),
                        term.buffer?.active?.type === "alternate"
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
                lastCursorRow.current = 0;
                redrawLine();
            },
            () => {
                const targetBeaconName = beaconNameRef.current || "Beacon";
                termInstance.current?.write(`Connected to ${targetBeaconName}\r\n>>> `);
                lastBufferHeight.current = 0;
                lastCursorRow.current = 0;
                isReplReadyRef.current = true;
                setIsReplReady(true);
                updatePromptPos();
                if (typeof window !== "undefined" && typeof window.requestAnimationFrame === "function") {
                    window.requestAnimationFrame(() => updatePromptPos());
                }
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

            const isUpArrow = data === "\x1b[A" || data === "\x1bOA";
            const isDownArrow = data === "\x1b[B" || data === "\x1bOB";
            const isLeftArrow = data === "\x1b[D" || data === "\x1bOD";
            const isRightArrow = data === "\x1b[C" || data === "\x1bOC";

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
                if (data === "\x1b[Z") { // Shift+Tab: cycle backwards
                    const list = completionsRef.current.list;
                    if (list.length > 0) {
                        const prev = (completionsRef.current.index - 1 + list.length) % list.length;
                        updateCompletionsUI(list, completionsRef.current.start, true, prev);
                    }
                    return;
                }
                if (code === 13) { // Enter: select
                    applyCompletion(completionsRef.current.list[completionsRef.current.index]);
                    return;
                }
                if (isDownArrow) { // Down
                    const list = completionsRef.current.list;
                    if (list.length > 0) {
                        const next = (completionsRef.current.index + 1) % list.length;
                        updateCompletionsUI(list, completionsRef.current.start, true, next);
                    }
                    return;
                }
                if (isUpArrow) { // Up
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
                    state.searchMatchIndex += 1;
                    redrawLine();
                    return;
                }
                if (data === "\x03" || data === "\x07" || code === 27) { // Ctrl+C / Ctrl+G / Esc
                    state.isSearching = false;
                    state.searchQuery = "";
                    state.searchMatchIndex = 0;
                    redrawLine();
                    return;
                }
                if (code === 13) { // Enter
                    const match = findReverseHistoryMatch(
                        state.history,
                        state.searchQuery,
                        state.searchMatchIndex
                    );
                    state.isSearching = false;
                    state.inputBuffer = match;
                    state.cursorPos = match.length;
                    state.preferredCol = null;
                    state.searchQuery = "";
                    state.searchMatchIndex = 0;
                    redrawLine();
                    return;
                }
                if (code === 127 || code === 8) { // Backspace
                    state.searchQuery = state.searchQuery.slice(0, -1);
                    state.searchMatchIndex = 0;
                    redrawLine();
                    return;
                }
                if (code >= 32) {
                    state.searchQuery += data;
                    state.searchMatchIndex = 0;
                    redrawLine();
                    return;
                }
                // Ignore other keys in search mode for now
                return;
            }

            // Reset sticky vertical column on any non-vertical navigation key
            if (!isUpArrow && !isDownArrow) {
                state.preferredCol = null;
            }

            // Normal mode
            if (data === "\x03") { // Ctrl+C
                adapter.current?.reset();
                if (state.inputBuffer.length > 0 && lastCursorRow.current < lastBufferHeight.current) {
                    state.cursorPos = state.inputBuffer.length;
                    redrawLine();
                }
                term.write("^C\r\n");
                state.inputBuffer = "";
                state.currentBlock = "";
                state.cursorPos = 0;
                state.preferredCol = null;
                state.savedDraftBuffer = "";
                state.historyIndex = -1;
                state.prompt = ">>> ";
                term.write(state.prompt);
                lastBufferHeight.current = 0;
                lastCursorRow.current = 0;
                return;
            }

            if (data === "\x12") { // Ctrl+R
                state.isSearching = true;
                state.searchQuery = "";
                state.searchMatchIndex = 0;
                redrawLine();
                return;
            }

            if (data === "\x0c") { // Ctrl+L
                const clearSeq = buildScrollbackPreservingClearSequence(
                    term.rows,
                    term.buffer?.active?.cursorY ?? 0,
                    lastCursorRow.current
                );
                term.write(clearSeq);
                lastBufferHeight.current = 0;
                lastCursorRow.current = 0;
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

            // Shift+Tab: Dedent current line by up to 4 spaces
            if (data === "\x1b[Z") {
                const dedented = dedentCurrentLine(state.inputBuffer, state.cursorPos);
                state.inputBuffer = dedented.buffer;
                state.cursorPos = dedented.cursorPos;
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

            if (isUpArrow) { // Up: navigate to previous line inside multi-line block first, then history
                if (state.preferredCol === null) {
                    state.preferredCol = getCursorLineInfo(state.inputBuffer, state.cursorPos).col;
                }
                const upPos = moveCursorUpInBuffer(
                    state.inputBuffer,
                    state.cursorPos,
                    state.preferredCol
                );
                if (upPos !== null) {
                    state.cursorPos = upPos;
                    redrawLine();
                    return;
                }
                state.preferredCol = null;
                if (state.history.length > 0) {
                    if (state.historyIndex === -1) {
                        state.savedDraftBuffer = state.inputBuffer;
                        state.historyIndex = state.history.length - 1;
                    } else if (state.historyIndex > 0) {
                        state.historyIndex--;
                    }
                    state.inputBuffer = state.history[state.historyIndex];
                    state.cursorPos = state.inputBuffer.length;
                    redrawLine();
                }
                return;
            }

            if (isDownArrow) { // Down: navigate to next line inside multi-line block first, then history
                if (state.preferredCol === null) {
                    state.preferredCol = getCursorLineInfo(state.inputBuffer, state.cursorPos).col;
                }
                const downPos = moveCursorDownInBuffer(
                    state.inputBuffer,
                    state.cursorPos,
                    state.preferredCol
                );
                if (downPos !== null) {
                    state.cursorPos = downPos;
                    redrawLine();
                    return;
                }
                state.preferredCol = null;
                if (state.historyIndex !== -1) {
                    if (state.historyIndex < state.history.length - 1) {
                        state.historyIndex++;
                        state.inputBuffer = state.history[state.historyIndex];
                    } else {
                        state.historyIndex = -1;
                        state.inputBuffer = state.savedDraftBuffer;
                        state.savedDraftBuffer = "";
                    }
                    state.cursorPos = state.inputBuffer.length;
                    redrawLine();
                }
                return;
            }

            if (isLeftArrow) { // Left
                if (state.cursorPos > 0) {
                    state.cursorPos--;
                    redrawLine();
                }
                return;
            }

            if (isRightArrow) { // Right
                if (state.cursorPos < state.inputBuffer.length) {
                    state.cursorPos++;
                    redrawLine();
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
                const currentLine = lines[lineIndex] ?? "";
                const currentLineBeforeCursor = currentLine.slice(0, col);
                if (currentLine.trimStart().startsWith("!")) {
                    return;
                }
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
                        applyCompletion(res.suggestions[0], res.start);
                    } else {
                        updateCompletionsUI(res.suggestions, res.start, true, 0);
                    }
                }
                return;
            }

            if (code >= 32 && code !== 127) {
                state.inputBuffer = state.inputBuffer.slice(0, state.cursorPos) + data + state.inputBuffer.slice(state.cursorPos);
                state.cursorPos += data.length;
                redrawLine();
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

                if (!finalizedCode.trim()) {
                    state.inputBuffer = "";
                    state.cursorPos = 0;
                    redrawLine();
                    term.write("\r\n");
                    state.currentBlock = "";
                    state.historyIndex = -1;
                    state.savedDraftBuffer = "";
                    state.prompt = ">>> ";
                    term.write(state.prompt);
                    lastBufferHeight.current = 0;
                    lastCursorRow.current = 0;
                    updatePromptPos();
                    return;
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
                    state.savedDraftBuffer = "";
                    state.inputBuffer = "";
                    state.cursorPos = 0;
                    state.prompt = ">>> ";
                    lastBufferHeight.current = 0;
                    lastCursorRow.current = 0;
                    redrawLine();
                    term.scrollToBottom?.();
                    return;
                }

                adapter.current?.reset();
                if (finalizedCode.trim()) {
                    registerUserSymbolsFromCode(finalizedCode.trimEnd());
                }
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
                    state.savedDraftBuffer = "";
                    state.inputBuffer = "";
                    state.cursorPos = 0;
                    state.prompt = ">>> ";
                    term.write(state.prompt);
                    markCommandAwaiting();
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
                    state.savedDraftBuffer = "";
                    state.inputBuffer = "";
                    state.cursorPos = 0;
                    state.prompt = ">>> ";
                } else if (res?.status === "incomplete") {
                    adapter.current?.reset();
                    const next = insertNewlineWithAutoIndent(finalizedCode, finalizedCode.length);
                    state.inputBuffer = next.buffer;
                    state.cursorPos = next.cursorPos;
                    lastCursorRow.current += 1;
                    redrawLine();
                    return;
                } else {
                    term.write(`\x1b[31mError: ${res?.message}\x1b[0m\r\n`);
                    if (isMultiline) {
                        // Keep the multi-line buffer intact so the operator can arrow up and fix the typo
                        state.inputBuffer = finalizedCode;
                        state.cursorPos = finalizedCode.length;
                        lastBufferHeight.current = 0;
                        lastCursorRow.current = 0;
                        redrawLine();
                        return;
                    }
                    term.write(">>> ");
                    state.currentBlock = "";
                    state.savedDraftBuffer = "";
                    state.inputBuffer = "";
                    state.cursorPos = 0;
                    state.prompt = ">>> ";
                }
                lastBufferHeight.current = 0;
                lastCursorRow.current = 0;
                updatePromptPos();
            } else if (code === 127 || code === 8) { // Backspace
                if (state.cursorPos > 0) {
                    const { lines, lineIndex, col } = getCursorLineInfo(state.inputBuffer, state.cursorPos);
                    const currentLineBefore = (lines[lineIndex] ?? "").slice(0, col);
                    // Smart dedent: if before cursor on this line is purely spaces aligned to 4 spaces, delete up to 4 spaces
                    if (col >= 4 && /^\s+$/.test(currentLineBefore) && col % 4 === 0) {
                        state.inputBuffer = state.inputBuffer.slice(0, state.cursorPos - 4) + state.inputBuffer.slice(state.cursorPos);
                        state.cursorPos -= 4;
                        redrawLine();
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
                        if (res.suggestions.length === 1 && state.inputBuffer.slice(res.start, state.cursorPos) === res.suggestions[0]) {
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

        // Custom key handler for copy/paste (Ctrl+C, Ctrl+Shift+C, Ctrl+V, etc.) and Shift+Enter
        termInstance.current.attachCustomKeyEventHandler((ev: KeyboardEvent) => {
            if (!termInstance.current) return true;
            return handleTerminalCustomKeyEvent(ev, termInstance.current, {
                onShiftEnter: () => handleData("\n", false),
            });
        });

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
                    state.preferredCol = null;
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
            clearCommandAwaiting();
            if (initialConnectTimeoutRef.current) {
                clearTimeout(initialConnectTimeoutRef.current);
                initialConnectTimeoutRef.current = null;
            }
            window.removeEventListener("resize", handleResize);
            resizeObserver.disconnect();
            adapter.current?.close();
            adapter.current = null;
            fitAddonRef.current = null;
            termInstance.current?.dispose();
            termInstance.current = null;
            if (redrawTimeoutRef.current) clearTimeout(redrawTimeoutRef.current);
        };
    }, [shellId, shellNodeId, termMounted, reconnectInitTick, setPortalId, redrawLine, updatePromptPos, markCommandAwaiting, clearCommandAwaiting, updateCompletionsUI, applyCompletion]);

    useEffect(() => {
        fontSizeRef.current = fontSize;
        const term = termInstance.current;
        if (!term) return;
        term.options.fontSize = fontSize;
        if (isTerminalElementVisible(termNodeRef.current)) {
            fitAddonRef.current?.fit();
            term.refresh?.(0, Math.max(0, (term.rows || 1) - 1));
        }
        updatePromptPos();
    }, [fontSize, updatePromptPos]);

    const getSessionInputs = useCallback(() => {
        const currentSessionTasks = shellTasksRef.current.filter(
            (t) => !t.shellId || String(t.shellId) === String(shellId)
        );
        if (currentSessionTasks.length > 0) {
            return currentSessionTasks
                .map((t) => stripInternalMacro(t.input || ""))
                .filter(Boolean)
                .join("\n");
        }
        return sessionInputs.current.join("\n");
    }, [shellId]);

    const setShellInput = useCallback((text: string) => {
        const term = termInstance.current;
        if (!term) return;
        const state = shellState.current;
        state.inputBuffer = text;
        state.cursorPos = text.length;
        state.preferredCol = null;
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

        registerUserSymbolsFromCode(trimmedCode);

        const state = shellState.current;

        // Clear any current partial input line from the prompt's top row
        const prevCursorRow = lastCursorRow.current;
        if (prevCursorRow > 0) {
            term.write(`\x1b[${prevCursorRow}A`);
        }
        term.write("\r\x1b[J");
        adapter.current.reset();

        // Render the executed code block with syntax highlighting
        const lines = trimmedCode.split("\n");
        lines.forEach((line, idx) => {
            const linePrompt = idx === 0 ? ">>> " : "... ";
            term.write(`${linePrompt}${highlightPythonSyntax(line)}\r\n`);
        });

        const res = adapter.current.paste_input(trimmedCode);
        if (res?.status === "complete") {
            registerUserSymbolsFromCode(trimmedCode);
            state.history.push(trimmedCode);
            sessionInputs.current.push(trimmedCode);
            saveHistory(state.history);
            setLocalHistory([...state.history]);
            state.currentBlock = "";
            state.historyIndex = -1;
            state.savedDraftBuffer = "";
            state.preferredCol = null;
            state.inputBuffer = "";
            state.cursorPos = 0;
            state.prompt = ">>> ";
            term.write(state.prompt);
            lastBufferHeight.current = 0;
            lastCursorRow.current = 0;
            markCommandAwaiting();
            updatePromptPos();
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
            state.savedDraftBuffer = "";
            state.preferredCol = null;
            state.inputBuffer = "";
            state.cursorPos = 0;
            state.prompt = ">>> ";
            term.write(state.prompt);
            lastBufferHeight.current = 0;
            lastCursorRow.current = 0;
            return true;
        } else {
            // Incomplete or syntax error in multi-line block: reset and show message
            adapter.current.reset();
            const errMsg = res?.message || "Incomplete block (check indentation or unclosed brackets)";
            term.write(`\x1b[31mError: ${errMsg}\x1b[0m\r\n>>> `);
            state.currentBlock = "";
            state.savedDraftBuffer = "";
            state.preferredCol = null;
            state.inputBuffer = "";
            state.cursorPos = 0;
            state.prompt = ">>> ";
            lastBufferHeight.current = 0;
            lastCursorRow.current = 0;
            return false;
        }
    }, [markCommandAwaiting, updatePromptPos]);

    const clearTerminal = useCallback(() => {
        const term = termInstance.current;
        if (!term) return;
        const clearSeq = buildScrollbackPreservingClearSequence(
            term.rows,
            term.buffer?.active?.cursorY ?? 0,
            lastCursorRow.current
        );
        term.write(clearSeq);
        lastBufferHeight.current = 0;
        lastCursorRow.current = 0;
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
        if (state.inputBuffer.length > 0 && lastCursorRow.current < lastBufferHeight.current) {
            state.cursorPos = state.inputBuffer.length;
            redrawLine();
        }
        term.write("^C\r\n");
        state.inputBuffer = "";
        state.currentBlock = "";
        state.cursorPos = 0;
        state.preferredCol = null;
        state.savedDraftBuffer = "";
        state.historyIndex = -1;
        state.prompt = ">>> ";
        term.write(state.prompt);
        lastBufferHeight.current = 0;
        lastCursorRow.current = 0;
    }, [redrawLine]);

    const sendCtrlR = useCallback(() => {
        const term = termInstance.current;
        if (!term) return;
        const state = shellState.current;
        if (state.isSearching) {
            state.searchMatchIndex += 1;
        } else {
            state.isSearching = true;
            state.searchQuery = "";
            state.searchMatchIndex = 0;
        }
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
        promptPos,
        isAwaitingCommand,
        completedTaskIds,
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
