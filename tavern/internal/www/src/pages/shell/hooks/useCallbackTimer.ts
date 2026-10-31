import { useEffect, useMemo, useState } from "react";
import moment from "moment";

export interface BeaconHistoryNode {
    id?: string;
    createdAt?: string;
    latency: number;
}

export interface CallbackStats {
    intervalSeconds: number | null;
    formattedInterval: string;
    sampleCount: number;
    onTimeCount: number;
    lateCount: number;
    onTimePercentage: number;
    avgLatencyMs: number | null;
    lastSeenAt: string | null;
    nextSeenAt: string | null;
}

export const ON_TIME_MIN_TOLERANCE_MS = 2000;

export const formatCallbackInterval = (intervalSeconds: number | null | undefined): string => {
    const sec = Number(intervalSeconds);
    if (intervalSeconds == null || Number.isNaN(sec) || sec <= 0) {
        return "Unknown";
    }
    if (sec < 60) {
        return `${sec}s`;
    }
    const hours = Math.floor(sec / 3600);
    const minutes = Math.floor((sec % 3600) / 60);
    const remainingSeconds = sec % 60;
    const parts: string[] = [];
    if (hours > 0) parts.push(`${hours}h`);
    if (minutes > 0) parts.push(`${minutes}m`);
    if (remainingSeconds > 0) parts.push(`${remainingSeconds}s`);
    return `${parts.join(" ")} (${sec}s)`;
};

export const computeCallbackStats = (
    beaconNode: any,
    fallbackIntervalSeconds?: number | null
): CallbackStats => {
    const rawInterval = beaconNode?.interval ?? fallbackIntervalSeconds;
    const parsedInterval =
        rawInterval != null && !Number.isNaN(Number(rawInterval)) && Number(rawInterval) > 0
            ? Number(rawInterval)
            : null;

    const edges = Array.isArray(beaconNode?.history?.edges) ? beaconNode.history.edges : [];
    const nodes: BeaconHistoryNode[] = edges
        .map((e: any) => e?.node)
        .filter((n: any): n is BeaconHistoryNode => Boolean(n && typeof n.latency === "number"))
        .slice(-100);

    const onTimeThresholdMs = Math.max(
        ON_TIME_MIN_TOLERANCE_MS,
        (parsedInterval ?? 0) * 100
    );

    const sampleCount = nodes.length;
    const onTimeCount = nodes.filter((n) => n.latency <= onTimeThresholdMs).length;
    const lateCount = sampleCount - onTimeCount;
    const onTimePercentage =
        sampleCount > 0 ? Math.round((onTimeCount / sampleCount) * 100) : 100;

    const avgLatencyMs =
        sampleCount > 0
            ? Math.round(nodes.reduce((sum, n) => sum + Math.max(0, n.latency), 0) / sampleCount)
            : null;

    return {
        intervalSeconds: parsedInterval,
        formattedInterval: formatCallbackInterval(parsedInterval),
        sampleCount,
        onTimeCount,
        lateCount,
        onTimePercentage,
        avgLatencyMs,
        lastSeenAt: beaconNode?.lastSeenAt ?? null,
        nextSeenAt: beaconNode?.nextSeenAt ?? null,
    };
};

export const useCallbackTimer = (beaconData: any, fallbackIntervalSeconds?: number | null) => {
    const [timeUntilCallback, setTimeUntilCallback] = useState<string>("");
    const [isMissedCallback, setIsMissedCallback] = useState(false);
    const [isLateCheckin, setIsLateCheckin] = useState(false);

    const nextSeenAt = beaconData?.node?.nextSeenAt;

    useEffect(() => {
        if (!nextSeenAt) return;
        const updateTimer = () => {
            const next = moment(nextSeenAt);
            const now = moment();
            const diff = next.diff(now, 'seconds');

            if (diff > 1 || diff < -1) {
                if (diff > 1) {
                    setTimeUntilCallback(next.fromNow());
                    setIsMissedCallback(false);
                    setIsLateCheckin(false);
                } else {
                    setTimeUntilCallback(`expected ${next.fromNow()}`);
                    setIsMissedCallback(true);

                    if (diff < -300) {
                        setIsLateCheckin(true);
                    } else {
                        setIsLateCheckin(false);
                    }
                }
            } else {
                setTimeUntilCallback("now");
                setIsMissedCallback(false);
                setIsLateCheckin(false);
            }
        };

        updateTimer(); // Initial call
        const intervalId = setInterval(updateTimer, 1000);
        return () => clearInterval(intervalId);
    }, [nextSeenAt]);

    const callbackStats = useMemo(
        () => computeCallbackStats(beaconData?.node, fallbackIntervalSeconds),
        [beaconData, fallbackIntervalSeconds]
    );

    return { timeUntilCallback, isMissedCallback, isLateCheckin, callbackStats };
};
