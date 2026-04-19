import { FC } from "react";
import {
    BarChart,
    Bar,
    XAxis,
    YAxis,
    CartesianGrid,
    Tooltip,
    Legend,
    ResponsiveContainer,
} from "recharts";
import { ApolloError } from "@apollo/client";
import { getTacticColor } from "../../../utils/utils";
import { EmptyState, EmptyStateType } from "../../../components/tavern-base-ui/EmptyState";
import { CustomTooltip } from "./CustomTooltip";
import { CustomLegend } from "./CustomLegend";
import { ChartDataPoint } from "./types";

interface QuestTimelineChartProps {
    loading: boolean;
    chartData: ChartDataPoint[];
    error: ApolloError | undefined;
    activeTactics: string[];
}

export const QuestTimelineChart: FC<QuestTimelineChartProps> = ({
    loading,
    chartData,
    error,
    activeTactics,
}) => {
    if (loading && !chartData.length) {
        return (
            <EmptyState
                type={EmptyStateType.loading}
                label="Loading quest timeline..."
            />
        );
    }

    if (error) {
        return (
            <EmptyState
                type={EmptyStateType.error}
                label="Failed to load quest timeline"
                details={error.message}
            />
        );
    }

    if (chartData.length === 0) {
        return (
            <EmptyState
                type={EmptyStateType.noData}
                label="No quest data available"
                details="Create some quests to see the timeline"
            />
        );
    }

    return (
        <ResponsiveContainer width="100%" height="100%">
            <BarChart
                data={chartData}
                margin={{ top: 20, right: 30, left: 20, bottom: 5 }}
            >
                <CartesianGrid strokeDasharray="3 3" stroke="#e5e7eb" />
                <XAxis
                    dataKey="displayLabel"
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
                <Tooltip content={<CustomTooltip />} />
                <Legend content={<CustomLegend />} />

                {activeTactics.map((tactic, index) => (
                    <Bar
                        key={tactic}
                        dataKey={tactic}
                        stackId="tactics"
                        fill={getTacticColor(tactic)}
                        name={tactic}
                        radius={
                            index === activeTactics.length - 1
                                ? [4, 4, 0, 0]
                                : [0, 0, 0, 0]
                        }
                    />
                ))}
            </BarChart>
        </ResponsiveContainer>
    );
};
