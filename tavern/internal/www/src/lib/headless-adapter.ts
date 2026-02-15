export interface ExecutionResult {
    status: "complete" | "incomplete" | "error";
    prompt?: string;
    message?: string;
}

export class HeadlessWasmAdapter {
    private repl: any; // HeadlessRepl instance
    private ws: WebSocket;
    private onOutputCallback: (content: string) => void;
    private onReadyCallback?: () => void;
    private isWsOpen: boolean = false;

    constructor(url: string, onOutput: (content: string) => void, onReady?: () => void) {
        this.onOutputCallback = onOutput;
        this.onReadyCallback = onReady;
        this.ws = new WebSocket(url);

        this.ws.onopen = () => {
            this.isWsOpen = true;
            this.checkReady();
        };

        this.ws.onmessage = (event) => {
            try {
                const msg = JSON.parse(event.data);
                if (msg.type === "OUTPUT") {
                    this.onOutputCallback(msg.content);
                }
            } catch (e) {
                console.error("Failed to parse WebSocket message", e);
            }
        };

        this.ws.onclose = () => {
            this.isWsOpen = false;
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
                if (this.isWsOpen) {
                    this.ws.send(JSON.stringify({
                        type: "EXECUTE",
                        command: result.payload
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

    complete(line: string, cursor: number): string[] {
        if (!this.repl) return [];
        const resultJson = this.repl.complete(line, cursor);
        try {
            return JSON.parse(resultJson);
        } catch (e) {
            console.error("Failed to parse completion result", e);
            return [];
        }
    }

    close() {
        if (this.ws) {
            this.ws.close();
        }
    }
}
