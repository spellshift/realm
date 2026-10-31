import { WebsocketMessage, WebsocketMessageKind } from "../pages/shell/websocket";

export interface ExecutionResult {
    status: "complete" | "incomplete" | "error" | "meta";
    prompt?: string;
    message?: string;
    meta_command?: any;
}

export type ConnectionStatus = "connected" | "disconnected" | "reconnecting";

export const DEFAULT_ELDRITCH_VERSION = "0.4.1";

export const WEBSOCKET_CONNECT_TIMEOUT_MS = 4_000;
export const WEBSOCKET_INITIAL_BACKOFF_MS = 250;
export const WEBSOCKET_MAX_BACKOFF_MS = 5_000;

export function getExponentialBackoffDelay(
    attempt: number,
    baseDelayMs: number = WEBSOCKET_INITIAL_BACKOFF_MS,
    maxDelayMs: number = WEBSOCKET_MAX_BACKOFF_MS
): number {
    return Math.min(maxDelayMs, baseDelayMs * Math.pow(2, Math.max(0, attempt)));
}

/**
 * Normalizes a multi-line Eldritch script before sending it to the REPL:
 * - Normalizes CRLF/CR to LF and expands tabs to 4 spaces
 * - Strips leading/trailing blank lines and trailing whitespace on each line
 * - Dedents common leading whitespace across all non-empty lines so top-level
 *   definitions (like snippets inserted after indented lines) start at column 0
 */
export function normalizeMultilineScript(raw: string): string {
    const rawLines = raw
        .replace(/\r\n/g, "\n")
        .replace(/\r/g, "\n")
        .replace(/\t/g, "    ")
        .split("\n")
        .map((line) => line.replace(/\s+$/, ""));

    while (rawLines.length > 0 && rawLines[0].trim() === "") {
        rawLines.shift();
    }
    while (rawLines.length > 0 && rawLines[rawLines.length - 1].trim() === "") {
        rawLines.pop();
    }

    if (rawLines.length === 0) {
        return "";
    }

    let minIndent = Infinity;
    for (const line of rawLines) {
        if (line.trim().length === 0) continue;
        const match = line.match(/^( *)/);
        const indent = match ? match[1].length : 0;
        if (indent < minIndent) {
            minIndent = indent;
        }
    }

    const dedented =
        minIndent > 0 && minIndent < Infinity
            ? rawLines.map((line) => (line.trim().length === 0 ? "" : line.slice(minIndent)))
            : rawLines.map((line) => (line.trim().length === 0 ? "" : line));

    return dedented.join("\n");
}

export class BrowserWasmAdapter {
    private repl: any; // BrowserRepl instance
    private ws: WebSocket | null = null;
    private url: string;
    private onMessageCallback: (msg: WebsocketMessage) => void;
    private onReadyCallback?: () => void;
    private onStatusChangeCallback?: (status: ConnectionStatus, message?: string) => void;
    private isWsOpen: boolean = false;
    private reconnectTimer: ReturnType<typeof setTimeout> | null = null;
    private connectTimeoutTimer: ReturnType<typeof setTimeout> | null = null;
    private isClosedExplicitly: boolean = false;
    private reconnectAttempts: number = 0;
    private isInitializingRepl: boolean = false;
    private shouldInitRepl: boolean = false;
    private handleWindowOnline: () => void;
    private handleVisibilityChange: () => void;
    public version: string = DEFAULT_ELDRITCH_VERSION;

    constructor(
        url: string,
        onMessage: (msg: WebsocketMessage) => void,
        onReady?: () => void,
        onStatusChange?: (status: ConnectionStatus, message?: string) => void
    ) {
        this.url = url;
        this.onMessageCallback = onMessage;
        this.onReadyCallback = onReady;
        this.onStatusChangeCallback = onStatusChange;

        const isSocketActiveOrConnecting = () =>
            Boolean(
                this.ws &&
                    (this.ws.readyState === WebSocket.OPEN ||
                        this.ws.readyState === WebSocket.CONNECTING)
            );

        this.handleWindowOnline = () => {
            if (!this.isClosedExplicitly && !this.isWsOpen && !isSocketActiveOrConnecting()) {
                this.reconnect();
            }
        };

        this.handleVisibilityChange = () => {
            if (
                typeof document !== "undefined" &&
                document.visibilityState === "visible" &&
                !this.isClosedExplicitly &&
                (!this.isWsOpen || !this.ws || this.ws.readyState !== WebSocket.OPEN) &&
                !isSocketActiveOrConnecting()
            ) {
                this.reconnect();
            }
        };

        if (typeof window !== "undefined") {
            window.addEventListener("online", this.handleWindowOnline);
        }
        if (typeof document !== "undefined") {
            document.addEventListener("visibilitychange", this.handleVisibilityChange);
        }

        this.connect();
    }

