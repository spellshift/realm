import { TimelineDataPoint } from "../utils/types";

export type TimelineMetric = {
    count: number;
    trend?: "up" | "down";
    timeframe?: string;
    trendValue?: string;
};

export const computeMetric = (chartData: TimelineDataPoint[]): TimelineMetric => {
    // The final bucket is always empty (in-progress window), so exclude it
    const buckets = chartData.length > 1 ? chartData.slice(0, -1) : chartData;

    //beacon metric shoudl represent current count not addetitive
    const count = buckets.length >= 1 ? chartData[buckets.length - 1].total : 0;

    if (buckets.length < 2) return { count, trend: undefined, timeframe: undefined, trendValue: undefined };

    const start = buckets[0];
    const end = buckets[buckets.length - 1];

    const trend: "up" | "down" | undefined =
        end.total === start.total ? undefined : end.total > start.total ? "up" : "down";

    const trendValue = start.total === 0
        ? undefined
        : `${Math.round(((end.total - start.total) / start.total) * 100)}%`;

    return {
        count,
        trend,
        timeframe: `since ${start.displayLabel}`,
        trendValue,
    };
};
