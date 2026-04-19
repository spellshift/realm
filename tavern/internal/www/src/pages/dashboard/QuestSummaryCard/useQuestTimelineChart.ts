import { useMemo, useState } from "react";
import { useQuery } from "@apollo/client";
import moment from "moment";
import { TIME_RANGES, TIME_RANGE_CONFIG, ALL_TACTICS, GET_QUEST_TIMELINE_CHART } from "./config";
import { TimeRange, QuestTimelineChartResponse, ChartDataPoint } from "./types";

export const useQuestTimelineChart = () => {
    const [timeRange, setTimeRange] = useState<TimeRange>("today");

    const config = TIME_RANGE_CONFIG[timeRange];
    const selectedIndex = TIME_RANGES.indexOf(timeRange);

    const queryVariables = useMemo(() => {
        const now = moment().startOf("hour").add(1, "hour");
        const start = moment().subtract(config.daysBack, "days").startOf("hour");

        return {
            start: start.toISOString(),
            end: now.toISOString(),
            granularity_seconds: config.granularity_seconds,
        };
    }, [config]);

    const { data, loading, error } = useQuery<QuestTimelineChartResponse>(
        GET_QUEST_TIMELINE_CHART,
        {
            variables: queryVariables,
            fetchPolicy: "cache-and-network",
            pollInterval: 5000,
        }
    );

    const chartData = useMemo((): ChartDataPoint[] => {
        if (!data?.metrics?.questTimelineChart) return [];

        const buckets = data.metrics.questTimelineChart;
        const firstNonZeroIndex = buckets.findIndex((bucket) => bucket.count > 0);

        // All buckets are empty — return nothing
        if (firstNonZeroIndex === -1) return [];

        return buckets.slice(firstNonZeroIndex).map((bucket) => {
            const timestamp = moment(bucket.startTimestamp);
            const displayLabel = timestamp.format(config.formatString);

            const tacticCounts: Record<string, number> = {};
            ALL_TACTICS.forEach((tactic) => {
                tacticCounts[tactic] = 0;
            });

            bucket.groupByTactic.forEach((tacticBucket) => {
                tacticCounts[tacticBucket.tactic] = tacticBucket.count;
            });

            return {
                timestamp: bucket.startTimestamp,
                displayLabel,
                total: bucket.count,
                ...tacticCounts,
            };
        });
    }, [data, config.formatString]);

    const activeTactics = useMemo((): string[] => {
        if (!chartData.length) return [];

        const tacticTotals: Record<string, number> = {};

        chartData.forEach((dataPoint) => {
            ALL_TACTICS.forEach((tactic) => {
                const count = dataPoint[tactic] as number;
                tacticTotals[tactic] = (tacticTotals[tactic] || 0) + count;
            });
        });

        return ALL_TACTICS.filter((tactic) => tacticTotals[tactic] > 0);
    }, [chartData]);

    const questMetric = useMemo(() => {
        // The final bucket is always empty (in-progress window), so exclude it
        const buckets = chartData.length > 1 ? chartData.slice(0, -1) : chartData;
        const count = buckets.reduce((sum, point) => sum + point.total, 0);

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
    }, [chartData]);

    const handleTabChange = (index: number) => {
        setTimeRange(TIME_RANGES[index]);
    };

    return {
        timeRange,
        selectedIndex,
        chartData,
        activeTactics,
        questMetric,
        loading,
        error,
        handleTabChange,
    };
};
