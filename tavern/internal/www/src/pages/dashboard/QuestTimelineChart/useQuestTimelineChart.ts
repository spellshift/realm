import { useMemo, useState } from "react";
import { useQuery } from "@apollo/client";
import moment from "moment";
import { TIME_RANGES, TIME_RANGE_CONFIG, ALL_TACTICS, GET_QUEST_TIMELINE_CHART } from "./config";
import { TimeRange, QuestTimelineChartResponse, ChartDataPoint } from "./types";

export const useQuestTimelineChart = () => {
    const [timeRange, setTimeRange] = useState<TimeRange>("last3days");

    const config = TIME_RANGE_CONFIG[timeRange];
    const selectedIndex = TIME_RANGES.indexOf(timeRange);

    const queryVariables = useMemo(() => {
        const now = moment();
        const start = moment().subtract(config.daysBack, "days");

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

        return data.metrics.questTimelineChart.map((bucket) => {
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

    const handleTabChange = (index: number) => {
        setTimeRange(TIME_RANGES[index]);
    };

    return {
        timeRange,
        selectedIndex,
        chartData,
        activeTactics,
        loading,
        error,
        handleTabChange,
    };
};
