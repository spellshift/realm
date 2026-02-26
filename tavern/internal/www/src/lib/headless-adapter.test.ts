import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { HeadlessWasmAdapter } from "./headless-adapter";

// Mock WebSocket
class MockWebSocket {
    url: string;
    onopen: (() => void) | null = null;
    onmessage: ((event: any) => void) | null = null;
    onclose: (() => void) | null = null;
    onerror: ((error: any) => void) | null = null;
    readyState: number = 0; // CONNECTING

    constructor(url: string) {
        this.url = url;
        // Simulate async connection
        setTimeout(() => {
            this.readyState = 1; // OPEN
            if (this.onopen) this.onopen();
        }, 10);
    }

    send(data: string) {}
    close() {
        this.readyState = 3; // CLOSED
        if (this.onclose) this.onclose();
    }
}

// Override global WebSocket
global.WebSocket = MockWebSocket as any;

describe("HeadlessWasmAdapter", () => {
    beforeEach(() => {
        vi.useFakeTimers();
    });

    afterEach(() => {
        vi.useRealTimers();
        vi.clearAllMocks();
    });

    it("should connect on initialization", async () => {
        const onStatusChange = vi.fn();
        const adapter = new HeadlessWasmAdapter("ws://test", () => {}, () => {}, onStatusChange);

        // Wait for connection simulation
        await vi.advanceTimersByTimeAsync(20);

        expect(onStatusChange).toHaveBeenCalledWith("connected");
    });

    it("should reconnect on unexpected close", async () => {
        const onStatusChange = vi.fn();
        const adapter = new HeadlessWasmAdapter("ws://test", () => {}, () => {}, onStatusChange);

        await vi.advanceTimersByTimeAsync(20);
        expect(onStatusChange).toHaveBeenLastCalledWith("connected");

        // Simulate close
        // @ts-ignore
        adapter["ws"].onclose();

        expect(onStatusChange).toHaveBeenCalledWith("disconnected");
        expect(onStatusChange).toHaveBeenCalledWith("reconnecting");

        // Fast forward reconnect interval (3000ms)
        await vi.advanceTimersByTimeAsync(3000);

        // Should try to connect again
        await vi.advanceTimersByTimeAsync(20); // Connection time

        expect(onStatusChange).toHaveBeenLastCalledWith("connected");
    });

    it("should not reconnect on explicit close", async () => {
        const onStatusChange = vi.fn();
        const adapter = new HeadlessWasmAdapter("ws://test", () => {}, () => {}, onStatusChange);

        await vi.advanceTimersByTimeAsync(20);

        adapter.close();

        expect(onStatusChange).toHaveBeenCalledWith("disconnected");
        // Should NOT call "reconnecting"
        expect(onStatusChange).not.toHaveBeenCalledWith("reconnecting");

        // Fast forward
        await vi.advanceTimersByTimeAsync(3000);

        // Should not have connected again
        expect(onStatusChange).toHaveBeenCalledTimes(2); // connected, disconnected
    });
});
