import { WebsocketMessage, WebsocketMessageKind } from "../pages/shellv2/websocket";

export interface ExecutionResult {
    status: "complete" | "incomplete" | "error";
    prompt?: string;
    message?: string;
}

export type ConnectionStatus = "connected" | "disconnected" | "reconnecting";

export class HeadlessWasmAdapter {
    private repl: any; // HeadlessRepl instance
    private ws!: WebSocket;
    private onMessageCallback: (msg: WebsocketMessage) => void;
    private onReadyCallback?: () => void;
    private onStatusChange?: (status: ConnectionStatus) => void;
    private isWsOpen: boolean = false;

    // Reconnection logic
    private url: string;
    private reconnectInterval: number = 3000;
    private isExplicitClose: boolean = false;
    private reconnectTimeout: any = null;

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
        // Cleanup existing listeners if any
        if (this.ws) {
            this.ws.onclose = null;
            this.ws.onerror = null;
            this.ws.onmessage = null;
            this.ws.onopen = null;
            this.ws.close();
        }

        this.ws = new WebSocket(this.url);

        this.ws.onopen = () => {
            this.isWsOpen = true;
            this.checkReady();
            if (this.onStatusChange) this.onStatusChange("connected");

            // Clear any pending reconnect timeout
            if (this.reconnectTimeout) {
                clearTimeout(this.reconnectTimeout);
                this.reconnectTimeout = null;
            }
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
            if (this.onStatusChange) this.onStatusChange("disconnected");

            if (!this.isExplicitClose) {
                if (this.onStatusChange) this.onStatusChange("reconnecting");
                this.reconnectTimeout = setTimeout(() => {
                    this.connect();
                }, this.reconnectInterval);
            }
        };

        this.ws.onerror = (error) => {
            console.error("WebSocket error:", error);
            // onclose will be called after onerror usually
        };
    }

    async init() {
        try {
            // Load the WASM module dynamically from public folder
            const wasmPath = "/wasm/eldritch_wasm.js";
            // @ts-ignore
            const module = await import(/* webpackIgnore: true */ /* @vite-ignore */ wasmPath);
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
                if (this.isWsOpen) {
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
        this.isExplicitClose = true;
        if (this.reconnectTimeout) {
            clearTimeout(this.reconnectTimeout);
            this.reconnectTimeout = null;
        }
        if (this.ws) {
            this.ws.close();
        }
    }

    getStatus(): ConnectionStatus {
        if (this.isWsOpen) return "connected";
        if (!this.isExplicitClose && this.reconnectTimeout) return "reconnecting";
        return "disconnected";
    }
}
