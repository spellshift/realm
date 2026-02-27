import { WebsocketMessage, WebsocketMessageKind } from "../pages/shellv2/websocket";

export interface ExecutionResult {
    status: "complete" | "incomplete" | "error";
    prompt?: string;
    message?: string;
}

export type ConnectionStatus = "connected" | "disconnected" | "reconnecting";

export class HeadlessWasmAdapter {
    private repl: any; // HeadlessRepl instance
    private ws: WebSocket | null = null;
    private url: string;
    private onMessageCallback: (msg: WebsocketMessage) => void;
    private onReadyCallback?: () => void;
    private onStatusChange?: (status: ConnectionStatus) => void;
    private isWsOpen: boolean = false;
    private reconnectTimer: number | null = null;
    private isClosed: boolean = false;

    constructor(
        url: string,
        onMessage: (msg: WebsocketMessage) => void,
        onReady?: () => void,
        onStatusChange?: (status: ConnectionStatus) => void
    ) {
        this.url = url;
        this.onMessageCallback = onMessage;
        this.onReadyCallback = onReady;
        this.onStatusChange = onStatusChange;

        this.connect();
    }

    private connect() {
        if (this.isClosed) return;

        // If we are already connected or connecting, close it first?
        // Logic: if connect() is called, we want to establish a new connection.
        if (this.ws) {
            try {
                this.ws.close();
            } catch (e) {
                // ignore
            }
            this.ws = null;
        }

        // Notify status as reconnecting (unless it's the very first time, but even then 'connecting' is fine)
        // Since we don't have 'connecting', 'reconnecting' is a good proxy for "trying to connect".
        // But maybe for the first time we might want to start with 'disconnected' -> 'connected'.
        // However, the prompt asked for "connected, disconnected, and reconnecting".
        this.onStatusChange?.("reconnecting");

        this.ws = new WebSocket(this.url);

        this.ws.onopen = () => {
            this.isWsOpen = true;
            this.onStatusChange?.("connected");
            this.checkReady();
            // If we had a reconnect timer active (e.g. we manually called connect), clear it
            if (this.reconnectTimer) {
                clearTimeout(this.reconnectTimer);
                this.reconnectTimer = null;
            }
        };

        this.ws.onmessage = (event) => {
            try {
                const msg = JSON.parse(event.data) as WebsocketMessage;
                this.onMessageCallback(msg);
            } catch (e) {
                console.error("Failed to parse WebSocket message", e);
            }
        };

        this.ws.onclose = () => {
            this.isWsOpen = false;
            this.onStatusChange?.("disconnected");
            this.scheduleReconnect();
        };

        this.ws.onerror = (e) => {
            // On error, we rely on onclose to handle the reconnection flow
            console.error("WebSocket error:", e);
        };
    }

    private scheduleReconnect() {
        if (this.isClosed) return;
        if (this.reconnectTimer) return;

        this.reconnectTimer = window.setTimeout(() => {
            this.reconnectTimer = null;
            this.connect();
        }, 3000);
    }

    async init() {
        try {
            // Load the WASM module dynamically from public folder
            // @ts-ignore
            const module = await import(/* webpackIgnore: true */ "/wasm/eldritch_wasm.js");
            await module.default("/wasm/eldritch_wasm_bg.wasm");
            this.repl = new module.HeadlessRepl();
            this.checkReady();
        } catch (e) {
            console.error("Failed to initialize WASM module", e);
        }
    }

    private checkReady() {
        if (this.repl && this.isWsOpen && this.onReadyCallback) {
            this.onReadyCallback();
            this.onReadyCallback = undefined; // Only call once
        }
    }

    input(line: string): ExecutionResult {
        if (!this.repl) {
            return { status: "error", message: "REPL not initialized" };
        }

        const resultJson = this.repl.input(line);
        try {
            const result = JSON.parse(resultJson);

            if (result.status === "complete") {
                if (this.isWsOpen && this.ws) {
                    this.ws.send(JSON.stringify({
                        kind: WebsocketMessageKind.Input,
                        input: result.payload
                    }));
                } else {
                    return { status: "error", message: "WebSocket not connected" };
                }
                return { status: "complete" };
            } else if (result.status === "incomplete") {
                return { status: "incomplete", prompt: result.prompt };
            } else {
                return { status: "error", message: result.message };
            }
        } catch (e) {
            console.error("Failed to parse REPL result", e);
            return { status: "error", message: "Internal REPL error" };
        }
    }

    complete(line: string, cursor: number): { suggestions: string[], start: number } {
        if (!this.repl) return { suggestions: [], start: cursor };
        // complete returns a JSON object { suggestions: [...], start: number }
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
        this.isClosed = true;
        if (this.reconnectTimer) {
            clearTimeout(this.reconnectTimer);
            this.reconnectTimer = null;
        }
        if (this.ws) {
            this.ws.onclose = null; // Prevent reconnection trigger
            this.ws.close();
            this.ws = null;
        }
    }
}