    private clearTimers() {
        if (this.reconnectTimer) {
            clearTimeout(this.reconnectTimer);
            this.reconnectTimer = null;
        }
        if (this.connectTimeoutTimer) {
            clearTimeout(this.connectTimeoutTimer);
            this.connectTimeoutTimer = null;
        }
    }

    private detachCurrentSocket() {
        if (!this.ws) return;
        const oldWs = this.ws;
        this.ws = null;
        oldWs.onopen = null;
        oldWs.onmessage = null;
        oldWs.onclose = null;
        oldWs.onerror = null;
        try {
            if (oldWs.readyState === WebSocket.OPEN || oldWs.readyState === WebSocket.CONNECTING) {
                oldWs.close();
            }
        } catch {
            // Ignore close errors on stale socket
        }
    }

    private connect() {
        if (this.isClosedExplicitly) return;

        this.clearTimers();
        this.detachCurrentSocket();

        let ws: WebSocket;
        try {
            ws = new WebSocket(this.url);
        } catch (err) {
            console.error("Failed to create WebSocket:", err);
            this.isWsOpen = false;
            this.scheduleReconnect();
            return;
        }

        this.ws = ws;

        // Guard against hung CONNECTING state (e.g. initial connection or after sleep/network switch)
        this.connectTimeoutTimer = setTimeout(() => {
            if (this.ws === ws && ws.readyState === WebSocket.CONNECTING) {
                this.isWsOpen = false;
                this.detachCurrentSocket();
                this.scheduleReconnect("Connection timed out");
            }
        }, WEBSOCKET_CONNECT_TIMEOUT_MS);

        ws.onopen = () => {
            if (this.ws !== ws) return;
            this.clearTimers();
            this.isWsOpen = true;
            this.reconnectAttempts = 0;
            if (this.shouldInitRepl && !this.repl && !this.isInitializingRepl) {
                void this.init();
            }
            this.checkReady();
            this.onStatusChangeCallback?.("connected");
        };

        ws.onmessage = (event) => {
            if (this.ws !== ws) return;
            try {
                const msg = JSON.parse(event.data) as WebsocketMessage;
                this.onMessageCallback(msg);
            } catch (e) {
                console.error("Failed to parse WebSocket message", e);
            }
        };

        ws.onclose = (event: CloseEvent) => {
            if (this.ws !== ws) return;
            this.clearTimers();
            this.isWsOpen = false;
            const reason = event.reason;
            if (this.isClosedExplicitly) {
                this.onStatusChangeCallback?.("disconnected", reason);
                return;
            }

            this.onStatusChangeCallback?.("disconnected", reason);
            this.scheduleReconnect(reason);
        };

        ws.onerror = (e) => {
            if (this.ws !== ws) return;
            console.error("WebSocket error:", e);
            // onError usually precedes onClose, so we handle reconnection in onClose
        };
    }

    private scheduleReconnect(reason?: string) {
        if (this.isClosedExplicitly) return;
        if (this.reconnectTimer) {
            this.onStatusChangeCallback?.("reconnecting", reason);
            return;
        }

        this.onStatusChangeCallback?.("reconnecting", reason);
        const delay = getExponentialBackoffDelay(this.reconnectAttempts);
        this.reconnectAttempts++;

        this.reconnectTimer = setTimeout(() => {
            this.reconnectTimer = null;
            this.connect();
        }, delay);
    }

    public reconnect() {
        if (this.isClosedExplicitly) return;
        this.reconnectAttempts = 0;
        this.onStatusChangeCallback?.("reconnecting");
        if (this.shouldInitRepl && !this.repl && !this.isInitializingRepl) {
            void this.init();
        }
        this.connect();
    }

    async init() {
        this.shouldInitRepl = true;
        if (this.repl || this.isInitializingRepl) {
            this.checkReady();
            return;
        }
        this.isInitializingRepl = true;
        try {
            // Load the WASM module dynamically from public folder
            const wasmJsPath = "/wasm/eldritch_wasm.js";
            // @ts-ignore
            const module = await import(/* @vite-ignore */ /* webpackIgnore: true */ wasmJsPath);
            await module.default("/wasm/eldritch_wasm_bg.wasm");
            this.repl = new module.BrowserRepl();

            // Load version from wasm package metadata
            try {
                const resp = await fetch("/wasm/package.json");
                const pkg = await resp.json();
                this.version = pkg.version || DEFAULT_ELDRITCH_VERSION;
            } catch {
                // Version display is non-critical; silently ignore failures
            }

            this.checkReady();
        } catch (e) {
            console.error("Failed to initialize WASM module", e);
        } finally {
            this.isInitializingRepl = false;
        }
    }

