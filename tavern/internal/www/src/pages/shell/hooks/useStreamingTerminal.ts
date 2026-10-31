import { useEffect, useRef } from "react";
import { Terminal } from "@xterm/xterm";
import { FitAddon } from "xterm-addon-fit";
import "@xterm/xterm/css/xterm.css";
import { WebsocketMessageKind } from "../websocket";
import {
    getViewportNonEmptyLineCount,
    isTerminalElementVisible,
    transformClearAnsiForScrollback,
} from "./shellUtils";
import {
    WEBSOCKET_CONNECT_TIMEOUT_MS,
    getExponentialBackoffDelay,
} from "../../../lib/browser-adapter";

export type PortalStreamStatus = "connecting" | "connected" | "disconnected";

export interface UseStreamingTerminalOptions {
    pivotId?: number;
    isActive?: boolean;
    layoutKey?: string;
    fontSize?: number;
    closedBannerLabel: string;
    buildWsUrl: (
        scheme: string,
        host: string,
        activePivotId?: number,
        dims?: { rows: number; cols: number }
    ) => string;
    onSocketOpen: (params: {
        term: Terminal;
        ws: WebSocket;
        isAutoRetry: boolean;
        targetPivotId?: number;
        sendResize: (ws: WebSocket | null, force?: boolean) => void;
        reportStatus: (status: PortalStreamStatus) => void;
    }) => void;
    onStatusConnected?: (params: {
        term: Terminal;
        ws: WebSocket;
        sendResize: (ws: WebSocket | null, force?: boolean) => void;
    }) => void;
    onTerminalData?: (data: string) => void;
    onConnectionStatusChange?: (status: PortalStreamStatus) => void;
    onPivotCreated?: (pivotId: string) => void;
    dependencyKey: string;
}

