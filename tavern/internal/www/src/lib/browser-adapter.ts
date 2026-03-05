import { WebsocketMessage, WebsocketMessageKind } from "../pages/shellv2/websocket";

export type ConnectionStatus = "connected" | "disconnected" | "reconnecting";

export class BrowserWasmAdapter {
    private repl: any; // BrowserRepl instance (used for tab-completion only)
    private ws: WebSocket | null = null;
    private url: string;
    private onMessageCallback: (msg: WebsocketMessage) => void;
    private onReadyCallback?: () => void;
    private onStatusChangeCallback?: (status: ConnectionStatus, message?: string) => void;
    private isWsOpen: boolean = false;
    private reconnectTimer: NodeJS.Timeout | null = null;
    private isClosedExplicitly: boolean = false;

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

        this.connect();
    }

    private connect() {
        if (this.isClosedExplicitly) return;

        this.ws = new WebSocket(this.url);

        this.ws.onopen = () => {
            this.isWsOpen = true;
            if (this.reconnectTimer) {
                clearTimeout(this.reconnectTimer);
                this.reconnectTimer = null;
            }
            this.onStatusChangeCallback?.("connected");
            this.checkReady();
        };

        this.ws.onmessage = (event) => {
            try {
                const msg = JSON.parse(event.data) as WebsocketMessage;
                this.onMessageCallback(msg);
            } catch (e) {
                console.error("Failed to parse WebSocket message", e);
            }
        };

        this.ws.onclose = (event: CloseEvent) => {
            this.isWsOpen = false;
            const reason = event.reason;
            if (this.isClosedExplicitly) {
                this.onStatusChangeCallback?.("disconnected", reason);
                return;
            }

            this.onStatusChangeCallback?.("disconnected", reason);
            this.scheduleReconnect();
        };

        this.ws.onerror = (e) => {
            console.error("WebSocket error:", e);
            // onError usually precedes onClose, so we handle reconnection in onClose
        };
    }

    private scheduleReconnect() {
        if (this.reconnectTimer || this.isClosedExplicitly) return;

        this.onStatusChangeCallback?.("reconnecting");
        this.reconnectTimer = setTimeout(() => {
            this.reconnectTimer = null;
            this.connect();
        }, 3000); // Retry every 3 seconds
    }

    async init() {
        try {
            // Load the WASM module dynamically from public folder (used for tab-completion)
            // @ts-ignore
            const module = await import(/* webpackIgnore: true */ "/wasm/eldritch_wasm.js");
            await module.default("/wasm/eldritch_wasm_bg.wasm");
            this.repl = new module.BrowserRepl();
            this.checkReady();
        } catch (e) {
            console.error("Failed to initialize WASM module", e);
        }
    }

    private checkReady() {
        if (this.repl && this.isWsOpen && this.onReadyCallback) {
            this.onReadyCallback();
            this.onReadyCallback = undefined;
        }
    }

    /** Request tab-completions from the WASM REPL for the given line/cursor position. */
    complete(line: string, cursor: number): { suggestions: string[], start: number } {
        if (!this.repl) return { suggestions: [], start: cursor };
        try {
            const resultJson = this.repl.complete(line, cursor);
            return JSON.parse(resultJson);
        } catch (e) {
            console.error("Failed to parse completion result", e);
            return { suggestions: [], start: cursor };
        }
    }

    close() {
        this.isClosedExplicitly = true;
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