    private checkReady() {
        if (this.repl && this.isWsOpen && this.onReadyCallback) {
            this.onReadyCallback();
            this.onReadyCallback = undefined;
        }
    }

    private isSocketReady(): boolean {
        return Boolean(this.isWsOpen && this.ws && this.ws.readyState === WebSocket.OPEN);
    }

    public isReady(): boolean {
        return Boolean(this.repl && this.isSocketReady());
    }

    input(line: string): ExecutionResult {
        if (!this.repl) {
            return { status: "error", message: "REPL not initialized" };
        }
        if (!this.isSocketReady()) {
            this.reconnect();
            return { status: "error", message: "WebSocket not connected (reconnecting...)" };
        }

        let resultJson = this.repl.input(line);
        try {
            let result = JSON.parse(resultJson);

            // In BrowserRepl::input (eldritch-wasm), `!command` macros expand into a 3-line
            // `for _nonomacroclowntown in range(1):` block on the first `input()` call and
            // require a follow-up `input("")` call to finalize.
            if (result.status === "incomplete" && line.trimStart().startsWith("!")) {
                resultJson = this.repl.input("");
                result = JSON.parse(resultJson);
            }

            if (result.status === "complete") {
                if (this.isSocketReady() && this.ws) {
                    this.ws.send(JSON.stringify({
                        kind: WebsocketMessageKind.Input,
                        input: result.payload
                    }));
                } else {
                    this.reconnect();
                    return { status: "error", message: "WebSocket not connected" };
                }
                return { status: "complete" };
            } else if (result.status === "incomplete") {
                return { status: "incomplete", prompt: result.prompt };
            } else if (result.status === "meta") {
                return { status: "meta", meta_command: result.meta_command };
            } else {
                return { status: "error", message: result.message };
            }
        } catch (e) {
            console.error("Failed to parse REPL result", e);
            return { status: "error", message: "Internal REPL error" };
        }
    }

    paste_input(text: string): ExecutionResult {
        if (!this.repl) {
            return { status: "error", message: "REPL not initialized" };
        }
        if (!this.isSocketReady()) {
            this.reconnect();
            return { status: "error", message: "WebSocket not connected (reconnecting...)" };
        }

        const normalized = normalizeMultilineScript(text);
        if (!normalized) {
            return { status: "incomplete", prompt: ">>> " };
        }

        this.repl.reset();
        let resultJson = this.repl.input(normalized);

        try {
            let result = JSON.parse(resultJson);

            // In BrowserRepl::input (eldritch-wasm), multi-line blocks or single-line
            // headers ending with ':' require a trailing empty line ("") call to finalize.
            if (result.status === "incomplete") {
                resultJson = this.repl.input("");
                result = JSON.parse(resultJson);
            }

            if (result.status === "complete") {
                if (this.isSocketReady() && this.ws) {
                    this.ws.send(JSON.stringify({
                        kind: WebsocketMessageKind.Input,
                        input: result.payload
                    }));
                } else {
                    this.reconnect();
                    return { status: "error", message: "WebSocket not connected" };
                }
                return { status: "complete" };
            } else if (result.status === "incomplete") {
                this.repl.reset();
                return { status: "incomplete", prompt: result.prompt };
            } else if (result.status === "meta") {
                return { status: "meta", meta_command: result.meta_command };
            } else {
                this.repl.reset();
                return { status: "error", message: result.message };
            }
        } catch (e) {
            console.error("Failed to parse REPL result", e);
            this.repl.reset();
            return { status: "error", message: "Internal REPL error" };
        }
    }

    complete(line: string, cursor: number): { suggestions: string[], start: number } {
        if (!this.repl) return { suggestions: [], start: cursor };
        const resultJson = this.repl.complete(line, cursor);
        try {
            return JSON.parse(resultJson);
        } catch (e) {
            console.error("Failed to parse completion result", e);
            return { suggestions: [], start: cursor };
        }
    }

    reset() {
        if (this.repl) {
            this.repl.reset();
        }
    }

    close() {
        this.isClosedExplicitly = true;
        this.clearTimers();
        if (typeof window !== "undefined") {
            window.removeEventListener("online", this.handleWindowOnline);
        }
        if (typeof document !== "undefined") {
            document.removeEventListener("visibilitychange", this.handleVisibilityChange);
        }
        this.isWsOpen = false;
        this.detachCurrentSocket();
    }
}