export function useStreamingTerminal({
    pivotId,
    isActive,
    layoutKey,
    fontSize = 13,
    closedBannerLabel,
    buildWsUrl,
    onSocketOpen,
    onStatusConnected,
    onTerminalData,
    onConnectionStatusChange,
    onPivotCreated,
    dependencyKey,
}: UseStreamingTerminalOptions) {
    const termRef = useRef<HTMLDivElement>(null);
    const termInstance = useRef<Terminal | null>(null);
    const fitAddonRef = useRef<FitAddon | null>(null);
    const fontSizeRef = useRef<number>(fontSize);
    const wsRef = useRef<WebSocket | null>(null);
    const lastSentSizeRef = useRef<{ rows: number; cols: number }>({ rows: 0, cols: 0 });
    const onConnectionStatusChangeRef = useRef(onConnectionStatusChange);
    const onPivotCreatedRef = useRef(onPivotCreated);
    const onTerminalDataRef = useRef(onTerminalData);
    const buildWsUrlRef = useRef(buildWsUrl);
    const onSocketOpenRef = useRef(onSocketOpen);
    const onStatusConnectedRef = useRef(onStatusConnected);
    const connectPivotIdRef = useRef<number | undefined>(pivotId);

    useEffect(() => {
        onConnectionStatusChangeRef.current = onConnectionStatusChange;
    }, [onConnectionStatusChange]);

    useEffect(() => {
        onPivotCreatedRef.current = onPivotCreated;
    }, [onPivotCreated]);

    useEffect(() => {
        onTerminalDataRef.current = onTerminalData;
        buildWsUrlRef.current = buildWsUrl;
        onSocketOpenRef.current = onSocketOpen;
        onStatusConnectedRef.current = onStatusConnected;
    }, [onTerminalData, buildWsUrl, onSocketOpen, onStatusConnected]);

    useEffect(() => {
        if (pivotId !== undefined) {
            connectPivotIdRef.current = pivotId;
        }
    }, [pivotId]);

    useEffect(() => {
        if (!termRef.current) return;

        const term = new Terminal({
            cursorBlink: true,
            theme: {
                background: "#11131a",
                foreground: "#e2e8f0",
                cursor: "#e2e8f0",
                cursorAccent: "#11131a",
                selectionBackground: "rgba(148, 163, 184, 0.28)",
            },
            fontFamily:
                "'JetBrains Mono', 'Fira Code', 'SFMono-Regular', Menlo, Monaco, Consolas, monospace",
            fontSize: fontSizeRef.current,
            lineHeight: 1.35,
            scrollback: 10_000_000,
        });

        term.parser?.registerCsiHandler({ final: "J" }, (params) => {
            return params[0] === 3;
        });

        const fitAddon = new FitAddon();
        fitAddonRef.current = fitAddon;
        lastSentSizeRef.current = { rows: 0, cols: 0 };
        term.loadAddon(fitAddon);

        term.open(termRef.current);
        if (isTerminalElementVisible(termRef.current)) {
            fitAddon.fit();
        }
        termInstance.current = term;

        const sendResize = (ws: WebSocket | null, force = false) => {
            if (!isTerminalElementVisible(termRef.current)) {
                return;
            }
            if (ws && ws.readyState === WebSocket.OPEN && term.rows > 0 && term.cols > 0) {
                const prev = lastSentSizeRef.current;
                if (!force && prev.rows === term.rows && prev.cols === term.cols) {
                    return;
                }
                lastSentSizeRef.current = { rows: term.rows, cols: term.cols };
                ws.send(
                    JSON.stringify({
                        kind: "resize",
                        rows: term.rows,
                        cols: term.cols,
                    })
                );
            }
        };

        const fitAndResize = (force = false) => {
            if (!isTerminalElementVisible(termRef.current)) {
                return;
            }
            fitAddon.fit();
            sendResize(wsRef.current, force);
        };

        const handleWindowResize = () => {
            fitAndResize(false);
        };
        window.addEventListener("resize", handleWindowResize);

        const resizeObserver = new ResizeObserver(() => {
            fitAndResize(false);
        });
        resizeObserver.observe(termRef.current);

        let cancelled = false;
        let autoReconnectTimer: ReturnType<typeof setTimeout> | null = null;
        let connectTimeoutTimer: ReturnType<typeof setTimeout> | null = null;
        let reconnectAttempts = 0;
        let endedRemotely = false;
        let hadFatalError = false;

        const clearSocketTimers = () => {
            if (autoReconnectTimer) {
                clearTimeout(autoReconnectTimer);
                autoReconnectTimer = null;
            }
            if (connectTimeoutTimer) {
                clearTimeout(connectTimeoutTimer);
                connectTimeoutTimer = null;
            }
        };

        const detachSocket = (ws: WebSocket | null) => {
            if (!ws) return;
            if (wsRef.current === ws) {
                wsRef.current = null;
            }
            ws.onopen = null;
            ws.onmessage = null;
            ws.onclose = null;
            ws.onerror = null;
            try {
                if (ws.readyState === WebSocket.OPEN || ws.readyState === WebSocket.CONNECTING) {
                    ws.close();
                }
            } catch {
                // Ignore close errors on stale socket
            }
        };

        const reportStatus = (status: PortalStreamStatus) => {
            onConnectionStatusChangeRef.current?.(status);
        };

        const scheduleReconnect = () => {
            if (cancelled || endedRemotely || hadFatalError) return;
            if (autoReconnectTimer) return;
            reportStatus("connecting");
            const delay = getExponentialBackoffDelay(reconnectAttempts);
            reconnectAttempts++;
            autoReconnectTimer = setTimeout(() => {
                autoReconnectTimer = null;
                openSocket(true);
            }, delay);
        };

        const openSocket = (isAutoRetry = false) => {
            if (cancelled) return;
            clearSocketTimers();
            detachSocket(wsRef.current);

            if (isTerminalElementVisible(termRef.current)) {
                fitAddon.fit();
            }
            const scheme = window.location.protocol === "https:" ? "wss" : "ws";
            const targetPivotId = connectPivotIdRef.current;
            const initialRows = term.rows;
            const initialCols = term.cols;
            const wsUrl = buildWsUrlRef.current(scheme, window.location.host, targetPivotId, {
                rows: initialRows,
                cols: initialCols,
            });
            const ws = new WebSocket(wsUrl);
            wsRef.current = ws;
            const urlHasInitialSize = wsUrl.includes("rows=") && wsUrl.includes("cols=");
            lastSentSizeRef.current = urlHasInitialSize
                ? { rows: initialRows, cols: initialCols }
                : { rows: 0, cols: 0 };

            connectTimeoutTimer = setTimeout(() => {
                if (cancelled || wsRef.current !== ws) return;
                if (ws.readyState === WebSocket.CONNECTING) {
                    detachSocket(ws);
                    scheduleReconnect();
                }
            }, WEBSOCKET_CONNECT_TIMEOUT_MS);

            ws.onopen = () => {
                if (cancelled || wsRef.current !== ws) return;
                if (connectTimeoutTimer) {
                    clearTimeout(connectTimeoutTimer);
                    connectTimeoutTimer = null;
                }
                reconnectAttempts = 0;
                onSocketOpenRef.current({
                    term,
                    ws,
                    isAutoRetry,
                    targetPivotId,
                    sendResize,
                    reportStatus,
                });
            };

            ws.onmessage = (event) => {
                if (cancelled || wsRef.current !== ws) return;
                try {
                    const msg = JSON.parse(event.data);
                    if (msg.kind === WebsocketMessageKind.Output) {
                        if (!endedRemotely) {
                            reconnectAttempts = 0;
                            reportStatus("connected");
                            term.options.cursorBlink = true;
                        }
                        term.write(
                            transformClearAnsiForScrollback(
                                msg.output,
                                term.rows,
                                getViewportNonEmptyLineCount(term),
                                term.buffer?.active?.type === "alternate"
                            )
                        );
                    } else if (msg.kind === WebsocketMessageKind.TaskError) {
                        term.write(`\x1b[31m${msg.error}\x1b[0m`);
                    } else if (msg.kind === WebsocketMessageKind.Error) {
                        hadFatalError = true;
                        reportStatus("disconnected");
                        term.options.cursorBlink = false;
                        term.write(`\r\n\x1b[31m[-] ${msg.error}\x1b[0m\r\n`);
                    } else if (msg.kind === WebsocketMessageKind.Status) {
                        if (msg.pivot_id) {
                            connectPivotIdRef.current = msg.pivot_id;
                            onPivotCreatedRef.current?.(String(msg.pivot_id));
                        }
                        if (msg.status === "connecting") {
                            reportStatus("connecting");
                        } else if (msg.status === "connected") {
                            reconnectAttempts = 0;
                            reportStatus("connected");
                            term.options.cursorBlink = true;
                            onStatusConnectedRef.current?.({ term, ws, sendResize });
                        } else if (msg.status === "closed") {
                            endedRemotely = true;
                            reportStatus("disconnected");
                            term.options.cursorBlink = false;
                            term.write(`\r\n\x1b[33m[*] ${closedBannerLabel}\x1b[0m\r\n`);
                        }
                    }
                } catch (e) {
                    console.error("Failed to parse websocket message", e);
                }
            };

            ws.onclose = () => {
                if (cancelled || wsRef.current !== ws) return;
                if (connectTimeoutTimer) {
                    clearTimeout(connectTimeoutTimer);
                    connectTimeoutTimer = null;
                }
                if (!endedRemotely && !hadFatalError) {
                    scheduleReconnect();
                    return;
                }
                reportStatus("disconnected");
                term.options.cursorBlink = false;
            };

            ws.onerror = (e) => {
                if (cancelled || wsRef.current !== ws) return;
                console.error("Portal WebSocket error:", e);
            };
        };

        // Defer WebSocket creation so React.StrictMode's synchronous mount/unmount cycle
        // does not open a duplicate ghost ShellPivot on the backend.
        const connectTimer = setTimeout(() => {
            openSocket(false);
        }, 0);

        term.onData((data) => {
            if (wsRef.current && wsRef.current.readyState === WebSocket.OPEN) {
                wsRef.current.send(
                    JSON.stringify({ kind: WebsocketMessageKind.Input, input: data })
                );
            }
            onTerminalDataRef.current?.(data);
        });

        return () => {
            cancelled = true;
            clearTimeout(connectTimer);
            clearSocketTimers();
            window.removeEventListener("resize", handleWindowResize);
            resizeObserver.disconnect();
            detachSocket(wsRef.current);
            fitAddonRef.current = null;
            termInstance.current = null;
            term.dispose();
        };
    }, [dependencyKey, closedBannerLabel]);

    useEffect(() => {
        fontSizeRef.current = fontSize;
        const term = termInstance.current;
        const fitAddon = fitAddonRef.current;
        if (!term) return;
        term.options.fontSize = fontSize;
        if (!fitAddon || !isTerminalElementVisible(termRef.current)) {
            return;
        }
        fitAddon.fit();
        if (term.rows > 0) {
            term.refresh?.(0, term.rows - 1);
        }
        const ws = wsRef.current;
        if (ws && ws.readyState === WebSocket.OPEN && term.rows > 0 && term.cols > 0) {
            const prev = lastSentSizeRef.current;
            if (prev.rows !== term.rows || prev.cols !== term.cols) {
                lastSentSizeRef.current = { rows: term.rows, cols: term.cols };
                ws.send(
                    JSON.stringify({
                        kind: "resize",
                        rows: term.rows,
                        cols: term.cols,
                    })
                );
            }
        }
    }, [fontSize]);

    const prevIsActiveRef = useRef<boolean | undefined>(isActive);

    useEffect(() => {
        const becameActive = Boolean(isActive && !prevIsActiveRef.current);
        prevIsActiveRef.current = isActive;

        if (!isActive) {
            return;
        }

        const syncLayoutAndRefresh = () => {
            const term = termInstance.current;
            const fitAddon = fitAddonRef.current;
            if (!term || !fitAddon || !isTerminalElementVisible(termRef.current)) {
                return;
            }
            fitAddon.fit();
            if (term.rows > 0) {
                term.refresh?.(0, term.rows - 1);
            }
            const ws = wsRef.current;
            if (ws && ws.readyState === WebSocket.OPEN && term.rows > 0 && term.cols > 0) {
                const prev = lastSentSizeRef.current;
                if (prev.rows !== term.rows || prev.cols !== term.cols) {
                    lastSentSizeRef.current = { rows: term.rows, cols: term.cols };
                    ws.send(
                        JSON.stringify({
                            kind: "resize",
                            rows: term.rows,
                            cols: term.cols,
                        })
                    );
                }
            }
            if (becameActive) {
                term.focus();
            }
        };

        syncLayoutAndRefresh();
        const timer = setTimeout(syncLayoutAndRefresh, 0);
        const rafId =
            typeof window !== "undefined" && typeof window.requestAnimationFrame === "function"
                ? window.requestAnimationFrame(syncLayoutAndRefresh)
                : null;
        return () => {
            clearTimeout(timer);
            if (rafId !== null && typeof window.cancelAnimationFrame === "function") {
                window.cancelAnimationFrame(rafId);
            }
        };
    }, [isActive, layoutKey]);

    return { termRef };
}

