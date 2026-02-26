import { renderHook, act } from "@testing-library/react";
import { useCallbackTimer } from "./useCallbackTimer";
import moment from "moment";
import { vi, describe, it, expect, beforeEach, afterEach } from "vitest";

describe("useCallbackTimer", () => {
    beforeEach(() => {
        vi.useFakeTimers();
    });

    afterEach(() => {
        vi.useRealTimers();
    });

    it("should return false for isMissedCallback and isLateCheckin when checkin is in the future", () => {
        const futureTime = moment().add(10, "minutes").toISOString();
        const beaconData = { node: { nextSeenAt: futureTime } };

        const { result } = renderHook(() => useCallbackTimer(beaconData));

        expect(result.current.isMissedCallback).toBe(false);
        // @ts-ignore
        expect(result.current.isLateCheckin).toBe(false);
    });

    it("should return true for isMissedCallback but false for isLateCheckin when checkin is 1 minute late", () => {
        const pastTime = moment().subtract(1, "minute").subtract(10, "seconds").toISOString();
        const beaconData = { node: { nextSeenAt: pastTime } };

        const { result } = renderHook(() => useCallbackTimer(beaconData));

        expect(result.current.isMissedCallback).toBe(true);
        // @ts-ignore
        expect(result.current.isLateCheckin).toBe(false);
    });

    it("should return true for isLateCheckin when checkin is over 5 minutes late", () => {
        const pastTime = moment().subtract(5, "minutes").subtract(10, "seconds").toISOString();
        const beaconData = { node: { nextSeenAt: pastTime } };

        const { result } = renderHook(() => useCallbackTimer(beaconData));

        expect(result.current.isMissedCallback).toBe(true);
        // @ts-ignore
        expect(result.current.isLateCheckin).toBe(true);
    });

    it("should handle null/undefined beaconData gracefully", () => {
        const { result } = renderHook(() => useCallbackTimer(null));

        expect(result.current.isMissedCallback).toBe(false);
        // @ts-ignore
        expect(result.current.isLateCheckin).toBe(false);
    });
});
