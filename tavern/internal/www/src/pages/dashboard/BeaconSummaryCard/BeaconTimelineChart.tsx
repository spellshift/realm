import { FC } from "react";
import {
    LineChart,
    Line,
    XAxis,
    YAxis,
    CartesianGrid,
    Tooltip,
    ResponsiveContainer,
} from "recharts";
import { ApolloError } from "@apollo/client";
import { EmptyState, EmptyStateType } from "../../../components/tavern-base-ui/EmptyState";
import { ChartDataPoint } from "../utils/types";

interface BeaconTimelineChartProps {
    loading: boolean;
    chartData: ChartDataPoint[];
    error: ApolloError | undefined;
    tickInterval: number;
}

export const BeaconTimelineChart: FC<BeaconTimelineChartProps> = ({ loading, chartData, error, tickInterval }) => {
    if (loading && !chartData.length) {
        return (
            <EmptyState
                type={EmptyStateType.loading}
                label="Loading beacon timeline..."
            />
        );
    }

    if (error) {
        return (
            <EmptyState
                type={EmptyStateType.error}
                label="Failed to load beacon timeline"
                details={error.message}
            />
        );
    }

    if (chartData.length === 0) {
        return (
            <EmptyState
                type={EmptyStateType.noData}
                label="No beacon data available"
                details="Beacons will appear here once they check in"
            />
        );
    }

    return (
        <ResponsiveContainer width="100%" height="100%">
            <LineChart
                data={chartData}
                margin={{ top: 20, right: 30, left: 20, bottom: 5 }}
            >
                <CartesianGrid strokeDasharray="3 3" stroke="#e5e7eb" />
                <XAxis
                    dataKey="displayLabel"
                    interval={tickInterval - 1}
                    tick={{ fill: "#6b7280", fontSize: 12 }}
                    tickLine={{ stroke: "#e5e7eb" }}
                    axisLine={{ stroke: "#e5e7eb" }}
                />
                <YAxis
                    tick={{ fill: "#6b7280", fontSize: 12 }}
                    tickLine={{ stroke: "#e5e7eb" }}
                    axisLine={{ stroke: "#e5e7eb" }}
                    allowDecimals={false}
                />
                <Tooltip
                    contentStyle={{ borderRadius: "8px", border: "1px solid #e5e7eb" }}
                    labelStyle={{ fontWeight: 600, color: "#111827" }}
                />
                <Line
                    type="monotone"
                    dataKey="total"
                    stroke="#7c3aed"
                    name="Beacons"
                    dot={false}
                    strokeWidth={2}
                />
            </LineChart>
        </ResponsiveContainer>
    );
};
