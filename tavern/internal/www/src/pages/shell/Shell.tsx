import { PageWrapper } from "../../features/page-wrapper";
import { Terminal } from "@xterm/xterm";
import { AttachAddon } from 'xterm-addon-attach';
import { useState, useEffect, useRef } from 'react';
import { useParams } from "react-router-dom";
import { useToast } from "@chakra-ui/react";
import '@xterm/xterm/css/xterm.css';
import { EmptyState, EmptyStateType } from "../../components/tavern-base-ui/EmptyState";
import Button from "../../components/tavern-base-ui/button/Button";
import Badge from "../../components/tavern-base-ui/badge/Badge";
import Breadcrumbs from "../../components/Breadcrumbs";

// This Proxy enables us to hook into the websocket to add JSON wrapping, which allows us to add
// ping messages to track latency.
class WebSocketProxy extends WebSocket {
    private onMessageCallback: ((this: WebSocket, ev: MessageEvent) => any) | null = null;
    private onOpenCallback: ((this: WebSocket, ev: Event) => any) | null = null;
    private onCloseCallback: ((this: WebSocket, ev: CloseEvent) => any) | null = null;
    private onErrorCallback: ((this: WebSocket, ev: Event) => any) | null = null;
    private messageListeners: Map<EventListenerOrEventListenerObject, EventListenerOrEventListenerObject> = new Map();

    // Callbacks to hook into the stream
    public onLatencyUpdate: ((latency: number) => void) | null = null;

    constructor(url: string, protocols?: string | string[]) {
        super(url, protocols);
        // Use addEventListener to hook into the native stream
        // This ensures we get called when the native socket receives a message
        // We do NOT use super.onmessage = ... as TS forbids it and it is not reliable if we want to intercept listeners.
        // By adding a listener here, we ensure handleMessage is called.
        // Note: This does not stop other listeners from being called if they are attached to super.
        // BUT, since we override addEventListener, users (AttachAddon) will attach through our wrapper.
        super.addEventListener('message', (ev) => this.handleMessage(ev as MessageEvent));
        super.addEventListener('open', (ev) => this.handleOpen(ev));
        super.addEventListener('close', (ev) => this.handleClose(ev));
        super.addEventListener('error', (ev) => this.handleError(ev));
    }

    // Helper to encode UTF-8 string to Base64
    private toBase64(str: string): string {
        const bytes = new TextEncoder().encode(str);
        const binString = Array.from(bytes, (byte) =>
            String.fromCodePoint(byte),
        ).join("");
        return btoa(binString);
    }

    // Helper to decode Base64 to UTF-8 string
    private fromBase64(b64: string): string {
        const binString = atob(b64);
        const bytes = Uint8Array.from(binString, (m) => m.codePointAt(0)!);
        return new TextDecoder().decode(bytes);
    }

    // Override send to wrap data
    send(data: string | ArrayBufferLike | Blob | ArrayBufferView): void {
        // If data is binary, we need to convert to something JSON serializable.
        // xterm attach addon usually sends strings.
        // In Go json, []byte encodes to/from Base64 string.
        // So we must base64 encode our data here.
        if (typeof data === 'string') {
             const b64 = this.toBase64(data);
             super.send(JSON.stringify({
                 type: "data",
                 data: b64
             }));
        } else {
             console.error("WebSocketProxy received non-string data, not supported yet:", data);
        }
    }

    sendPing() {
        // Send a ping with current timestamp
        const now = Date.now().toString();
        const b64 = this.toBase64(now);
        super.send(JSON.stringify({
            type: "ping",
            data: b64
        }));
    }

    private processMessage(ev: MessageEvent): MessageEvent | null {
        // Parse JSON
        try {
            let jsonStr = "";
            if (typeof ev.data === "string") {
                jsonStr = ev.data;
            } else if (ev.data instanceof Blob) {
                 // Blobs are not handled yet for synchronous parsing in this structure
                 // But since we control the backend, we should be fine assuming strings or pre-decoded ArrayBuffers
                 return null;
            } else if (ev.data instanceof ArrayBuffer) {
                const dec = new TextDecoder("utf-8");
                jsonStr = dec.decode(ev.data);
            }

            const payload = JSON.parse(jsonStr);
            if (payload.type === "data") {
                 // Decode Base64
                 const str = this.fromBase64(payload.data);
                 return new MessageEvent("message", {
                     data: str
                 });
            } else if (payload.type === "ping") {
                // Calculate Latency
                const sentAtStr = this.fromBase64(payload.data);
                const sentAt = parseInt(sentAtStr);
                const now = Date.now();
                const latency = now - sentAt;
                if (this.onLatencyUpdate) {
                    this.onLatencyUpdate(latency);
                }
                return null;
            }

        } catch (e) {
            console.error("Failed to parse websocket message", e);
            // Fallback: return original event
            return ev;
        }
        return null;
    }

    private handleMessage(ev: MessageEvent) {
        const processed = this.processMessage(ev);
        if (processed) {
            if (this.onMessageCallback) {
                this.onMessageCallback.call(this, processed);
            }
        }
    }

    private handleOpen(ev: Event) {
        if (this.onOpenCallback) this.onOpenCallback.call(this, ev);
    }
    private handleClose(ev: CloseEvent) {
        if (this.onCloseCallback) this.onCloseCallback.call(this, ev);
    }
    private handleError(ev: Event) {
        if (this.onErrorCallback) this.onErrorCallback.call(this, ev);
    }

