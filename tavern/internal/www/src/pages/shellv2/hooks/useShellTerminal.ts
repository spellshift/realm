import { useEffect, useRef, useState, useCallback } from "react";
import { Terminal } from "@xterm/xterm";
import { FitAddon } from "xterm-addon-fit";
import "@xterm/xterm/css/xterm.css";
import { HeadlessWasmAdapter, ConnectionStatus } from "../../../lib/headless-adapter";
import { WebsocketMessage } from "../websocket";
import { loadHistory } from "./shellUtils";
import { ShellState } from "./types";
import { useTerminalTooltip } from "./useTerminalTooltip";
import { useTerminalCompletions } from "./useTerminalCompletions";
import { renderLine } from "./terminal/render";
import { handleTerminalInput } from "./terminal/input";
import { handleAdapterMessage } from "./terminal/adapter";

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

    // Custom hooks
    const { tooltipState, handleMouseMove } = useTerminalTooltip(termInstance, termRef);
    const {
        completions,
        completionStart,
        showCompletions,
        completionIndex,
        completionPos,
        updateCompletionsUI,
        completionsRef
    } = useTerminalCompletions(termInstance);

    const lastBufferHeight = useRef(0);
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
        renderLine(termInstance.current, shellState.current, lastBufferHeight);
    }, []);

    const scheduleRedraw = useCallback(() => {
        if (redrawTimeoutRef.current) {
            clearTimeout(redrawTimeoutRef.current);
        }
        redrawTimeoutRef.current = setTimeout(() => {
            redrawLine();
        }, 50);
    }, [redrawLine]);

    const applyCompletion = useCallback((completion: string) => {
        const state = shellState.current;
        const start = completionsRef.current.start;
        if (start >= 0 && start <= state.cursorPos) {
            const prefix = state.inputBuffer.slice(0, start);
            const suffix = state.inputBuffer.slice(state.cursorPos);
            state.inputBuffer = prefix + completion + suffix;
            state.cursorPos = start + completion.length;
            redrawLine();
        }
        updateCompletionsUI([], 0, false, 0);
    }, [redrawLine, updateCompletionsUI, completionsRef]);

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

        // We need a local redrawLine reference for the adapter callback closure if we want to avoid deps
        // But since we use refs for state, the useCallback version is fine to call.

        const scheme = window.location.protocol === "https:" ? "wss" : "ws";
        const url = `${scheme}://${window.location.host}/shellv2/ws?shell_id=${shellId}`;

        adapter.current = new HeadlessWasmAdapter(
            url,
            (msg: WebsocketMessage) => {
                handleAdapterMessage({
                    msg,
                    term: termInstance.current,
                    lastBufferHeight,
                    redrawLine, // Using the useCallback instance
                    setPortalId
                });
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

        termInstance.current.onData((data) => {
            // Check for late checkin and block input
            if (isLateCheckinRef.current) return;
            // Check for connection status and block input
            if (connectionStatusRef.current !== "connected") return;

            handleTerminalInput({
                data,
                term: termInstance.current,
                state: shellState.current,
                adapter: adapter.current,
                completionsRef,
                updateCompletionsUI,
                applyCompletion,
                redrawLine,
                lastBufferHeight,
                scheduleRedraw
            });
        });

        return () => {
            window.removeEventListener("resize", handleResize);
            adapter.current?.close();
            termInstance.current?.dispose();
            if (redrawTimeoutRef.current) clearTimeout(redrawTimeoutRef.current);
        };
    }, [
        shellId,
        loading,
        error,
        shellNodeId,
        shellClosedAt,
        setPortalId,
        redrawLine,
        updateCompletionsUI,
        applyCompletion,
        completionsRef,
        scheduleRedraw
    ]);

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
        connectionMessage
    };
};
