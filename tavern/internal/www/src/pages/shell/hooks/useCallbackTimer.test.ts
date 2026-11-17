import { renderHook } from "@testing-library/react";
import {
    useCallbackTimer,
    computeCallbackStats,
    formatCallbackInterval,
    formatCountdownDuration,
} from "./useCallbackTimer";
import moment from "moment";
import { vi, describe, it, expect, beforeEach, afterEach } from "vitest";

describe("useCallbackTimer", () => {
    beforeEach(() => {
        vi.useFakeTimers();
        vi.setSystemTime(new Date("2025-01-01T12:00:00Z"));
    });

    afterEach(() => {
        vi.useRealTimers();
    });

    it("formats countdown durations simply as s / min / hr strings", () => {
        expect(formatCountdownDuration(5)).toBe("5s");
        expect(formatCountdownDuration(600)).toBe("10min");
        expect(formatCountdownDuration(3600)).toBe("1hr");
        expect(formatCountdownDuration(-3600)).toBe("1hr");
        expect(formatCountdownDuration(-45)).toBe("45s");
    });

    it("should return false for isMissedCallback and isLateCheckin when checkin is in the future", () => {
        const futureTime = moment().add(10, "minutes").toISOString();
        const beaconData = { node: { nextSeenAt: futureTime } };

        const { result } = renderHook(() => useCallbackTimer(beaconData));

        expect(result.current.isMissedCallback).toBe(false);
        expect(result.current.timeUntilCallback).toBe("Next 10min");
        // @ts-ignore
        expect(result.current.isLateCheckin).toBe(false);
    });

    it("should return true for isMissedCallback but false for isLateCheckin when checkin is 1 minute late", () => {
        const pastTime = moment().subtract(1, "minute").subtract(10, "seconds").toISOString();
        const beaconData = { node: { nextSeenAt: pastTime } };

        const { result } = renderHook(() => useCallbackTimer(beaconData));

        expect(result.current.isMissedCallback).toBe(true);
        expect(result.current.timeUntilCallback).toBe("Missing 1min ago");
        // @ts-ignore
        expect(result.current.isLateCheckin).toBe(false);
    });

    it("should return true for isLateCheckin when checkin is over 5 minutes late", () => {
        const pastTime = moment().subtract(5, "minutes").subtract(10, "seconds").toISOString();
        const beaconData = { node: { nextSeenAt: pastTime } };

        const { result } = renderHook(() => useCallbackTimer(beaconData));

        expect(result.current.isMissedCallback).toBe(true);
        expect(result.current.timeUntilCallback).toBe("Missing 5min ago");
        // @ts-ignore
        expect(result.current.isLateCheckin).toBe(true);
    });

    it("should handle null/undefined beaconData gracefully", () => {
        const { result } = renderHook(() => useCallbackTimer(null));

        expect(result.current.isMissedCallback).toBe(false);
        expect(result.current.isLateCheckin).toBe(false);
        expect(result.current.callbackStats.formattedInterval).toBe("Unknown");
        expect(result.current.callbackStats.onTimeCount).toBe(0);
        expect(result.current.callbackStats.sampleCount).toBe(0);
        expect(result.current.callbackStats.missedCount).toBe(0);
    });

    it("computes callback interval, on-time count, and mathematically infers missed callbacks when silent", () => {
        expect(formatCallbackInterval(5)).toBe("5s");
        expect(formatCallbackInterval(60)).toBe("1m (60s)");
        expect(formatCallbackInterval(90)).toBe("1m 30s (90s)");

        const edges = Array.from({ length: 100 }, (_, idx) => ({
            node: {
                id: `h-${idx}`,
                // 96 on-time check-ins (<= 2000ms), 4 delayed check-ins (15000ms)
                latency: idx < 96 ? 120 : 15000,
            },
        }));

        const stats = computeCallbackStats({
            interval: 5,
            lastSeenAt: new Date(Date.now() - 26_000).toISOString(),
            history: { totalCount: 120, edges },
        });

        expect(stats.intervalSeconds).toBe(5);
        expect(stats.formattedInterval).toBe("5s");
        expect(stats.sampleCount).toBe(100);
        expect(stats.onTimeCount).toBe(96);
        expect(stats.lateCount).toBe(4);
        // Silent for 26s with 5s interval -> floor(26/5) = 5 missed intervals
        expect(stats.missedCount).toBe(5);
        expect(stats.expectedCount).toBe(105);
        expect(stats.onTimePercentage).toBe(91);
    });
});
