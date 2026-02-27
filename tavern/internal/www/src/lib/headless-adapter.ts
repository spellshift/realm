import { WebsocketMessage, WebsocketMessageKind } from "../pages/shellv2/websocket";

export type ConnectionStatus = "connected" | "disconnected" | "reconnecting";

export interface ExecutionResult {
    status: "complete" | "incomplete" | "error";
    prompt?: string;
    message?: string;
}

export class HeadlessWasmAdapter {
    private repl: any; // HeadlessRepl instance
    private ws: WebSocket | null = null;
    private onMessageCallback: (msg: WebsocketMessage) => void;
    private onReadyCallback?: () => void;
    private onStatusChangeCallback?: (status: ConnectionStatus) => void;
    private isWsOpen: boolean = false;
    private url: string;
    private shouldReconnect: boolean = true;
    private reconnectTimer: any = null;

    constructor(
        url: string,
        onMessage: (msg: WebsocketMessage) => void,
        onReady?: () => void,
        onStatusChange?: (status: ConnectionStatus) => void
    ) {
        this.url = url;
        this.onMessageCallback = onMessage;
        this.onReadyCallback = onReady;
        this.onStatusChangeCallback = onStatusChange;

        this.connect();
    }

    private connect() {
        if (!this.shouldReconnect) return;

        // Clear any existing timer
        if (this.reconnectTimer) {
            clearTimeout(this.reconnectTimer);
            this.reconnectTimer = null;
        }

        this.ws = new WebSocket(this.url);

        this.ws.onopen = () => {
            this.isWsOpen = true;
            this.onStatusChangeCallback?.("connected");
            this.checkReady();
        };

        this.ws.onmessage = (event) => {
            try {
                const msg = JSON.parse(event.data) as WebsocketMessage;
                // Basic validation or filtering could happen here, but we pass it all
                this.onMessageCallback(msg);
            } catch (e) {
                console.error("Failed to parse WebSocket message", e);
            }
        };

        this.ws.onclose = () => {
            this.isWsOpen = false;
            if (this.shouldReconnect) {
                this.onStatusChangeCallback?.("reconnecting");
                this.reconnectTimer = setTimeout(() => {
                    this.connect();
                }, 3000); // Retry every 3 seconds
            } else {
                this.onStatusChangeCallback?.("disconnected");
            }
        };

        this.ws.onerror = (error) => {
            console.error("WebSocket error:", error);
            // onclose will handle the reconnection logic
        };
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
        this.shouldReconnect = false;
        if (this.reconnectTimer) {
            clearTimeout(this.reconnectTimer);
            this.reconnectTimer = null;
        }
        if (this.ws) {
            this.ws.close();
            this.ws = null;
        }
    }
}
