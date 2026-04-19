import { useMemo, useState } from "react";
import { useQuery } from "@apollo/client";
import moment from "moment";
import { BEACON_TIME_RANGE_CONFIG, TIME_RANGES, TimeRange, beaconComputeTimeWindow } from "../utils/timeRange";
import { GET_BEACON_TIMELINE_CHART } from "./config";
import { BeaconTimelineChartResponse, ChartDataPoint } from "../utils/types";
import { computeMetric } from "./computeMetric";

export const useBeaconTimelineChart = () => {
    const [timeRange, setTimeRange] = useState<TimeRange>("today");

    const config = BEACON_TIME_RANGE_CONFIG[timeRange];
    const selectedIndex = TIME_RANGES.indexOf(timeRange);

    const queryVariables = useMemo(() => {
        const { start, stop } = beaconComputeTimeWindow(config);
        return {
            start: start.toISOString(),
            end: stop.toISOString(),
            granularity_seconds: config.granularity_seconds,
        };
    }, [config]);

    const { data, loading, error } = useQuery<BeaconTimelineChartResponse>(
        GET_BEACON_TIMELINE_CHART,
        {
            variables: queryVariables,
            fetchPolicy: "cache-and-network",
            pollInterval: 5000,
        }
    );

    const chartData = useMemo((): ChartDataPoint[] => {
        if (!data?.metrics?.beaconTimelineChart) return [];

        const buckets = data.metrics.beaconTimelineChart;
        const firstNonZeroIndex = buckets.findIndex((bucket) => bucket.count > 0);

        if (firstNonZeroIndex === -1) return [];

        return buckets.slice(firstNonZeroIndex).map((bucket) => ({
            timestamp: bucket.startTimestamp,
            displayLabel: moment(bucket.startTimestamp).format(config.formatString),
            total: bucket.count,
        }));
    }, [data, config.formatString]);

    const beaconMetric = useMemo(() => computeMetric(chartData), [chartData]);

    const handleTabChange = (index: number) => {
        setTimeRange(TIME_RANGES[index]);
    };

    return {
        selectedIndex,
        chartData,
        beaconMetric,
        tickInterval: config.tickInterval,
        loading,
        error,
        handleTabChange,
    };
};
