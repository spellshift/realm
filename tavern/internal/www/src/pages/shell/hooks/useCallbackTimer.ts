import { useEffect, useMemo, useState } from "react";

export interface BeaconHistoryNode {
    id?: string;
    createdAt?: string;
    latency: number;
}

export interface CallbackStats {
    intervalSeconds: number | null;
    formattedInterval: string;
    sampleCount: number;
    expectedCount: number;
    onTimeCount: number;
    lateCount: number;
    missedCount: number;
    onTimePercentage: number;
    avgLatencyMs: number | null;
    lastSeenAt: string | null;
    nextSeenAt: string | null;
}

export const ON_TIME_MIN_TOLERANCE_MS = 2000;
const OVERDUE_MISSED_GRACE_MS = 1000;

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

export const formatCountdownDuration = (totalSeconds: number): string => {
    const abs = Math.max(0, Math.round(Math.abs(totalSeconds)));
    if (abs < 60) {
        return `${abs}s`;
    }
    if (abs < 3600) {
        return `${Math.floor(abs / 60)}min`;
    }
    if (abs < 86400) {
        return `${Math.floor(abs / 3600)}hr`;
    }
    return `${Math.floor(abs / 86400)}d`;
};

export const computeCallbackStats = (
    beaconNode: any,
    fallbackIntervalSeconds?: number | null,
    nowMs: number = Date.now()
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

    let missedCount = 0;

    if (parsedInterval !== null) {
        const intervalMs = parsedInterval * 1000;
        const gapGraceMs = Math.max(onTimeThresholdMs, intervalMs * 0.5);

        const timestamps = nodes
            .map((n) => (n.createdAt ? new Date(n.createdAt).getTime() : NaN))
            .filter((t) => Number.isFinite(t))
            .sort((a, b) => a - b);

        for (let i = 1; i < timestamps.length; i += 1) {
            const deltaMs = timestamps[i] - timestamps[i - 1];
            if (deltaMs > intervalMs + gapGraceMs) {
                missedCount += Math.floor((deltaMs - gapGraceMs) / intervalMs);
            }
        }

        const nextSeenStr = beaconNode?.nextSeenAt;
        const lastSeenStr = beaconNode?.lastSeenAt;
        const nextMs = nextSeenStr ? new Date(nextSeenStr).getTime() : NaN;
        const expectedNextMs = Number.isFinite(nextMs)
            ? nextMs
            : lastSeenStr && Number.isFinite(new Date(lastSeenStr).getTime())
            ? new Date(lastSeenStr).getTime() + intervalMs
            : NaN;

        if (Number.isFinite(expectedNextMs) && nowMs - expectedNextMs > OVERDUE_MISSED_GRACE_MS) {
            missedCount += 1 + Math.floor((nowMs - expectedNextMs) / intervalMs);
        }
    } else if (beaconNode?.nextSeenAt) {
        const nextMs = new Date(beaconNode.nextSeenAt).getTime();
        if (Number.isFinite(nextMs) && nowMs - nextMs > OVERDUE_MISSED_GRACE_MS) {
            missedCount += 1;
        }
    }

    const expectedCount = sampleCount + missedCount;
    const onTimePercentage =
        expectedCount > 0 ? Math.round((onTimeCount / expectedCount) * 100) : 100;

    const avgLatencyMs =
        sampleCount > 0
            ? Math.round(nodes.reduce((sum, n) => sum + Math.max(0, n.latency), 0) / sampleCount)
            : null;

    return {
        intervalSeconds: parsedInterval,
        formattedInterval: formatCallbackInterval(parsedInterval),
        sampleCount,
        expectedCount,
        onTimeCount,
        lateCount,
        missedCount,
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
    const [nowMs, setNowMs] = useState<number>(() => Date.now());

    const nextSeenAt = beaconData?.node?.nextSeenAt;

    useEffect(() => {
        if (!nextSeenAt) return;
        const updateTimer = () => {
            const nextMs = new Date(nextSeenAt).getTime();
            if (!Number.isFinite(nextMs)) {
                return;
            }
            const currentNow = Date.now();
            setNowMs(currentNow);
            const diffSeconds = Math.round((nextMs - currentNow) / 1000);

            if (diffSeconds > 1) {
                setTimeUntilCallback(`Next ${formatCountdownDuration(diffSeconds)}`);
                setIsMissedCallback(false);
                setIsLateCheckin(false);
            } else if (diffSeconds < -1) {
                setTimeUntilCallback(`Missing ${formatCountdownDuration(Math.abs(diffSeconds))} ago`);
                setIsMissedCallback(true);
                setIsLateCheckin(diffSeconds < -300);
            } else {
                setTimeUntilCallback("Next now");
                setIsMissedCallback(false);
                setIsLateCheckin(false);
            }
        };

        updateTimer();
        const intervalId = setInterval(updateTimer, 1000);
        return () => clearInterval(intervalId);
    }, [nextSeenAt]);

    const callbackStats = useMemo(
        () => computeCallbackStats(beaconData?.node, fallbackIntervalSeconds, nowMs),
        [beaconData, fallbackIntervalSeconds, nowMs]
    );

    return { timeUntilCallback, isMissedCallback, isLateCheckin, callbackStats };
};