    // Override addEventListener to wrap 'message' listeners
    addEventListener(type: string, listener: EventListenerOrEventListenerObject, options?: boolean | AddEventListenerOptions): void {
        if (type === 'message') {
            // Check if we have already wrapped this listener to avoid memory leaks/duplicate additions
            if (this.messageListeners.has(listener)) {
                return;
            }
            const wrappedListener = (ev: Event) => {
                const originalEvent = ev as MessageEvent;
                const processed = this.processMessage(originalEvent);
                if (processed) {
                    if (typeof listener === 'function') {
                        listener.call(this, processed);
                    } else {
                        listener.handleEvent(processed);
                    }
                }
            };
            this.messageListeners.set(listener, wrappedListener);
            super.addEventListener(type, wrappedListener, options);
        } else {
            super.addEventListener(type, listener, options);
        }
    }

    removeEventListener(type: string, listener: EventListenerOrEventListenerObject, options?: boolean | EventListenerOptions): void {
        if (type === 'message' && this.messageListeners.has(listener)) {
            const wrapped = this.messageListeners.get(listener)!;
            super.removeEventListener(type, wrapped, options);
            this.messageListeners.delete(listener);
        } else {
            super.removeEventListener(type, listener, options);
        }
    }

    // Mimic WebSocket properties
    set onmessage(cb: (ev: MessageEvent) => any) { this.onMessageCallback = cb; }
    get onmessage() { return this.onMessageCallback as any; }

    set onopen(cb: (ev: Event) => any) { this.onOpenCallback = cb; }
    get onopen() { return this.onOpenCallback as any; }

    set onclose(cb: (ev: CloseEvent) => any) { this.onCloseCallback = cb; }
    get onclose() { return this.onCloseCallback as any; }

    set onerror(cb: (ev: Event) => any) { this.onErrorCallback = cb; }
    get onerror() { return this.onErrorCallback as any; }
}


const Shell = () => {
    const { shellId } = useParams();
    const toast = useToast();

    const [wsIsOpen, setWsIsOpen] = useState(false);
    const [latency, setLatency] = useState<number | null>(null);
    const ws = useRef<WebSocketProxy | null>(null);
    const termRef = useRef<Terminal | null>(null);
    if (termRef.current === null) {
        termRef.current = new Terminal();
    }

    // Setup WebSocket
    useEffect(() => {
        if (!ws.current) {
            const scheme = window.location.protocol === "https:" ? 'wss' : 'ws';
            // Use our Proxy
            const socket = new WebSocketProxy(`${scheme}://${window.location.host}/shell/ws?shell_id=${shellId}`);
            socket.binaryType = "arraybuffer"; // Ensure we receive arraybuffers to decode properly

            socket.onopen = (e) => {
                setWsIsOpen(true);
                toast({
                    title: 'Shell Connected',
                    description: 'Only output after your connection is displayed, so you may need to enter a newline to see the prompt',
                    status: 'success',
                    duration: 6000,
                    isClosable: true,
                })
                const attachAddon = new AttachAddon(socket);
                termRef.current?.loadAddon(attachAddon);
            };
            socket.onerror = (e) => {
                toast({
                    title: 'Shell Connection Error',
                    description: `Something went wrong with the underlying connection to the shell (${e.type})`,
                    status: 'error',
                    duration: 6000,
                    isClosable: true,
                })
            }
            socket.onclose = (e) => {
                toast({
                    title: 'Shell Closed',
                    description: `Your shell connection has been closed, however the shell may still be available (${e.type})`,
                    status: 'info',
                    duration: 6000,
                    isClosable: true,
                })
            }

            socket.onLatencyUpdate = (l) => {
                setLatency(l);
            }

            ws.current = socket;

            socket.onclose = (e) => {
                setWsIsOpen(false);
            }
        }

        // Cleanup
        return () => {
             if (ws.current) {
                 ws.current.close();
                 ws.current = null;
             }
        }
    }, [shellId]);

    // Ping Loop
    useEffect(() => {
        const timer = setInterval(() => {
            if (wsIsOpen && ws.current) {
                ws.current.sendPing();
            }
        }, 2000);
        return () => clearInterval(timer);
    }, [wsIsOpen]);

    const renderTerminal = (div: HTMLDivElement) => { if (div) { termRef.current?.open(div); } };

    //TODO: Expand to fetch active users for this page
    return (
        <PageWrapper>
            <Breadcrumbs pages={[{
                label: "Shell",
                link: "/shell"
            }]} />
            <div className="border-b-2 border-gray-200 pb-6 sm:flex flex-row sm:items-center sm:justify-between">
                <div className="flex flex-col gap-2">
                    <div className="flex flex-row gap-4 items-center">
                        <h3 className="text-xl font-semibold leading-6 text-gray-900">Shell for id:{shellId}</h3>
                        <Badge badgeStyle={{ color: "purple" }} >BETA FEATURE</Badge>
                        {latency !== null && (
                             <Badge badgeStyle={{ color: latency < 200 ? "green" : "red" }}>
                                {latency}ms
                             </Badge>
                        )}
                    </div>
                    <p className="max-w-2xl text-sm">Start by clicking inside the terminal, you may need to enter a newline to see the terminal prompt.</p>
                </div>
                <a title="Report a Bug" target="_blank" href="https://github.com/spellshift/realm/issues/new?template=bug_report.md&labels=bug&title=%5Bbug%5D%20Reverse%20Shell%3A%20%3CYOUR%20ISSUE%3E" rel="noreferrer">
                    <Button buttonStyle={{ color: "gray", size: "md" }}>
                        Report a bug
                    </Button>
                </a>
            </div>

            {
                wsIsOpen ?
                    <div id="terminal" className="w-full bg-gray-500 h-96" ref={renderTerminal} /> :
                    <EmptyState label="Connecting..." type={EmptyStateType.loading} />
            }
        </PageWrapper >
    );
}
export default Shell;
